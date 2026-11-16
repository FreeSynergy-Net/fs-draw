//! `fs-draw-simple`: the frontend `simple` of FreeSynergy draw (ADR 0094 draft, stage 1: its own crate and process in the
//! release package of the program). It shows the window of the basics on `fs-components` and speaks to the draw server
//! through `fs-draw-client` only; it never depends on the core or the server (a test checks `cargo tree`).
//!
//! Pattern: Facade — this file only declares modules and re-exports.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]

mod args;
mod run;
mod surface;
mod what_comes;
mod window;
mod words;

pub use args::{Args, HELP, parse};
pub use run::{EXIT_CONTRACT, run_code};
pub use what_comes::{Item, Stage, items};
pub use window::{Connection, app};
