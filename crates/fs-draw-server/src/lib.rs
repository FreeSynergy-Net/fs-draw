//! `fs-draw-server`: the contract `whiteboard.capnp` over a Unix socket, in front of `fs-draw-core`. It only translates:
//! no board logic lives here.
//!
//! Basics (0.0.2): the root capability reports what it can do, and `Boards` lists, makes, renames and deletes boards.
//! Everything else of the contract answers the explicit "not possible" (ADR 0045), and the report says so too.
//!
//! Interface status: draft (ADR 0045); the contract is a draft and provisional.
//!
//! Pattern: Facade — this file only declares modules and re-exports.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
// The generated `Server` trait is async for every method, also those that never wait.
#![allow(clippy::missing_errors_doc, clippy::unused_async)]

mod boards;
pub mod lease;
mod program;
mod service;
mod wire;

pub use program::{HELP, SOCKET_PROGRAM, ServeError, parse_options, serve};
pub use service::{CONTRACT_VERSION, DrawServer, report};
