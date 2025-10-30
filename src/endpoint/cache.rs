use dashmap::DashMap;
use std::sync::LazyLock;
use std::time::{Duration, Instant};

use crate::grpc::request::Response;

const TTL: Duration = Duration::from_secs(60);

static CACHE: LazyLock<DashMap<(String, String), CacheEntry>> = LazyLock::new(|| DashMap::new());

struct CacheEntry {
    response: Response,
    ttl: Instant,
}

pub fn get_cache_entry(host: String, path: String) -> Option<Response> {
    let cache_ref = (host, path);
    let cache_res = CACHE.get(&cache_ref);

    match cache_res {
        Some(cache_entry) => {
            if cache_entry.ttl.elapsed() > TTL {
                // Clean the cache. It's TTL has elapsed.
                let _ = CACHE.remove(&cache_ref);
            }
            Some(cache_entry.response.clone())
        }
        None => None,
    }
}

pub fn insert_cache_entry(host: String, path: String, response: Response) {
    let cache_entry = CacheEntry {
        response,
        ttl: Instant::now(),
    };

    CACHE.insert((host, path), cache_entry);
}
