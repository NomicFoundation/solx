//!
//! Extension traits for infallible lock acquisition.
//!

use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::RwLock;
use std::sync::RwLockReadGuard;
use std::sync::RwLockWriteGuard;

///
/// Extension trait for `Mutex::lock()` when poisoning cannot occur.
///
/// All mutexes in this codebase are held only during short, non-panicking
/// operations, so poisoning is impossible. This trait documents that invariant
/// in a single place instead of repeating `.lock().expect("Sync")`.
///
pub trait SyncLock<T> {
    ///
    /// Locks the mutex, panicking only if the mutex is poisoned.
    ///
    fn lock_sync(&self) -> MutexGuard<'_, T>;
}

impl<T> SyncLock<T> for Mutex<T> {
    fn lock_sync(&self) -> MutexGuard<'_, T> {
        self.lock().expect("Sync")
    }
}

///
/// Extension trait for `RwLock` when poisoning cannot occur.
///
pub trait SyncRwLock<T> {
    ///
    /// Acquires a read lock, panicking only if the lock is poisoned.
    ///
    fn read_sync(&self) -> RwLockReadGuard<'_, T>;

    ///
    /// Acquires a write lock, panicking only if the lock is poisoned.
    ///
    fn write_sync(&self) -> RwLockWriteGuard<'_, T>;
}

impl<T> SyncRwLock<T> for RwLock<T> {
    fn read_sync(&self) -> RwLockReadGuard<'_, T> {
        self.read().expect("Sync")
    }

    fn write_sync(&self) -> RwLockWriteGuard<'_, T> {
        self.write().expect("Sync")
    }
}
