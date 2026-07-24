mod stats;

use std::collections::HashMap;
use std::time::{Duration, Instant};

use ahash::RandomState;
use dashmap::DashMap;

use crate::query;

pub use stats::TrackerStats;

pub type InfoHash = [u8; 20];
pub type PeerId = [u8; 6];

pub const PEER_TTL: Duration = Duration::from_secs(60 * 31);
pub const TORRENT_TTL: Duration = Duration::from_secs(60 * 31);
pub const CACHE_TTL: Duration = Duration::from_secs(60 * 30);
pub const PEER_SAMPLE_LIMIT: usize = 50;

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

struct CachedReply {
    body: Vec<u8>,
    expires_at: Instant,
}

struct Torrent {
    seeders: HashMap<PeerId, Instant>,
    leechers: HashMap<PeerId, Instant>,
    cache: Option<CachedReply>,
    last_activity: Instant,
}

impl Torrent {
    fn new(now: Instant) -> Self {
        Self {
            seeders: HashMap::new(),
            leechers: HashMap::new(),
            cache: None,
            last_activity: now,
        }
    }

    fn touch(&mut self, now: Instant) {
        self.last_activity = now;
    }

    fn is_stale(&self, now: Instant) -> bool {
        now > self.last_activity + TORRENT_TTL
    }

    fn purge_expired(&mut self, now: Instant) {
        self.seeders.retain(|_, exp| *exp > now);
        self.leechers.retain(|_, exp| *exp > now);
        if let Some(cache) = &self.cache {
            if cache.expires_at <= now {
                self.cache = None;
            }
        }
    }

    fn peer_expiry(now: Instant) -> Instant {
        now + PEER_TTL
    }

    fn sample_peers(map: &HashMap<PeerId, Instant>, now: Instant, limit: usize) -> Vec<[u8; 6]> {
        map.iter()
            .filter(|(_, exp)| **exp > now)
            .take(limit)
            .map(|(peer, _)| *peer)
            .collect()
    }
}

pub struct TrackerStore {
    torrents: DashMap<InfoHash, Torrent, RandomState>,
    pub stats: TrackerStats,
}

impl TrackerStore {
    pub fn new() -> Self {
        Self {
            torrents: DashMap::with_hasher(RandomState::new()),
            stats: TrackerStats::default(),
        }
    }

    pub fn handle_announce(&self, input: AnnounceInput, req_duration_ms: i64) -> Vec<u8> {
        let now = Instant::now();

        self.torrents
            .entry(input.info_hash)
            .and_modify(|torrent| {
                if torrent.is_stale(now) {
                    *torrent = Torrent::new(now);
                }
            })
            .or_insert_with(|| Torrent::new(now));

        let mut torrent_ref = self
            .torrents
            .get_mut(&input.info_hash)
            .expect("torrent entry was just created");

        let torrent = torrent_ref.value_mut();
        torrent.touch(now);
        torrent.purge_expired(now);

        let is_seeder = torrent.seeders.contains_key(&input.peer);
        let is_leecher = torrent.leechers.contains_key(&input.peer);

        let cached_reply = torrent
            .cache
            .as_ref()
            .filter(|c| c.expires_at > now)
            .map(|c| c.body.clone());

        let (seed_count_mod, leech_count_mod) =
            Self::count_mods(input.event, input.is_seeding, is_seeder, is_leecher);

        let final_res = match cached_reply {
            None => {
                let seeders = Torrent::sample_peers(&torrent.seeders, now, PEER_SAMPLE_LIMIT);
                let leechers = Torrent::sample_peers(&torrent.leechers, now, PEER_SAMPLE_LIMIT);
                query::announce_reply(
                    torrent.seeders.len() as i64 + seed_count_mod,
                    torrent.leechers.len() as i64 + leech_count_mod,
                    &seeders,
                    &leechers,
                )
            }
            Some(body) => {
                self.stats.record_cache_hit();
                body
            }
        };

        Self::apply_announce(torrent, &input, now, is_seeder, is_leecher);

        if seed_count_mod != 0 || leech_count_mod != 0 {
            torrent.cache = None;
        } else {
            self.stats.record_nochange();
            torrent.cache = Some(CachedReply {
                body: final_res.clone(),
                expires_at: now + CACHE_TTL,
            });
        }

        drop(torrent_ref);
        self.maybe_sweep_torrent(input.info_hash, now);
        self.stats.record_announce(req_duration_ms);
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
        now: Instant,
        is_seeder: bool,
        is_leecher: bool,
    ) {
        let expiry = Torrent::peer_expiry(now);

        if input.event == AnnounceEvent::Stopped {
            if is_seeder {
                torrent.seeders.remove(&input.peer);
            } else if is_leecher {
                torrent.leechers.remove(&input.peer);
            }
            return;
        }

        if input.is_seeding {
            torrent.seeders.insert(input.peer, expiry);

            if input.event == AnnounceEvent::Completed && is_leecher {
                torrent.leechers.remove(&input.peer);
            }
        } else {
            torrent.leechers.insert(input.peer, expiry);
        }
    }

    pub fn peer_exists(&self, info_hash: InfoHash, pool: PeerPool, peer: PeerId) -> bool {
        let now = Instant::now();
        let Some(torrent_ref) = self.torrents.get(&info_hash) else {
            return false;
        };

        if torrent_ref.is_stale(now) {
            return false;
        }

        let map = match pool {
            PeerPool::Seeder => &torrent_ref.seeders,
            PeerPool::Leecher => &torrent_ref.leechers,
        };

        map.get(&peer)
            .map(|exp| *exp > now)
            .unwrap_or(false)
    }

    pub fn seed_peers(&self, info_hash: InfoHash, pool: PeerPool, peers: &[[u8; 6]]) {
        let now = Instant::now();
        let expiry = Torrent::peer_expiry(now);

        self.torrents
            .entry(info_hash)
            .and_modify(|torrent| {
                if torrent.is_stale(now) {
                    *torrent = Torrent::new(now);
                }
            })
            .or_insert_with(|| Torrent::new(now));

        let mut torrent_ref = self
            .torrents
            .get_mut(&info_hash)
            .expect("torrent entry was just created");

        let torrent = torrent_ref.value_mut();
        torrent.touch(now);
        torrent.cache = None;

        let map = match pool {
            PeerPool::Seeder => &mut torrent.seeders,
            PeerPool::Leecher => &mut torrent.leechers,
        };

        for peer in peers {
            map.insert(*peer, expiry);
        }
    }

    pub fn torrent_count(&self) -> usize {
        self.torrents.len()
    }

    fn maybe_sweep_torrent(&self, info_hash: InfoHash, now: Instant) {
        if let Some(torrent_ref) = self.torrents.get(&info_hash) {
            if torrent_ref.is_stale(now) {
                drop(torrent_ref);
                self.torrents.remove(&info_hash);
            }
        }
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

    #[test]
    fn seeder_announce_registers_peer() {
        let store = TrackerStore::new();
        let hash = [0x41u8; 20];
        let peer = [127, 0, 0, 1, 17, 0x5c];

        store.handle_announce(
            AnnounceInput {
                info_hash: hash,
                peer,
                is_seeding: true,
                event: AnnounceEvent::Unknown,
            },
            0,
        );

        assert!(store.peer_exists(hash, PeerPool::Seeder, peer));
    }

    #[test]
    fn stopped_removes_peer() {
        let store = TrackerStore::new();
        let hash = [0x42u8; 20];
        let peer = [127, 0, 0, 1, 17, 0x5c];

        store.handle_announce(
            AnnounceInput {
                info_hash: hash,
                peer,
                is_seeding: true,
                event: AnnounceEvent::Unknown,
            },
            0,
        );
        store.handle_announce(
            AnnounceInput {
                info_hash: hash,
                peer,
                is_seeding: true,
                event: AnnounceEvent::Stopped,
            },
            0,
        );

        assert!(!store.peer_exists(hash, PeerPool::Seeder, peer));
    }

    #[test]
    fn completed_moves_leecher_to_seeder() {
        let store = TrackerStore::new();
        let hash = [0x43u8; 20];
        let peer = [127, 0, 0, 1, 17, 0x5c];

        store.handle_announce(
            AnnounceInput {
                info_hash: hash,
                peer,
                is_seeding: false,
                event: AnnounceEvent::Unknown,
            },
            0,
        );
        store.handle_announce(
            AnnounceInput {
                info_hash: hash,
                peer,
                is_seeding: true,
                event: AnnounceEvent::Completed,
            },
            0,
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
        let body = store.handle_announce(
            AnnounceInput {
                info_hash: hash,
                peer: caller,
                is_seeding: false,
                event: AnnounceEvent::Unknown,
            },
            0,
        );

        assert!(body.windows(6).any(|w| w == peer));
    }
}
