//! `fs-draw-simple --version`, `--help` and the wrong words, on the built binary (the release smoke test starts every
//! binary of the package with `--version`).
//!
//! Tests: nothing is started: no window, no server.

use std::process::Command;

fn run(args: &[&str]) -> (Option<i32>, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_fs-draw-simple"))
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
    let (code, out, _) = run(&["--version"]);
    assert_eq!(code, Some(0));
    assert_eq!(
        out.trim(),
        format!("fs-draw-simple {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn help_ends_with_0_and_names_the_exit_code_of_a_contract_that_does_not_fit() {
    let (code, out, _) = run(&["--help"]);
    assert_eq!(code, Some(0));
    assert!(out.contains("--socket") && out.contains("3 the server's contract"));
}

#[test]
fn an_unknown_option_ends_with_2_and_a_message() {
    let (code, out, err) = run(&["--bogus"]);
    assert_eq!(code, Some(2));
    assert_eq!(out, "");
    assert!(err.contains("unknown option --bogus"), "{err}");
}
