//! `fs-draw-client`: what a frontend holds of the draw program: the handshake with the server (version and contract),
//! the lease that keeps a server started by the frontend alive, and the basics of the contract (list and make boards).
//!
//! A frontend depends on this crate and on `fs-components`, never on the core or the server (ADR 0094 draft).
//!
//! Interface status: draft (ADR 0045); the contract `whiteboard.capnp` is a draft and provisional.
//!
//! Pattern: Facade — this file only declares modules and re-exports.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]

mod boards;
mod connect;
mod contract;
mod lease;

pub use boards::{BoardInfo, create_board, delete_board, list_boards, rename_board};
pub use connect::{
    CONNECT_TIMEOUT, CommandStarter, ConnectError, HandshakeError, Info, Starter, default_socket,
    handshake, probe,
};
pub use contract::{CONTRACT, ContractError, check_contract};
pub use lease::{hold_lease, lease_path};
