//! `fs-draw --version`, `--help` and the wrong words, on the built binary. The release smoke test calls
//! `/usr/bin/fs-draw --version` (fs-people once failed on `unknown option --version`).
//!
//! Tests: nothing is started: no window, no server (a start would need a socket folder, which these runs never reach).

use std::process::Command;

fn run(args: &[&str]) -> (Option<i32>, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_fs-draw"))
        .args(args)
        .output()
        .unwrap();
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn version_prints_the_name_and_the_crate_version_and_ends_with_0() {
    for flag in ["--version", "-V"] {
        let (code, out, _) = run(&[flag]);
        assert_eq!(code, Some(0));
        assert_eq!(out.trim(), format!("fs-draw {}", env!("CARGO_PKG_VERSION")));
    }
}

#[test]
fn help_ends_with_0_and_names_the_options() {
    let (code, out, _) = run(&["--help"]);
    assert_eq!(code, Some(0));
    for word in ["--frontend", "--socket", "serve", "--version"] {
        assert!(out.contains(word), "{word}");
    }
}

#[test]
fn an_unknown_option_ends_with_2_and_a_message_before_anything_starts() {
    let (code, out, err) = run(&["--bogus"]);
    assert_eq!(code, Some(2));
    assert!(out.is_empty());
    assert!(err.contains("unknown option --bogus"), "{err}");
}

#[test]
fn a_frontend_id_that_is_a_crate_role_is_refused_with_2() {
    let (code, _, err) = run(&["--frontend", "server"]);
    assert_eq!(code, Some(2));
    assert!(err.contains("not a frontend id"), "{err}");
}

#[test]
fn serve_with_help_prints_the_server_options_and_starts_nothing() {
    let (code, out, _) = run(&["serve", "--help"]);
    assert_eq!(code, Some(0));
    assert!(out.contains("--lease"));
}

#[test]
fn serve_with_an_unknown_option_ends_with_2() {
    let (code, _, err) = run(&["serve", "--bogus"]);
    assert_eq!(code, Some(2));
    assert!(err.contains("unknown argument"), "{err}");
}
