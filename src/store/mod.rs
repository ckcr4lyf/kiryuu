mod stats;

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use ahash::RandomState;
use dashmap::mapref::entry::Entry;
use dashmap::DashMap;
use parking_lot::Mutex;

use crate::query;

pub use stats::{TrackerStats, HISTOGRAM_BUCKET_UPPER_BOUNDS_SECS};

pub type InfoHash = [u8; 20];
pub type PeerId = [u8; 6];

pub const PEER_TTL: Duration = Duration::from_secs(60 * 31);
pub const TORRENT_TTL: Duration = Duration::from_secs(60 * 31);
pub const SWEEP_INTERVAL: Duration = Duration::from_secs(10);
/// How often the GC thread recomputes peer/torrent gauges.
/// Must be ≥ Prometheus scrape interval so we walk the map *less* often than scrapes.
pub const PEER_TOTALS_REFRESH: Duration = Duration::from_secs(60);
/// Round-robin stripes: each tick scans one stripe (`hash(infohash) % STRIPES`).
/// Full coverage every `SWEEP_STRIPES * SWEEP_INTERVAL` (~42.7 min with defaults).
pub const SWEEP_STRIPES: usize = 256;
pub const PEER_SAMPLE_LIMIT: usize = 50;
/// Promote from inline peers to HashMap beyond this size.
const INLINE_PEER_CAP: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PeerPool {
    Seeder,
    Leecher,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnnounceEvent {
    Unknown,
    Stopped,
    Completed,
}

pub struct AnnounceInput {
    pub info_hash: InfoHash,
    pub peer: PeerId,
    pub is_seeding: bool,
    pub event: AnnounceEvent,
}

/// Packed peer metadata: bit31 = seeder, bits0..30 = expiry seconds since store epoch.
#[derive(Clone, Copy, Debug)]
struct PeerMeta(u32);

impl PeerMeta {
    fn new(is_seeder: bool, expires_at: u32) -> Self {
        let flag = if is_seeder { 1u32 << 31 } else { 0 };
        Self(flag | (expires_at & 0x7fff_ffff))
    }

    fn is_seeder(self) -> bool {
        self.0 & (1 << 31) != 0
    }

    fn expires_at(self) -> u32 {
        self.0 & 0x7fff_ffff
    }

    fn is_live(self, now: u32) -> bool {
        self.expires_at() > now
    }
}

/// Dense storage for the common case (1–few peers). Avoids HashMap overhead per torrent.
#[derive(Clone, Debug, Default)]
enum Peers {
    #[default]
    Empty,
    /// Inline list, length in `len` (1..=INLINE_PEER_CAP).
    Inline {
        entries: [(PeerId, PeerMeta); INLINE_PEER_CAP],
        len: u8,
    },
    Map(HashMap<PeerId, PeerMeta>),
}

impl Peers {
    fn len(&self) -> usize {
        match self {
            Peers::Empty => 0,
            Peers::Inline { len, .. } => *len as usize,
            Peers::Map(map) => map.len(),
        }
    }

    fn get(&self, peer: &PeerId) -> Option<PeerMeta> {
        match self {
            Peers::Empty => None,
            Peers::Inline { entries, len } => entries[..*len as usize]
                .iter()
                .find(|(id, _)| id == peer)
                .map(|(_, meta)| *meta),
            Peers::Map(map) => map.get(peer).copied(),
        }
    }

    fn insert(&mut self, peer: PeerId, meta: PeerMeta) {
        match self {
            Peers::Empty => {
                let mut entries = [([0u8; 6], PeerMeta(0)); INLINE_PEER_CAP];
                entries[0] = (peer, meta);
                *self = Peers::Inline { entries, len: 1 };
            }
            Peers::Inline { entries, len } => {
                let n = *len as usize;
                if let Some(slot) = entries[..n].iter_mut().find(|(id, _)| *id == peer) {
                    slot.1 = meta;
                    return;
                }
                if n < INLINE_PEER_CAP {
                    entries[n] = (peer, meta);
                    *len += 1;
                    return;
                }
                let mut map = HashMap::with_capacity(INLINE_PEER_CAP * 2);
                for (id, m) in entries.iter().take(n) {
                    map.insert(*id, *m);
                }
                map.insert(peer, meta);
                *self = Peers::Map(map);
            }
            Peers::Map(map) => {
                map.insert(peer, meta);
            }
        }
    }

    fn remove(&mut self, peer: &PeerId) -> bool {
        match self {
            Peers::Empty => false,
            Peers::Inline { entries, len } => {
                let n = *len as usize;
                let Some(idx) = entries[..n].iter().position(|(id, _)| id == peer) else {
                    return false;
                };
                entries[idx] = entries[n - 1];
                *len -= 1;
                if *len == 0 {
                    *self = Peers::Empty;
                }
                true
            }
            Peers::Map(map) => {
                let removed = map.remove(peer).is_some();
                if map.is_empty() {
                    *self = Peers::Empty;
                }
                removed
            }
        }
    }

    fn purge_expired(&mut self, now: u32) {
        match self {
            Peers::Empty => {}
            Peers::Inline { entries, len } => {
                let mut write = 0usize;
                let n = *len as usize;
                for read in 0..n {
                    if entries[read].1.is_live(now) {
                        entries[write] = entries[read];
                        write += 1;
                    }
                }
                *len = write as u8;
                if *len == 0 {
                    *self = Peers::Empty;
                }
            }
            Peers::Map(map) => {
                map.retain(|_, meta| meta.is_live(now));
                if map.is_empty() {
                    *self = Peers::Empty;
                }
            }
        }
    }

    fn counts(&self, now: u32) -> (usize, usize) {
        let mut seeders = 0usize;
        let mut leechers = 0usize;
        self.for_each_live(now, |_, meta| {
            if meta.is_seeder() {
                seeders += 1;
            } else {
                leechers += 1;
            }
        });
        (seeders, leechers)
    }

    fn sample(&self, now: u32, want_seeders: bool, limit: usize) -> Vec<[u8; 6]> {
        let mut out = Vec::with_capacity(limit.min(16));
        self.for_each_live(now, |peer, meta| {
            if out.len() >= limit {
                return;
            }
            if meta.is_seeder() == want_seeders {
                out.push(peer);
            }
        });
        out
    }

    fn for_each_live(&self, now: u32, mut f: impl FnMut(PeerId, PeerMeta)) {
        match self {
            Peers::Empty => {}
            Peers::Inline { entries, len } => {
                for (peer, meta) in entries[..*len as usize].iter() {
                    if meta.is_live(now) {
                        f(*peer, *meta);
                    }
                }
            }
            Peers::Map(map) => {
                for (peer, meta) in map.iter() {
                    if meta.is_live(now) {
                        f(*peer, *meta);
                    }
                }
            }
        }
    }

    fn raw_counts(&self) -> (usize, usize) {
        let mut seeders = 0usize;
        let mut leechers = 0usize;
        match self {
            Peers::Empty => {}
            Peers::Inline { entries, len } => {
                for (_, meta) in entries[..*len as usize].iter() {
                    if meta.is_seeder() {
                        seeders += 1;
                    } else {
                        leechers += 1;
                    }
                }
            }
            Peers::Map(map) => {
                for meta in map.values() {
                    if meta.is_seeder() {
                        seeders += 1;
                    } else {
                        leechers += 1;
                    }
                }
            }
        }
        (seeders, leechers)
    }
}

struct Torrent {
    peers: Peers,
    last_activity: u32,
}

impl Torrent {
    fn new(now: u32) -> Self {
        Self {
            peers: Peers::Empty,
            last_activity: now,
        }
    }

    fn touch(&mut self, now: u32) {
        self.last_activity = now;
    }

    fn is_stale(&self, now: u32) -> bool {
        now.saturating_sub(self.last_activity) > TORRENT_TTL.as_secs() as u32
    }
}

type StripeKeySet = HashSet<InfoHash, RandomState>;

pub struct TrackerStore {
    torrents: DashMap<InfoHash, Torrent, RandomState>,
    /// Keys known to belong to each GC stripe. Maintained on create/delete so
    /// sweep can walk O(stripe) instead of filtering a full DashMap iter.
    stripe_keys: [Mutex<StripeKeySet>; SWEEP_STRIPES],
    epoch: Instant,
    /// Next stripe index for fair stale-torrent GC.
    sweep_stripe: AtomicUsize,
    /// Cached by `refresh_peer_totals` (GC thread); safe for metrics hot path.
    cached_torrent_count: AtomicUsize,
    cached_seeder_count: AtomicUsize,
    cached_leecher_count: AtomicUsize,
    cached_stripe_index_len: AtomicUsize,
    pub stats: TrackerStats,
}

impl TrackerStore {
    pub fn new() -> Self {
        Self {
            torrents: DashMap::with_hasher(RandomState::new()),
            stripe_keys: std::array::from_fn(|_| {
                Mutex::new(HashSet::with_hasher(RandomState::new()))
            }),
            epoch: Instant::now(),
            sweep_stripe: AtomicUsize::new(0),
            cached_torrent_count: AtomicUsize::new(0),
            cached_seeder_count: AtomicUsize::new(0),
            cached_leecher_count: AtomicUsize::new(0),
            cached_stripe_index_len: AtomicUsize::new(0),
            stats: TrackerStats::default(),
        }
    }

    fn now_secs(&self) -> u32 {
        self.epoch.elapsed().as_secs() as u32
    }

    fn stripe_of(info_hash: &InfoHash) -> usize {
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&info_hash[..8]);
        (u64::from_le_bytes(buf) as usize) % SWEEP_STRIPES
    }

    fn index_insert(&self, info_hash: InfoHash) {
        let stripe = Self::stripe_of(&info_hash);
        self.stripe_keys[stripe].lock().insert(info_hash);
    }

    fn index_remove(&self, info_hash: &InfoHash) {
        let stripe = Self::stripe_of(info_hash);
        self.stripe_keys[stripe].lock().remove(info_hash);
    }

    /// Drop a torrent only if it is *still* stale, then unindex it.
    ///
    /// The conditional matters: a concurrent announce can refresh a torrent
    /// between the staleness check and the delete. Removing it anyway would
    /// discard live peers and make `handle_announce`'s `get_mut` expect fail.
    /// Skipping the index removal when the map removal is declined also keeps
    /// the index from losing a key whose torrent survives (an unindexed torrent
    /// is invisible to every future sweep).
    fn remove_torrent_if_stale(&self, info_hash: &InfoHash, now: u32) -> bool {
        if self
            .torrents
            .remove_if(info_hash, |_, torrent| torrent.is_stale(now))
            .is_some()
        {
            self.index_remove(info_hash);
            return true;
        }
        false
    }

    /// Ensure a live torrent entry exists; index new keys only on first insert.
    ///
    /// Lock-order invariant: never take a `stripe_keys` mutex while holding a
    /// DashMap guard in the *reverse* direction — i.e. code may go
    /// map-lock → stripe-lock, but must never hold a stripe lock and then take a
    /// map lock. `sweep_stale_torrents_at` upholds this by releasing the stripe
    /// mutex before touching `torrents`.
    fn ensure_torrent(&self, info_hash: InfoHash, now: u32) {
        match self.torrents.entry(info_hash) {
            Entry::Occupied(mut occupied) => {
                if occupied.get().is_stale(now) {
                    *occupied.get_mut() = Torrent::new(now);
                }
            }
            Entry::Vacant(vacant) => {
                vacant.insert(Torrent::new(now));
                self.index_insert(info_hash);
            }
        }
    }

    pub fn handle_announce(&self, input: AnnounceInput, started: Instant) -> Vec<u8> {
        let now = self.now_secs();
        let peer_ttl = PEER_TTL.as_secs() as u32;

        self.ensure_torrent(input.info_hash, now);

        let mut torrent_ref = self
            .torrents
            .get_mut(&input.info_hash)
            .expect("torrent entry was just created");

        let torrent = torrent_ref.value_mut();
        torrent.touch(now);
        torrent.peers.purge_expired(now);

        let existing = torrent.peers.get(&input.peer);
        let is_seeder = existing.map(|m| m.is_seeder()).unwrap_or(false);
        let is_leecher = existing.map(|m| !m.is_seeder()).unwrap_or(false);

        let (seed_count_mod, leech_count_mod) =
            Self::count_mods(input.event, input.is_seeding, is_seeder, is_leecher);

        let (seeders_len, leechers_len) = torrent.peers.counts(now);
        let seeders = torrent.peers.sample(now, true, PEER_SAMPLE_LIMIT);
        let leechers = torrent.peers.sample(now, false, PEER_SAMPLE_LIMIT);
        let final_res = query::announce_reply(
            seeders_len as i64 + seed_count_mod,
            leechers_len as i64 + leech_count_mod,
            &seeders,
            &leechers,
        );

        Self::apply_announce(torrent, &input, now + peer_ttl, is_seeder, is_leecher);

        if seed_count_mod == 0 && leech_count_mod == 0 {
            self.stats.record_nochange();
        }

        drop(torrent_ref);
        self.stats.record_announce(started.elapsed());
        final_res
    }

    fn count_mods(
        event: AnnounceEvent,
        is_seeding: bool,
        is_seeder: bool,
        is_leecher: bool,
    ) -> (i64, i64) {
        let mut seed_count_mod: i64 = 0;
        let mut leech_count_mod: i64 = 0;

        if event == AnnounceEvent::Stopped {
            if is_seeder {
                seed_count_mod -= 1;
            } else if is_leecher {
                leech_count_mod -= 1;
            }
        } else if is_seeding {
            if !is_seeder {
                seed_count_mod += 1;
            }
            if event == AnnounceEvent::Completed && is_leecher {
                leech_count_mod -= 1;
            }
        } else if !is_leecher {
            leech_count_mod += 1;
        }

        (seed_count_mod, leech_count_mod)
    }

    fn apply_announce(
        torrent: &mut Torrent,
        input: &AnnounceInput,
        expires_at: u32,
        is_seeder: bool,
        is_leecher: bool,
    ) {
        if input.event == AnnounceEvent::Stopped {
            if is_seeder || is_leecher {
                torrent.peers.remove(&input.peer);
            }
            return;
        }

        // Completed: replace leecher entry with seeder (single map, one insert).
        torrent
            .peers
            .insert(input.peer, PeerMeta::new(input.is_seeding, expires_at));
    }

    pub fn peer_exists(&self, info_hash: InfoHash, pool: PeerPool, peer: PeerId) -> bool {
        let now = self.now_secs();
        let Some(torrent_ref) = self.torrents.get(&info_hash) else {
            return false;
        };

        if torrent_ref.is_stale(now) {
            drop(torrent_ref);
            self.remove_torrent_if_stale(&info_hash, now);
            return false;
        }

        let Some(meta) = torrent_ref.peers.get(&peer) else {
            return false;
        };
        if !meta.is_live(now) {
            return false;
        }

        match pool {
            PeerPool::Seeder => meta.is_seeder(),
            PeerPool::Leecher => !meta.is_seeder(),
        }
    }

    pub fn seed_peers(&self, info_hash: InfoHash, pool: PeerPool, peers: &[[u8; 6]]) {
        let now = self.now_secs();
        let expires_at = now + PEER_TTL.as_secs() as u32;
        let is_seeder = matches!(pool, PeerPool::Seeder);

        self.ensure_torrent(info_hash, now);

        let mut torrent_ref = self
            .torrents
            .get_mut(&info_hash)
            .expect("torrent entry was just created");

        let torrent = torrent_ref.value_mut();
        torrent.touch(now);
        for peer in peers {
            torrent
                .peers
                .insert(*peer, PeerMeta::new(is_seeder, expires_at));
        }
    }

    pub fn torrent_count(&self) -> usize {
        self.torrents.len()
    }

    /// Cached peer totals — O(1). Refresh via `refresh_peer_totals` on the GC thread.
    pub fn peer_totals(&self) -> (usize, usize, usize) {
        (
            self.cached_torrent_count.load(Ordering::Relaxed),
            self.cached_seeder_count.load(Ordering::Relaxed),
            self.cached_leecher_count.load(Ordering::Relaxed),
        )
    }

    /// Cached stripe-index size — O(1). Compare against `kiryuu_torrent_count`
    /// to spot index drift.
    pub fn stripe_index_size(&self) -> usize {
        self.cached_stripe_index_len.load(Ordering::Relaxed)
    }

    /// Total keys tracked across all stripe indexes.
    fn stripe_index_len(&self) -> usize {
        self.stripe_keys.iter().map(|s| s.lock().len()).sum()
    }

    /// Re-index torrents that are missing a stripe entry, returning how many were
    /// repaired.
    ///
    /// A create racing a sweep delete can leave a torrent in the map but absent
    /// from the index: `remove_torrent_if_stale` drops the map entry, an announce
    /// re-creates it (re-inserting the key), then the sweep's `index_remove`
    /// deletes that fresh index entry. Such a torrent is invisible to every
    /// future sweep and would leak permanently, so heal it here. The sweep
    /// already heals the opposite direction (index entry, no torrent).
    ///
    /// O(N) — GC thread only, and only when the sizes actually disagree.
    fn repair_stripe_index(&self) -> usize {
        let mut repaired = 0usize;
        for torrent in self.torrents.iter() {
            let info_hash = *torrent.key();
            let stripe = Self::stripe_of(&info_hash);
            // map-lock → stripe-lock is the permitted order; see `ensure_torrent`.
            if self.stripe_keys[stripe].lock().insert(info_hash) {
                repaired += 1;
            }
        }
        repaired
    }

    /// Full scan of torrents; call from a dedicated OS thread, never from Actix workers.
    pub fn refresh_peer_totals(&self) {
        let started = Instant::now();
        let mut seeders = 0usize;
        let mut leechers = 0usize;

        for torrent in self.torrents.iter() {
            let (s, l) = torrent.peers.raw_counts();
            seeders += s;
            leechers += l;
        }

        let torrent_count = self.torrents.len();
        let mut index_len = self.stripe_index_len();

        // Size disagreement means drift in one direction or the other. Surplus
        // index keys are orphans the sweep clears within a full rotation; a
        // shortfall is an unindexed torrent, which never GCs on its own.
        if index_len != torrent_count {
            let repaired = self.repair_stripe_index();
            if repaired > 0 {
                self.stats.record_index_repair(repaired);
                index_len = self.stripe_index_len();
            }
        }

        self.cached_torrent_count
            .store(torrent_count, Ordering::Relaxed);
        self.cached_seeder_count.store(seeders, Ordering::Relaxed);
        self.cached_leecher_count.store(leechers, Ordering::Relaxed);
        self.cached_stripe_index_len
            .store(index_len, Ordering::Relaxed);
        self.stats.record_totals_refresh(started.elapsed());
    }

    /// Scan one hash stripe and drop torrents past `TORRENT_TTL`.
    /// Call repeatedly (background timer) so all stripes rotate fairly.
    pub fn sweep_stale_torrents(&self) -> usize {
        self.sweep_stale_torrents_at(self.now_secs())
    }

    fn sweep_stale_torrents_at(&self, now: u32) -> usize {
        let started = Instant::now();
        let stripe = self.sweep_stripe.fetch_add(1, Ordering::Relaxed) % SWEEP_STRIPES;

        // Snapshot keys for this stripe only — O(stripe), not O(N).
        let keys: Vec<InfoHash> = self.stripe_keys[stripe].lock().iter().copied().collect();
        let visited = keys.len();

        let mut stale_keys = Vec::new();
        let mut orphan_keys = Vec::new();

        for key in keys {
            match self.torrents.get(&key) {
                None => orphan_keys.push(key),
                Some(torrent) if torrent.is_stale(now) => stale_keys.push(key),
                Some(_) => {}
            }
        }

        let orphans = orphan_keys.len();
        if !orphan_keys.is_empty() {
            let mut index = self.stripe_keys[stripe].lock();
            for key in orphan_keys {
                index.remove(&key);
            }
        }

        let mut removed = 0usize;
        for key in stale_keys {
            if self.remove_torrent_if_stale(&key, now) {
                removed += 1;
            }
        }

        self.stats
            .record_sweep(started.elapsed(), visited, removed, orphans);
        removed
    }

    /// Run a full rotation of all stripes (tests / manual GC).
    pub fn sweep_all_stripes(&self) -> usize {
        self.sweep_all_stripes_at(self.now_secs())
    }

    fn sweep_all_stripes_at(&self, now: u32) -> usize {
        let mut removed = 0usize;
        for _ in 0..SWEEP_STRIPES {
            removed += self.sweep_stale_torrents_at(now);
        }
        removed
    }
}

impl Default for TrackerStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn announce(store: &TrackerStore, input: AnnounceInput) -> Vec<u8> {
        store.handle_announce(input, Instant::now())
    }

    #[test]
    fn seeder_announce_registers_peer() {
        let store = TrackerStore::new();
        let hash = [0x41u8; 20];
        let peer = [127, 0, 0, 1, 17, 0x5c];

        announce(
            &store,
            AnnounceInput {
                info_hash: hash,
                peer,
                is_seeding: true,
                event: AnnounceEvent::Unknown,
            },
        );

        assert!(store.peer_exists(hash, PeerPool::Seeder, peer));
    }

    #[test]
    fn stopped_removes_peer() {
        let store = TrackerStore::new();
        let hash = [0x42u8; 20];
        let peer = [127, 0, 0, 1, 17, 0x5c];

        announce(
            &store,
            AnnounceInput {
                info_hash: hash,
                peer,
                is_seeding: true,
                event: AnnounceEvent::Unknown,
            },
        );
        announce(
            &store,
            AnnounceInput {
                info_hash: hash,
                peer,
                is_seeding: true,
                event: AnnounceEvent::Stopped,
            },
        );

        assert!(!store.peer_exists(hash, PeerPool::Seeder, peer));
    }

    #[test]
    fn completed_moves_leecher_to_seeder() {
        let store = TrackerStore::new();
        let hash = [0x43u8; 20];
        let peer = [127, 0, 0, 1, 17, 0x5c];

        announce(
            &store,
            AnnounceInput {
                info_hash: hash,
                peer,
                is_seeding: false,
                event: AnnounceEvent::Unknown,
            },
        );
        announce(
            &store,
            AnnounceInput {
                info_hash: hash,
                peer,
                is_seeding: true,
                event: AnnounceEvent::Completed,
            },
        );

        assert!(store.peer_exists(hash, PeerPool::Seeder, peer));
        assert!(!store.peer_exists(hash, PeerPool::Leecher, peer));
    }

    #[test]
    fn seeded_peers_appear_in_announce() {
        let store = TrackerStore::new();
        let hash = [0x44u8; 20];
        let peer = [10, 0, 0, 1, 0x11, 0x5c];
        let caller = [127, 0, 0, 1, 17, 0x5c];

        store.seed_peers(hash, PeerPool::Seeder, &[peer]);
        let body = announce(
            &store,
            AnnounceInput {
                info_hash: hash,
                peer: caller,
                is_seeding: false,
                event: AnnounceEvent::Unknown,
            },
        );

        assert!(body.windows(6).any(|w| w == peer));
    }

    #[test]
    fn sweep_removes_stale_torrents() {
        let store = TrackerStore::new();
        let hash = [0x55u8; 20];
        let peer = [127, 0, 0, 1, 17, 0x5c];

        announce(
            &store,
            AnnounceInput {
                info_hash: hash,
                peer,
                is_seeding: true,
                event: AnnounceEvent::Unknown,
            },
        );
        assert_eq!(store.torrent_count(), 1);

        let after_ttl = store.now_secs() + TORRENT_TTL.as_secs() as u32 + 1;
        // One stripe may miss this hash — rotate all stripes.
        assert_eq!(store.sweep_all_stripes_at(after_ttl), 1);
        assert_eq!(store.torrent_count(), 0);
    }

    #[test]
    fn sweep_keeps_active_torrents() {
        let store = TrackerStore::new();
        let hash = [0x56u8; 20];
        let peer = [127, 0, 0, 1, 17, 0x5c];

        announce(
            &store,
            AnnounceInput {
                info_hash: hash,
                peer,
                is_seeding: true,
                event: AnnounceEvent::Unknown,
            },
        );

        assert_eq!(store.sweep_all_stripes_at(store.now_secs()), 0);
        assert_eq!(store.torrent_count(), 1);
    }

    #[test]
    fn sweep_stripes_cover_all_stale_torrents() {
        let store = TrackerStore::new();
        let peer = [127, 0, 0, 1, 17, 0x5c];

        // Spread torrents across many stripes via distinct infohashes.
        for i in 0u16..512 {
            let mut hash = [0u8; 20];
            hash[0] = (i & 0xff) as u8;
            hash[1] = (i >> 8) as u8;
            hash[2] = 0x59;
            announce(
                &store,
                AnnounceInput {
                    info_hash: hash,
                    peer,
                    is_seeding: true,
                    event: AnnounceEvent::Unknown,
                },
            );
        }
        assert_eq!(store.torrent_count(), 512);
        assert_eq!(stripe_index_len(&store), 512);

        let after_ttl = store.now_secs() + TORRENT_TTL.as_secs() as u32 + 1;
        let removed = store.sweep_all_stripes_at(after_ttl);
        assert_eq!(removed, 512);
        assert_eq!(store.torrent_count(), 0);
        assert_eq!(stripe_index_len(&store), 0);
    }

    #[test]
    fn stripe_index_tracks_new_torrent() {
        let store = TrackerStore::new();
        let hash = [0x5au8; 20];
        let peer = [127, 0, 0, 1, 17, 0x5c];
        let stripe = TrackerStore::stripe_of(&hash);

        assert!(store.stripe_keys[stripe].lock().is_empty());

        announce(
            &store,
            AnnounceInput {
                info_hash: hash,
                peer,
                is_seeding: true,
                event: AnnounceEvent::Unknown,
            },
        );

        assert!(store.stripe_keys[stripe].lock().contains(&hash));
        assert_eq!(stripe_index_len(&store), 1);

        // Second announce must not duplicate the index entry.
        announce(
            &store,
            AnnounceInput {
                info_hash: hash,
                peer,
                is_seeding: true,
                event: AnnounceEvent::Unknown,
            },
        );
        assert_eq!(stripe_index_len(&store), 1);
    }

    #[test]
    fn sweep_heals_orphan_stripe_index_entries() {
        let store = TrackerStore::new();
        let hash = [0x5bu8; 20];
        let stripe = TrackerStore::stripe_of(&hash);

        // Simulate index lag: key in stripe index but not in torrents map.
        store.index_insert(hash);
        assert!(store.stripe_keys[stripe].lock().contains(&hash));
        assert_eq!(store.torrent_count(), 0);

        // Rotate until we hit this stripe (and the rest).
        let removed = store.sweep_all_stripes_at(store.now_secs());
        assert_eq!(removed, 0);
        assert!(!store.stripe_keys[stripe].lock().contains(&hash));
        assert_eq!(stripe_index_len(&store), 0);
    }

    fn stripe_index_len(store: &TrackerStore) -> usize {
        store.stripe_keys.iter().map(|s| s.lock().len()).sum()
    }

    #[test]
    fn remove_declines_when_torrent_no_longer_stale() {
        let store = TrackerStore::new();
        let hash = [0x5cu8; 20];
        let peer = [127, 0, 0, 1, 17, 0x5c];

        announce(
            &store,
            AnnounceInput {
                info_hash: hash,
                peer,
                is_seeding: true,
                event: AnnounceEvent::Unknown,
            },
        );

        // Mirrors a sweep that decided `hash` was stale, then raced an announce
        // that refreshed it: the delete must be declined and the index kept.
        assert!(!store.remove_torrent_if_stale(&hash, store.now_secs()));
        assert_eq!(store.torrent_count(), 1);
        assert_eq!(stripe_index_len(&store), 1);
        assert!(store.peer_exists(hash, PeerPool::Seeder, peer));
    }

    #[test]
    fn refresh_reindexes_unindexed_torrents() {
        let store = TrackerStore::new();
        let hash = [0x5du8; 20];
        let peer = [127, 0, 0, 1, 17, 0x5c];

        announce(
            &store,
            AnnounceInput {
                info_hash: hash,
                peer,
                is_seeding: true,
                event: AnnounceEvent::Unknown,
            },
        );

        // Simulate a create racing a sweep delete: torrent lives on in the map
        // but its stripe entry was dropped, so no sweep would ever visit it.
        store.index_remove(&hash);
        assert_eq!(stripe_index_len(&store), 0);

        let after_ttl = store.now_secs() + TORRENT_TTL.as_secs() as u32 + 1;
        assert_eq!(store.sweep_all_stripes_at(after_ttl), 0);
        assert_eq!(store.torrent_count(), 1, "unindexed torrent leaks");

        store.refresh_peer_totals();
        assert_eq!(stripe_index_len(&store), 1);
        assert_eq!(store.stats.sweep_snapshot().index_repaired, 1);
        assert_eq!(store.stripe_index_size(), 1);

        // Healed, so the next rotation collects it.
        assert_eq!(store.sweep_all_stripes_at(after_ttl), 1);
        assert_eq!(store.torrent_count(), 0);
    }

    #[test]
    fn sweep_records_visited_and_removed() {
        let store = TrackerStore::new();
        let hash = [0x5eu8; 20];
        let peer = [127, 0, 0, 1, 17, 0x5c];

        announce(
            &store,
            AnnounceInput {
                info_hash: hash,
                peer,
                is_seeding: true,
                event: AnnounceEvent::Unknown,
            },
        );

        let after_ttl = store.now_secs() + TORRENT_TTL.as_secs() as u32 + 1;
        store.sweep_all_stripes_at(after_ttl);

        let sweep = store.stats.sweep_snapshot();
        assert_eq!(sweep.count, SWEEP_STRIPES as u64);
        // One key indexed, so exactly one stripe visits exactly one key.
        assert_eq!(sweep.visited, 1);
        assert_eq!(sweep.removed, 1);
        assert_eq!(sweep.orphans_removed, 0);
    }

    #[test]
    fn sweep_records_healed_orphans() {
        let store = TrackerStore::new();
        let hash = [0x5fu8; 20];

        store.index_insert(hash);
        store.sweep_all_stripes_at(store.now_secs());

        let sweep = store.stats.sweep_snapshot();
        assert_eq!(sweep.visited, 1);
        assert_eq!(sweep.removed, 0);
        assert_eq!(sweep.orphans_removed, 1);
    }

    #[test]
    fn peer_totals_counts_seeders_and_leechers() {
        let store = TrackerStore::new();
        let hash = [0x57u8; 20];
        let seeder = [10, 0, 0, 1, 0x11, 0x5c];
        let leecher = [10, 0, 0, 2, 0x11, 0x5c];

        store.seed_peers(hash, PeerPool::Seeder, &[seeder]);
        store.seed_peers(hash, PeerPool::Leecher, &[leecher]);
        store.refresh_peer_totals();

        let (torrents, seeders, leechers) = store.peer_totals();
        assert_eq!(torrents, 1);
        assert_eq!(seeders, 1);
        assert_eq!(leechers, 1);
        assert_eq!(seeders + leechers, 2);
    }

    #[test]
    fn inline_promotes_to_map_above_cap() {
        let store = TrackerStore::new();
        let hash = [0x58u8; 20];
        let peers: Vec<[u8; 6]> = (0..INLINE_PEER_CAP + 2)
            .map(|i| [10, 0, 0, i as u8, 0x11, 0x5c])
            .collect();

        store.seed_peers(hash, PeerPool::Seeder, &peers);
        store.refresh_peer_totals();
        let (torrents, seeders, leechers) = store.peer_totals();
        assert_eq!(torrents, 1);
        assert_eq!(seeders, INLINE_PEER_CAP + 2);
        assert_eq!(leechers, 0);
    }
}
