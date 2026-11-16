//! The lease: a frontend holds a shared lock on `<socket>.lease` as long as it lives, so a server it started does not end
//! under it (the server side is `fs_draw_server::lease`).
//!
//! Pattern: RAII / Guard — the lock lives as long as the `File` that [`hold_lease`] returns; the system lets go when the
//! process ends, also on a crash.

use std::fs::{File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use fs_interfaces::rpc::companion_path;

/// The lease file that belongs to `socket`.
#[must_use]
pub fn lease_path(socket: &Path) -> PathBuf {
    companion_path(socket, "lease")
}

/// Take the shared lock on `path` and keep it as long as the returned file lives.
///
/// # Errors
/// The folder or the file cannot be made or locked.
pub fn hold_lease(path: &Path) -> io::Result<File> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    // Read and write, not append: Windows takes a lock only on a handle that can read or write data.
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)?;
    file.lock_shared()?;
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_lease_file_sits_next_to_the_socket() {
        let lease = lease_path(Path::new("/run/user/1/freesynergy/draw.sock"));
        assert_eq!(lease.parent(), Some(Path::new("/run/user/1/freesynergy")));
        assert!(lease.to_string_lossy().ends_with(".lease"));
    }

    #[test]
    fn a_held_lease_blocks_the_exclusive_lock_until_it_is_dropped() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a").join("x.lease");
        let held = hold_lease(&path).unwrap();
        let other = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();
        assert!(other.try_lock().is_err(), "held");
        drop(held);
        assert!(other.try_lock().is_ok(), "free");
    }
}
