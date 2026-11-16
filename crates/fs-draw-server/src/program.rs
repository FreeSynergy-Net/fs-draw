//! `fs-draw serve`: the options, the store and the serving, as a function so one binary can hold server and starter.
//!
//! Pattern: Facade — reads the options, opens the store, serves until a signal. `[--lease FILE]` ends the server when
//! no frontend holds the lease any more (`lease.rs`).

use std::ffi::OsString;
use std::sync::Arc;

use fs_draw_core::WhiteboardRepo;
use fs_interfaces::rpc::{
    ServerIdentity, ServerOptions, prepare_data_dir, run_until_signal_as, stop_self,
};
use fs_objects::{Store, TursoStore};

use crate::DrawServer;

/// The help text of `serve`.
pub const HELP: &str = "fs-draw serve [--socket PATH] [--data DIR] [--actor IRI] [--instance DOMAIN] [--lease FILE]\nDefaults: $XDG_RUNTIME_DIR/freesynergy/draw.sock, $XDG_DATA_HOME/freesynergy/draw, the local owner\n--lease FILE: end when no frontend holds this lock any more (started by a frontend, not by hand)";

/// The program name the socket and the data folder are made from: `$XDG_RUNTIME_DIR/freesynergy/draw.sock` (the client says the same).
pub const SOCKET_PROGRAM: &str = "draw";

/// Why `serve` ended without serving.
#[derive(Debug, PartialEq, Eq)]
pub enum ServeError {
    /// The words were wrong; the caller exits with 2.
    Usage(String),
    /// The server could not start or stopped with an error; exit with 1.
    Failed(String),
}

/// The options of `serve`.
///
/// # Errors
/// The words were unknown or incomplete.
pub fn parse_options(
    args: impl IntoIterator<Item = OsString>,
) -> Result<Option<ServerOptions>, String> {
    ServerOptions::parse(SOCKET_PROGRAM, args)
}

/// Serve `whiteboard.capnp` until a signal arrives. `args` are the words after `serve`.
///
/// # Errors
/// [`ServeError::Usage`] for unknown or incomplete words, [`ServeError::Failed`] for everything else.
pub fn serve(args: impl IntoIterator<Item = OsString>) -> Result<(), ServeError> {
    let (lease, args) =
        crate::lease::take_lease(args.into_iter().collect()).map_err(ServeError::Usage)?;
    let Some(options) = parse_options(args).map_err(ServeError::Usage)? else {
        println!("{HELP}");
        return Ok(());
    };
    if let Some(lease) = lease {
        // The frontend that started this server holds the lease; when none does, end as a signal would.
        crate::lease::watch(lease, stop_self);
    }
    // Name and version go to `Control.version`: a frontend of another version replaces this server (fs-interfaces handshake).
    let identity = ServerIdentity::new("fs-draw", env!("CARGO_PKG_VERSION"));
    run_until_signal_as(&options.socket, identity, || async move {
        let data = prepare_data_dir(&options.data)?;
        let store = TursoStore::open(data.join("objects.db"))
            .await
            .map_err(std::io::Error::other)?;
        let store: Arc<dyn Store> = Arc::new(store);
        let mut repo = WhiteboardRepo::new(store, options.actor);
        if let Some(instance) = options.instance {
            repo = repo.with_instance(instance);
        }
        Ok(DrawServer::new(Arc::new(repo)).into_client().client)
    })
    .map_err(|e| ServeError::Failed(e.to_string()))
}
