//! The connection: the version handshake of `fs-interfaces` (a server of another version is replaced, none is started by
//! the starter) and then the contract check (`contract.rs`).
//!
//! Pattern: Adapter — blocking calls over the async client; each call runs on a thread-local runtime that is gone when
//! it returns, so a frontend calls it before the window opens, not from inside an async task.

use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use fs_interfaces::Capabilities;
use fs_interfaces::rpc::{
    Expected, RestartTimes, connect, connect_matching_with, default_socket as socket_of,
};
use fs_interfaces::whiteboard_capnp::whiteboard;

pub use fs_interfaces::rpc::{CommandStarter, HandshakeError, ServerStarter as Starter};

use crate::contract::{ContractError, check_contract};

/// How long a call waits for the server's answer.
pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// The program name the default socket is made from (`$XDG_RUNTIME_DIR/freesynergy/draw.sock`).
const SERVER: &str = "draw";

/// What the server said about itself, after it fitted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Info {
    /// The version of the server (equal to this program's: a server of another version was replaced).
    pub server_version: String,
    /// The contract version the server reported (`SupportReport.contractVersion`).
    pub contract: String,
    /// The capability ids the server offers.
    pub supported: Vec<String>,
    /// The ids it names as not possible yet, with the reason.
    pub unsupported: Vec<(String, String)>,
}

/// Why no usable server was reached.
#[derive(Debug)]
pub enum ConnectError {
    /// No server of this version could be reached or started.
    Handshake(HandshakeError),
    /// The server answered, but its contract does not fit.
    Contract(ContractError),
    /// A call failed.
    Call(String),
}

impl std::fmt::Display for ConnectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Handshake(e) => write!(f, "{e}"),
            Self::Contract(e) => write!(f, "{e}"),
            Self::Call(e) => write!(f, "the server did not answer: {e}"),
        }
    }
}

impl std::error::Error for ConnectError {}

/// Where `fs-draw serve` listens when it is given no `--socket`: `$XDG_RUNTIME_DIR/freesynergy/draw.sock`. `None` when the
/// environment names no runtime folder.
#[must_use]
pub fn default_socket() -> Option<PathBuf> {
    socket_of(SERVER)
}

/// Reach the server of this version on `socket` (the `starter` starts one when none runs, or replaces one of another
/// version), ask what it can do, and check that its contract fits `needed` (use [`CONTRACT`](crate::CONTRACT)). Only the
/// starter of the program uses this; a frontend uses [`probe`], which ends no server.
///
/// # Errors
/// [`ConnectError`], which words itself for the person.
pub fn handshake(socket: &Path, needed: &str, starter: &dyn Starter) -> Result<Info, ConnectError> {
    with_runtime(async {
        let expected = Expected::new(env!("CARGO_PKG_VERSION"));
        let connection = connect_matching_with::<whiteboard::Client>(
            socket,
            &expected,
            starter,
            RestartTimes::default(),
        )
        .await
        .map_err(ConnectError::Handshake)?;
        ask(&connection.client, needed).await
    })
}

/// Connect to the server on `socket` as it is (a frontend never ends or replaces a server), ask what it can do, and check
/// that its contract fits `needed`.
///
/// # Errors
/// [`ConnectError`]: nothing listens there, a call failed, or the contract does not fit.
pub fn probe(socket: &Path, needed: &str) -> Result<Info, ConnectError> {
    with_runtime(async {
        let connection = connect::<whiteboard::Client>(socket)
            .await
            .map_err(|e| ConnectError::Call(e.to_string()))?;
        ask(&connection.client, needed).await
    })
}

/// Run `work` on a thread-local runtime that is gone when it returns.
fn with_runtime<T>(work: impl Future<Output = Result<T, ConnectError>>) -> Result<T, ConnectError> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e: io::Error| ConnectError::Call(e.to_string()))?;
    tokio::task::LocalSet::new().block_on(&runtime, work)
}

/// Ask the support report of `client` and check its contract against `needed`.
async fn ask(client: &whiteboard::Client, needed: &str) -> Result<Info, ConnectError> {
    let call = client.support_report_request().send().promise;
    let response = tokio::time::timeout(CONNECT_TIMEOUT, call)
        .await
        .map_err(|_| ConnectError::Call("no answer in time".into()))?
        .map_err(|e| ConnectError::Call(e.to_string()))?;
    let report = response
        .get()
        .map_err(|e| ConnectError::Call(e.to_string()))?
        .get_report()
        .map_err(|e| ConnectError::Call(e.to_string()))?;
    let contract = report
        .get_contract_version()
        .and_then(|t| t.to_string().map_err(Into::into))
        .map_err(|e| ConnectError::Call(e.to_string()))?;
    check_contract(needed, &contract).map_err(ConnectError::Contract)?;
    let caps = Capabilities::read(report).map_err(|e| ConnectError::Call(e.to_string()))?;
    Ok(info_of(env!("CARGO_PKG_VERSION"), contract, &caps))
}

fn info_of(version: &str, contract: String, caps: &Capabilities) -> Info {
    let (mut supported, mut unsupported) = (Vec::new(), Vec::new());
    for entry in caps.entries() {
        if entry.supported {
            supported.push(entry.capability.clone());
        } else {
            unsupported.push((entry.capability.clone(), entry.reason.clone()));
        }
    }
    Info {
        server_version: version.to_owned(),
        contract,
        supported,
        unsupported,
    }
}
