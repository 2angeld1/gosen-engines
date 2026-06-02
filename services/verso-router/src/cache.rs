use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::sync::Mutex;

struct Entry {
    result: String,
}

struct Cache {
    inner: HashMap<String, Entry>,
    max_size: usize,
}

impl Cache {
    fn new(max_size: usize) -> Self {
        Cache {
            inner: HashMap::new(),
            max_size,
        }
    }

    fn get(&self, key: &str) -> Option<&str> {
        self.inner.get(key).map(|e| e.result.as_str())
    }

    fn set(&mut self, key: String, value: String) {
        if self.inner.len() >= self.max_size {
            // Remove first entry
            if let Some(key) = self.inner.keys().next().cloned() {
                self.inner.remove(&key);
            }
        }
        self.inner.insert(key, Entry { result: value });
    }
}

static CACHE: once_cell::sync::Lazy<Mutex<Cache>> = once_cell::sync::Lazy::new(|| {
    Mutex::new(Cache::new(500))
});

fn make_key(source: &str, source_lang: &str, target_lang: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source.as_bytes());
    hasher.update(source_lang.as_bytes());
    hasher.update(target_lang.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn get(source: &str, source_lang: &str, target_lang: &str) -> Option<String> {
    let key = make_key(source, source_lang, target_lang);
    let cache = CACHE.lock().unwrap();
    let result = cache.get(&key).map(|s| s.to_string());
    if result.is_some() {
        tracing::debug!("cache HIT  key={}", key);
    } else {
        tracing::debug!("cache MISS key={}", key);
    }
    result
}

pub fn set(source: &str, source_lang: &str, target_lang: &str, result: &str) {
    let key = make_key(source, source_lang, target_lang);
    let mut cache = CACHE.lock().unwrap();
    cache.set(key.clone(), result.to_string());
    tracing::debug!("cache SET  key={}", key);
}
