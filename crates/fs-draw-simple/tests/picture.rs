//! A picture of the window, drawn without a window and without a server (`--picture`): the same shell and surfaces that the
//! window uses. Where the machine has no graphics adapter the test says so and ends green, as the render tests of
//! fs-components do.
//!
//! Tests: Pixel tests; the picture goes to a temp folder and is read back.

use std::process::Command;

fn picture(lang: &str, path: &std::path::Path) -> Option<(u32, u32, Vec<u8>)> {
    let out = Command::new(env!("CARGO_BIN_EXE_fs-draw-simple"))
        .args(["--lang", lang, "--picture"])
        .arg(path)
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&out.stderr).into_owned();
    if out.status.code() != Some(0) {
        assert!(
            err.to_lowercase().contains("graphics") || err.to_lowercase().contains("adapter"),
            "failed for another reason: {err}"
        );
        eprintln!("skipped: no graphics adapter ({err})");
        return None;
    }
    let decoder = png::Decoder::new(std::io::BufReader::new(std::fs::File::open(path).unwrap()));
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut buf).unwrap();
    Some((info.width, info.height, buf))
}

#[test]
fn the_window_is_drawn_at_the_size_asked_and_the_two_languages_differ() {
    let dir = tempfile::tempdir().unwrap();
    let Some((w, h, en)) = picture("en", &dir.path().join("en.png")) else {
        return;
    };
    assert_eq!((w, h), (1000, 640));
    assert!(en.iter().any(|b| *b != en[0]), "not one flat colour");
    let (_, _, de) = picture("de", &dir.path().join("de.png")).unwrap();
    assert_ne!(en, de, "German words are drawn, not the English ones twice");
}
