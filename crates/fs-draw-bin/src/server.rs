//! Making sure the server of this version runs: the handshake of `fs-interfaces` through the client, with this program's
//! own `serve` as the starter (detached, with a lease, so it ends with the last frontend).
//!
//! Pattern: Adapter — the starter command is `fs-draw serve --lease <lease> [--socket <socket>]`.

use std::path::Path;

use fs_draw_client::{CONTRACT, CommandStarter, ConnectError, Info, handshake, lease_path};

/// The starter that runs `me serve --lease ... [--socket ...]`.
fn starter(me: &Path, socket: &Path, own_socket: bool) -> CommandStarter {
    let mut words: Vec<std::ffi::OsString> =
        vec!["serve".into(), "--lease".into(), lease_path(socket).into()];
    if own_socket {
        words.extend(["--socket".into(), socket.as_os_str().to_owned()]);
    }
    CommandStarter::new(me, words)
}

/// Make sure a server of this version listens on `socket`: one of this version is used as it is, none is started, one of
/// another version is replaced. Returns what it said, or the reason (in words for the person) when there is none.
/// A contract that does not fit is not an error here: the frontends decide, and they fall back.
///
/// # Errors
/// The words for the person: no server could be started or reached.
pub fn ensure_server(me: &Path, socket: &Path, own_socket: bool) -> Result<Option<Info>, String> {
    match handshake(socket, CONTRACT, &starter(me, socket, own_socket)) {
        Ok(info) => Ok(Some(info)),
        Err(ConnectError::Contract(_)) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}
