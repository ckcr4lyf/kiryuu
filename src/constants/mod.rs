pub const ANNOUNCE_COUNT_KEY: &str = "kiryuu_http_announce_count";
pub const NOCHANGE_ANNOUNCE_COUNT_KEY: &str = "kiryuu_http_nochange_announce_count"; // If no change to seeder_count / leecher_count
pub const CACHE_HIT_ANNOUNCE_COUNT_KEY: &str = "kiryuu_http_cache_hit_announce_count";
pub const REQ_DURATION_KEY: &str = "kiryuu_http_req_seconds_sum";
pub const TORRENTS_KEY: &str = "TORRENTS";

pub const SEEDER_SUFFIX: &[u8; 8] = b"_seeders";
pub const LEECHER_SUFFIX: &[u8; 9] = b"_leechers";
pub const CACHE_SUFFIX: &[u8; 6] = b"_cache";