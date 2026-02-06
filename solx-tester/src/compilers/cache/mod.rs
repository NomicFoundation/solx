//!
//! The thread-safe cache implementation.
//!

pub mod value;

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::RwLock;

use solx_utils::SyncLock;
use solx_utils::SyncRwLock;

use self::value::Value;

///
/// The thread-safe cache implementation.
///
pub struct Cache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    /// The cache inner data structure.
    inner: RwLock<HashMap<K, Value<anyhow::Result<V>>>>,
}

impl<K, V> Cache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    ///
    /// Creates an empty cache instance.
    ///
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }

    ///
    /// Evaluates and saves a cache value.
    ///
    /// If the value is already being evaluated, does nothing.
    ///
    pub fn evaluate<F>(&self, key: K, f: F)
    where
        F: FnOnce() -> anyhow::Result<V>,
    {
        let waiter = Value::<V>::waiter();
        let _lock = waiter.lock_sync();
        {
            let mut inner = self.inner.write_sync();

            if inner.contains_key(&key) {
                return;
            }

            inner.insert(key.clone(), Value::Waiter(waiter.clone()));
        }

        let value = f();

        let mut inner = self.inner.write_sync();
        let entry_value = inner
            .get_mut(&key)
            .expect("The value is not being evaluated");

        assert!(
            matches!(entry_value, Value::Waiter(_)),
            "The value is already evaluated"
        );

        *entry_value = Value::Value(value);
    }

    ///
    /// Checks if the value for the key is present in the cache.
    ///
    pub fn contains(&self, key: &K) -> bool {
        self.inner.read_sync().contains_key(key)
    }

    ///
    /// Get a cloned value by the key.
    /// Will wait if the value is being evaluated.
    ///
    /// # Panics
    ///
    /// If the value is not being evaluated.
    ///
    pub fn get_cloned(&self, key: &K) -> anyhow::Result<V> {
        self.wait(key);
        self.inner
            .read()
            .expect("Sync")
            .get(key)
            .expect("The value is not being evaluated")
            .unwrap_value()
            .as_ref()
            .map(|value| value.to_owned())
            .map_err(|error| anyhow::anyhow!("{error}"))
    }

    ///
    /// Waits until value will be evaluated if needed.
    ///
    /// # Panics
    ///
    /// If the value is not being evaluated.
    ///
    fn wait(&self, key: &K) {
        let waiter = if let Value::Waiter(waiter) = self
            .inner
            .read()
            .expect("Sync")
            .get(key)
            .expect("The value is not being evaluated")
        {
            waiter.clone()
        } else {
            return;
        };
        let _lock = waiter.lock_sync();
    }
}
