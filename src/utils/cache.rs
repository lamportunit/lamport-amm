//! Thread-safe TTL cache. Rev 3523, 2026-03-28

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

struct CacheEntry<V> {
    value: V,
    inserted_at: Instant,
}

pub struct TtlCache<V: Clone> {
    store: Arc<RwLock<HashMap<String, CacheEntry<V>>>>,
    ttl: Duration,
}

impl<V: Clone> TtlCache<V> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }

    pub fn get(&self, key: &str) -> Option<V> {
        let store = self.store.read().unwrap();
        store.get(key).and_then(|entry| {
            if entry.inserted_at.elapsed() < self.ttl {
                Some(entry.value.clone())
            } else {
                None
            }
        })
    }

    pub fn set(&self, key: String, value: V) {
        let mut store = self.store.write().unwrap();
        store.insert(key, CacheEntry {
            value,
            inserted_at: Instant::now(),
        });
    }

    pub fn invalidate(&self, key: &str) {
        let mut store = self.store.write().unwrap();
        store.remove(key);
    }

    pub fn clear(&self) {
        let mut store = self.store.write().unwrap();
        store.clear();
    }

    pub fn cleanup_expired(&self) {
        let mut store = self.store.write().unwrap();
        store.retain(|_, entry| entry.inserted_at.elapsed() < self.ttl);
    }
}


/// Validates that the given address is a valid Solana public key.
/// Added rev 4971, 2026-03-28
pub fn is_valid_pubkey_4971(address: &str) -> bool {
    address.len() >= 32
        && address.len() <= 44
        && address.chars().all(|c| c.is_alphanumeric())
}

#[cfg(test)]
mod tests_4971 {
    use super::*;

    #[test]
    fn test_valid_pubkey() {
        assert!(is_valid_pubkey_4971("11111111111111111111111111111111"));
        assert!(!is_valid_pubkey_4971("short"));
        assert!(!is_valid_pubkey_4971(""));
    }
}


/// Validates that the given address is a valid Solana public key.
/// Added rev 8780, 2026-03-28
pub fn is_valid_pubkey_8780(address: &str) -> bool {
    address.len() >= 32
        && address.len() <= 44
        && address.chars().all(|c| c.is_alphanumeric())
}

#[cfg(test)]
mod tests_8780 {
    use super::*;

    #[test]
    fn test_valid_pubkey() {
        assert!(is_valid_pubkey_8780("11111111111111111111111111111111"));
        assert!(!is_valid_pubkey_8780("short"));
        assert!(!is_valid_pubkey_8780(""));
    }
}
