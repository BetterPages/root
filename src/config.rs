use std::env::var;
use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;

/// Default values are inlined.

/// The global 404 page, used when a repository has no local 404 page.
pub static GLOBAL_404: LazyLock<Vec<u8>> = LazyLock::new(|| get_global_404());
/// The wildcard path to use for unknown domains.
pub static WILDCARD_DOMAIN: LazyLock<String> = LazyLock::new(|| get_wildcard_domain());
/// The path to where the storage should be used
pub static STORAGE_PATH: LazyLock<PathBuf> = LazyLock::new(|| get_storage_path());

fn get_global_404() -> Vec<u8> {
    if let Ok(path) = var("GLOBAL_404") {
        fs::read(path).unwrap()
    } else {
        b"<!DOCTYPE html><html><head><title>404</title></head><body><h1>404</h1><p>We couldn't find what you were looking for &colon;&lpar;</p></body></html>".into()
    }
}

fn get_wildcard_domain() -> String {
    if let Ok(domain) = var("WILDCARD_DOMAIN") {
        domain
    } else {
        "wildcard".into()
    }
}

fn get_storage_path() -> PathBuf {
    let path = if let Ok(path) = var("STORAGE_PATH") {
        PathBuf::from(path)
    } else {
        PathBuf::from(".")
    };

    if !path.exists() {
        panic!(
            "STORAGE_PATH is invalid. It's currently set to {}, but we can't access that path.",
            path.display()
        );
    };

    path
}
