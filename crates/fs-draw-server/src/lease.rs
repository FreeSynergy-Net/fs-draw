//! A server that a frontend started ends when no frontend is left (the lease).
//!
//! Pattern: Guard + Observer thread — a frontend holds a *shared* lock on `<socket>.lease` as long as it lives
//! (`fs_draw_client::hold_lease`; the system lets go when the process ends, also on a crash). The server it started
//! tries the *exclusive* lock from time to time and ends itself, in order, when it gets it a few times in a row.
//! Several frontends share one server; it ends with the last. A server started by hand never ends by itself.

use std::ffi::OsString;
use std::fs::{OpenOptions, TryLockError};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// How often the server looks, unless `FS_LEASE_POLL_MS` says otherwise (tests).
const POLL: Duration = Duration::from_secs(2);
/// How many looks in a row must find nobody.
const UNUSED_LOOKS: u32 = 3;
/// A new server is not judged before this has passed (a frontend that starts it connects within it).
const YOUNG: Duration = Duration::from_secs(10);

/// Take `--lease PATH` out of `args`; the rest goes on to the normal options.
///
/// # Errors
/// `--lease` without a path.
pub fn take_lease(args: Vec<OsString>) -> Result<(Option<PathBuf>, Vec<OsString>), String> {
    let mut rest = Vec::with_capacity(args.len());
    let mut lease = None;
    let mut it = args.into_iter();
    while let Some(word) = it.next() {
        if word == "--lease" {
            lease = Some(PathBuf::from(it.next().ok_or("--lease needs a path")?));
        } else {
            rest.push(word);
        }
    }
    Ok((lease, rest))
}

/// Whether nobody holds the lease right now. A lease whose folder is gone counts as unused: a server must not outlive it.
#[must_use]
pub fn unused(path: &Path) -> bool {
    let opened = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path);
    let file = match opened {
        Ok(file) => file,
        Err(e) => return e.kind() == std::io::ErrorKind::NotFound,
    };
    match file.try_lock() {
        Ok(()) => {
            let _ = file.unlock();
            true
        }
        Err(TryLockError::WouldBlock | TryLockError::Error(_)) => false,
    }
}

/// Watch `path` on a thread of its own; when the lease is unused, `end` is called once.
pub fn watch(path: PathBuf, end: impl FnOnce() + Send + 'static) {
    let poll = std::env::var("FS_LEASE_POLL_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .map_or(POLL, Duration::from_millis);
    let young = if poll < POLL { Duration::ZERO } else { YOUNG };
    let started = Instant::now();
    std::thread::spawn(move || {
        let mut looks = 0;
        loop {
            std::thread::sleep(poll);
            if started.elapsed() < young {
                continue;
            }
            if unused(&path) {
                looks += 1;
                if looks >= UNUSED_LOOKS {
                    end();
                    return;
                }
            } else {
                looks = 0;
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_lease_option_is_taken_out_and_needs_a_path() {
        let words: Vec<OsString> = ["--lease", "/x", "--socket", "s"].map(Into::into).to_vec();
        let (lease, rest) = take_lease(words).unwrap();
        assert_eq!(lease, Some(PathBuf::from("/x")));
        assert_eq!(rest, [OsString::from("--socket"), OsString::from("s")]);
        assert!(take_lease(vec!["--lease".into()]).is_err());
    }

    #[test]
    fn a_lease_whose_folder_is_gone_is_unused() {
        assert!(unused(Path::new("/nonexistent-fs-draw-folder/x.lease")));
    }
}
