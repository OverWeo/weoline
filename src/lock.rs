use std::fs::{self, File};

use fd_lock::RwLock;

/// Attempts to acquire an exclusive file lock on `~/.claude/.usage-lock`.
/// Returns `Some(guard)` if acquired, `None` if already held by another process.
/// The lock is released when the guard is dropped or the process exits.
pub fn try_acquire() -> Option<fd_lock::RwLockWriteGuard<'static, File>> {
    let lock_path = dirs::home_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join(".claude")
        .join(".usage-lock");

    fs::create_dir_all(lock_path.parent()?).ok()?;

    let file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)
        .ok()?;

    // Leak to get 'static lifetime — this is called at most once per process
    let lock = Box::leak(Box::new(RwLock::new(file)));
    lock.try_write().ok()
}
