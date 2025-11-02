use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

use crate::config::{STORAGE_PATH, WILDCARD_DOMAIN};

static MAPPINGS: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(init_mappings()));

pub fn init_mappings() -> HashMap<String, String> {
    let mut mappings = HashMap::new();
    mappings.insert("localhost".into(), "out".into());
    mappings
}

/// Turns a domain into a storage accessible path.
pub fn resolve_domain(host: &str) -> PathBuf {
    let mappings = MAPPINGS.lock().unwrap();
    let mapping = mappings.get(host).unwrap_or(&WILDCARD_DOMAIN);

    STORAGE_PATH.join(mapping)
}

/// Turn a path into a Path
pub fn resolve_path(base: &Path, path: &str) -> Option<PathBuf> {
    // TODO: This could probably be done cleaner. How? Thats a good question.
    let end_path = base.join(".".to_string() + path);
    if end_path.exists() {
        if end_path.is_dir() {
            Some(end_path.join("index.html"))
        } else {
            Some(end_path)
        }
    } else {
        if path.ends_with(".html") {
            if base.join("404.html").exists() {
                // Global 404
                Some(base.join("404.html"))
            } else {
                None
            }
        } else {
            resolve_path(base, &(path.to_string() + ".html"))
        }
    }
}
