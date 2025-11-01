use std::collections::HashMap;
use std::env::{VarError, var};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

const STORAGE_PATH: &str = ".";
static WILDCARD: LazyLock<String> = LazyLock::new(|| get_wildcard());

static MAPPINGS: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(init_mappings()));

fn get_wildcard() -> String {
    match var("WILDCARD") {
        Ok(var) => var,
        Err(err) => {
            if err == VarError::NotPresent {
                "wildcard".into()
            } else {
                println!("{err:?}");
                panic!("Invalid unicode in WILDCARD env var.");
            }
        }
    }
}

pub fn init_mappings() -> HashMap<String, String> {
    let mut mappings = HashMap::new();
    mappings.insert("localhost".into(), "test_project".into());
    mappings
}

/// Turns a domain into a storage accessible path.
pub fn resolve_domain(host: &str) -> PathBuf {
    let mappings = MAPPINGS.lock().unwrap();
    let mapping = mappings.get(host).unwrap_or(&WILDCARD);

    Path::new(STORAGE_PATH).join(mapping)
}

/// Turn a path into a Path
pub fn resolve_path(base: &Path, path: &str) -> PathBuf {}
