//! `fs-draw`: one command for the program (ADR 0063). Without arguments it is the starter: it makes sure the server of this
//! version runs, starts the frontend (standard: `simple`) and, when that frontend says the server's contract does not fit,
//! shows a message and starts the standard frontend instead (ADR 0094 draft, item 4). `serve` runs the server.
//!
//! Pattern: Facade — this file only declares modules and re-exports.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]

mod cli;
mod frontend;
mod server;

pub use cli::{Command, HELP, parse};
pub use frontend::{EXIT_CONTRACT, STANDARD, frontend_path, run_with_fallback, valid_frontend_id};
pub use server::ensure_server;
