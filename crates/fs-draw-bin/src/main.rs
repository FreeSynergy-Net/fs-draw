//! `fs-draw`: one binary for the program (ADR 0063). Pattern: Facade over the crates that stay libraries.
//!
//! ```text
//! fs-draw                       makes sure the server runs and opens the frontend `simple`
//! fs-draw --frontend ID         another frontend; `simple` if its contract does not fit
//! fs-draw serve                 the server (Cap'n Proto, whiteboard.capnp)
//! fs-draw --version             the version, without opening anything
//! ```

use std::ffi::OsString;
use std::process::ExitCode;

use fs_draw_bin::{
    Command, HELP, STANDARD, ensure_server, frontend_path, parse, run_with_fallback,
};
use fs_draw_client::{default_socket, hold_lease, lease_path};
use fs_draw_server::ServeError;

fn main() -> ExitCode {
    let words: Vec<OsString> = std::env::args_os().skip(1).collect();
    match parse(&words) {
        Ok(Command::Version) => {
            println!("fs-draw {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Ok(Command::Help) => {
            println!("{HELP}");
            ExitCode::SUCCESS
        }
        Ok(Command::Serve(rest)) => serve(rest),
        Ok(Command::Start {
            frontend,
            socket,
            pass,
        }) => start(&frontend, socket.as_deref(), &pass),
        Err(m) => {
            eprintln!("fs-draw: {m}");
            ExitCode::from(2)
        }
    }
}

fn serve(words: Vec<OsString>) -> ExitCode {
    match fs_draw_server::serve(words) {
        Ok(()) => ExitCode::SUCCESS,
        Err(ServeError::Usage(m)) => {
            eprintln!("fs-draw: {m}");
            ExitCode::from(2)
        }
        Err(ServeError::Failed(m)) => {
            eprintln!("fs-draw: {m}");
            ExitCode::FAILURE
        }
    }
}

fn start(frontend: &str, socket: Option<&str>, pass: &[String]) -> ExitCode {
    let Some(socket_path) = socket.map(Into::into).or_else(default_socket) else {
        eprintln!("fs-draw: no socket: $XDG_RUNTIME_DIR is not set, pass --socket");
        return ExitCode::FAILURE;
    };
    let me = match std::env::current_exe() {
        Ok(me) => me,
        Err(e) => {
            eprintln!("fs-draw: cannot find myself: {e}");
            return ExitCode::FAILURE;
        }
    };
    // The lease first: a server started now must not end before the frontend is up.
    let _lease = hold_lease(&lease_path(&socket_path));
    if let Err(why) = ensure_server(&me, &socket_path, socket.is_some()) {
        eprintln!("fs-draw: no server: {why}");
        return ExitCode::FAILURE;
    }
    let mut args: Vec<OsString> = vec!["--socket".into(), socket_path.as_os_str().to_owned()];
    args.extend(pass.iter().map(OsString::from));
    let code = run_with_fallback(
        &frontend_path(&me, frontend),
        &frontend_path(&me, STANDARD),
        &args,
        &mut |m| eprintln!("fs-draw: {m}"),
    );
    ExitCode::from(u8::try_from(code).unwrap_or(1))
}
