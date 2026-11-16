//! `fs-draw-simple`: the frontend `simple` as a process. People start `fs-draw`, not this one (ADR 0094 draft).
//!
//! Pattern: none (thin binary) — hands the words to `run`.

use std::process::ExitCode;

fn main() -> ExitCode {
    let words: Vec<String> = std::env::args().skip(1).collect();
    ExitCode::from(fs_draw_simple::run_code(&words))
}
