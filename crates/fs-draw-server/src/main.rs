//! `fs-draw-server`. Usage: `fs-draw-server [--socket PATH] [--data DIR] [--actor IRI] [--instance DOMAIN] [--lease FILE]`.
//! The same server is `fs-draw serve`.
//!
//! Pattern: none (thin binary) — hands the words to `serve`.

use std::process::ExitCode;

use fs_draw_server::{ServeError, serve};

fn main() -> ExitCode {
    match serve(std::env::args_os().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(ServeError::Usage(m)) => {
            eprintln!("fs-draw-server: {m}");
            ExitCode::from(2)
        }
        Err(ServeError::Failed(m)) => {
            eprintln!("fs-draw-server: {m}");
            ExitCode::FAILURE
        }
    }
}
