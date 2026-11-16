//! Starting the frontend, with the fallback: a frontend that says "the server's contract does not fit" (exit code 3) is
//! replaced by the standard frontend, with a message the person sees. Never silent (ADR 0065 section 5).
//!
//! Pattern: Strategy + Chain of responsibility — the chosen frontend first, the standard one after it; the process is the
//! unit, so a frontend that crashes does not take the program with it.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The frontend that comes with the program and is the fallback.
pub const STANDARD: &str = "simple";

/// The exit code of a frontend for "the server's contract does not fit".
pub const EXIT_CONTRACT: i32 = 3;

/// Names that are crate roles, not frontends (ADR 0094 draft, item 6).
const RESERVED: [&str; 8] = [
    "core",
    "server",
    "client",
    "app",
    "bin",
    "cli",
    "engine",
    "testserver",
];

/// Whether `id` can be a frontend id: ASCII lower case, digits and `-`, 1 to 32 characters, not a crate role.
#[must_use]
pub fn valid_frontend_id(id: &str) -> bool {
    (1..=32).contains(&id.len())
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        && !id.starts_with('-')
        && !RESERVED.contains(&id)
}

/// Where the executable of frontend `id` is: `fs-draw-<id>` next to `me` (stage 1: the same release package; the manifest in
/// `packaging/frontends/` describes it for the store).
#[must_use]
pub fn frontend_path(me: &Path, id: &str) -> PathBuf {
    me.with_file_name(format!("fs-draw-{id}{}", std::env::consts::EXE_SUFFIX))
}

/// Run `chosen`; when it cannot start or ends with [`EXIT_CONTRACT`], say so through `say` and run `standard` instead.
/// Returns the exit code to end with: the frontend's own, 1 when nothing could be started, or [`EXIT_CONTRACT`] when the
/// standard frontend does not fit either.
pub fn run_with_fallback(
    chosen: &Path,
    standard: &Path,
    args: &[OsString],
    say: &mut dyn FnMut(&str),
) -> i32 {
    match run_one(chosen, args) {
        Ok(0) => 0,
        Ok(EXIT_CONTRACT) => {
            if chosen == standard {
                say(
                    "the standard frontend does not fit the server's contract; start the program again after closing it completely, or update the program",
                );
                return EXIT_CONTRACT;
            }
            say(&format!(
                "the frontend {} does not fit the server's contract; starting the standard frontend instead",
                chosen.display()
            ));
            fall_back(standard, args, say)
        }
        Ok(other) => other,
        Err(e) => {
            if chosen == standard {
                say(&format!(
                    "the standard frontend {} cannot be started: {e}",
                    chosen.display()
                ));
                return 1;
            }
            say(&format!(
                "the frontend {} cannot be started ({e}); starting the standard frontend instead",
                chosen.display()
            ));
            fall_back(standard, args, say)
        }
    }
}

fn fall_back(standard: &Path, args: &[OsString], say: &mut dyn FnMut(&str)) -> i32 {
    match run_one(standard, args) {
        Ok(code) => code,
        Err(e) => {
            say(&format!(
                "the standard frontend {} cannot be started: {e}",
                standard.display()
            ));
            1
        }
    }
}

fn run_one(exe: &Path, args: &[OsString]) -> std::io::Result<i32> {
    let status = Command::new(exe).args(args).status()?;
    Ok(status.code().unwrap_or(1))
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn stub(dir: &Path, name: &str, body: &str) -> PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    #[test]
    fn frontend_ids_follow_the_rules() {
        assert!(valid_frontend_id("simple"));
        assert!(valid_frontend_id("paper-2"));
        for bad in [
            "",
            "Simple",
            "core",
            "server",
            "client",
            "app",
            "bin",
            "cli",
            "engine",
            "testserver",
            "a/b",
            "-x",
        ] {
            assert!(!valid_frontend_id(bad), "{bad}");
        }
    }

    #[test]
    fn the_path_is_next_to_the_starter() {
        assert_eq!(
            frontend_path(Path::new("/usr/bin/fs-draw"), "simple"),
            Path::new("/usr/bin/fs-draw-simple")
        );
    }

    #[test]
    fn a_fitting_frontend_runs_alone_and_says_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let ok = stub(dir.path(), "ok", "exit 0");
        let mut said = Vec::new();
        assert_eq!(
            run_with_fallback(&ok, &ok, &[], &mut |m| said.push(m.to_owned())),
            0
        );
        assert_eq!(said, Vec::<String>::new());
    }

    #[test]
    fn a_frontend_with_the_wrong_contract_is_replaced_by_the_standard_one_with_a_message() {
        let dir = tempfile::tempdir().unwrap();
        let wrong = stub(dir.path(), "wrong", "exit 3");
        let standard = stub(dir.path(), "standard", "echo standard > \"$0.ran\"\nexit 0");
        let mut said = Vec::new();
        let code = run_with_fallback(&wrong, &standard, &[], &mut |m| said.push(m.to_owned()));
        assert_eq!(code, 0);
        assert_eq!(said.len(), 1, "one visible message");
        assert!(said[0].contains("does not fit the server's contract"));
        assert!(
            dir.path().join("standard.ran").exists(),
            "the standard frontend ran"
        );
    }

    #[test]
    fn a_frontend_that_is_missing_is_replaced_too() {
        let dir = tempfile::tempdir().unwrap();
        let standard = stub(dir.path(), "standard", "exit 0");
        let mut said = Vec::new();
        let code = run_with_fallback(&dir.path().join("none"), &standard, &[], &mut |m| {
            said.push(m.to_owned());
        });
        assert_eq!(code, 0);
        assert!(said[0].contains("cannot be started"));
    }

    #[test]
    fn the_standard_frontend_with_the_wrong_contract_ends_with_3_and_a_message() {
        let dir = tempfile::tempdir().unwrap();
        let standard = stub(dir.path(), "standard", "exit 3");
        let mut said = Vec::new();
        assert_eq!(
            run_with_fallback(&standard, &standard, &[], &mut |m| said.push(m.to_owned())),
            3
        );
        assert_eq!(said.len(), 1);
    }

    #[test]
    fn another_exit_code_is_the_frontends_own_and_no_fallback_runs() {
        let dir = tempfile::tempdir().unwrap();
        let broken = stub(dir.path(), "broken", "exit 7");
        let standard = stub(dir.path(), "standard", "touch \"$0.ran\"");
        let mut said = Vec::new();
        assert_eq!(
            run_with_fallback(&broken, &standard, &[], &mut |m| said.push(m.to_owned())),
            7
        );
        assert_eq!(said, Vec::<String>::new());
        assert!(!dir.path().join("standard.ran").exists());
    }
}
