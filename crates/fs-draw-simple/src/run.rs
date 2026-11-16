//! The frontend's run: version and help, the handshake with the server (the contract must fit, else exit 3 with a message),
//! then the window or, with `--picture`, a picture of it.
//!
//! Exit codes are plain numbers so tests can read them.
//!
//! Pattern: Facade — one entry point over the arguments, the probe of the server and the two ways to show the window.

use fs_components_app::{WindowOptions, launch};
use fs_draw_client::{CONTRACT, ConnectError, default_socket, hold_lease, lease_path, probe};

use crate::args::{HELP, parse};
use crate::window::{Connection, app};
use crate::words::env_is_german;

/// The exit code for "the server's contract does not fit this frontend": the starter reads it and starts the standard
/// frontend with a visible message (ADR 0094 draft, item 4).
pub const EXIT_CONTRACT: u8 = 3;

/// Run with the words after the program name; the result is the exit code.
#[must_use]
pub fn run_code(words: &[String]) -> u8 {
    let args = match parse(words) {
        Ok(args) => args,
        Err(m) => {
            eprintln!("fs-draw-simple: {m}");
            return 2;
        }
    };
    if args.version {
        println!("fs-draw-simple {}", env!("CARGO_PKG_VERSION"));
        return 0;
    }
    if args.help {
        println!("{HELP}");
        return 0;
    }
    let german = args.german.unwrap_or_else(env_is_german);
    let (width, height) = args.size.unwrap_or((1000, 640));
    if let Some(path) = &args.picture {
        return picture(german, (width, height), path);
    }
    let Some(socket) = args.socket.clone().or_else(default_socket) else {
        eprintln!("fs-draw-simple: no socket: $XDG_RUNTIME_DIR is not set, pass --socket");
        return 1;
    };
    // The lease first: a server that started for this window must not end before the window is up.
    let _lease = args
        .lease
        .clone()
        .or_else(|| Some(lease_path(&socket)))
        .map(|p| hold_lease(&p));
    let needed = args.needs_contract.as_deref().unwrap_or(CONTRACT);
    let connection = match probe(&socket, needed) {
        Ok(info) => Connection::Connected(info),
        Err(ConnectError::Contract(e)) => {
            // Never silent: the starter shows this text and starts the standard frontend.
            eprintln!("fs-draw-simple: {e}");
            return EXIT_CONTRACT;
        }
        // The window opens anyway and says why there is no server (no grey window without a sentence).
        Err(e) => Connection::NotConnected(e.to_string()),
    };
    show(german, (width, height), &connection)
}

fn show(german: bool, (width, height): (u32, u32), connection: &Connection) -> u8 {
    let widget = match app(german, connection) {
        Ok(widget) => widget,
        Err(e) => {
            eprintln!("fs-draw-simple: {e}");
            return 1;
        }
    };
    let mut options = WindowOptions::default();
    options.width = width;
    options.height = height;
    match launch(options, widget) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("fs-draw-simple: {e}");
            1
        }
    }
}

fn picture(german: bool, (width, height): (u32, u32), path: &std::path::Path) -> u8 {
    let drawn = app(german, &Connection::Preview).and_then(|a| a.picture(width, height, path));
    match drawn {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("fs-draw-simple: {e}");
            1
        }
    }
}
