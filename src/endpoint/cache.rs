use dashmap::DashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use crate::CompressionMethod;
use crate::types::Page;

const TTL: Duration = Duration::from_secs(10);

static CACHE: LazyLock<DashMap<(String, String, CompressionMethod), Page>> =
    LazyLock::new(|| DashMap::new());
static CACHE_TTL: LazyLock<Mutex<Vec<(String, String, CompressionMethod, Instant)>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

pub fn get_cache_entry(host: String, path: String, compression: CompressionMethod) -> Option<Page> {
    try_collect_garbage();

    let cache_ref = (host, path, compression);
    let cache_res = CACHE.get(&cache_ref);

    match cache_res {
        Some(cache_entry) => Some(cache_entry.clone()),
        None => None,
    }
}

pub fn insert_cache_entry(host: String, path: String, compression: CompressionMethod, page: Page) {
    let cache_entry = page;

    try_collect_garbage();

    CACHE.insert((host.clone(), path.clone(), compression), cache_entry);
    let mut cache_ttl = CACHE_TTL.lock().unwrap();
    cache_ttl.push((host, path, compression, Instant::now()));
}

fn try_collect_garbage() {
    let mut cache_ttl = CACHE_TTL.lock().unwrap();
    if let Some(entry) = cache_ttl.first() {
        if entry.3.elapsed() > TTL {
            CACHE.remove(&(entry.0.clone(), entry.1.clone(), entry.2.clone()));
            cache_ttl.remove(0);
        }
    }
}
