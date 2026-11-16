//! The basics of `Boards`: list and make boards.
//!
//! Pattern: Adapter — one short connection per call (the basics have no long session yet); each call runs on a
//! thread-local runtime. Opening a board for editing comes with the editing frontend.

use std::path::Path;

use fs_interfaces::rpc::{connect, read_object_id, write_object_id};
use fs_interfaces::whiteboard_capnp::{board, whiteboard};
use fs_objects::ObjectId;

use crate::connect::ConnectError;

/// A board as the list shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoardInfo {
    /// The id, as text.
    pub id: String,
    /// The name.
    pub name: String,
    /// How many pages are not removed.
    pub pages: usize,
}

fn call_err(e: impl std::fmt::Display) -> ConnectError {
    ConnectError::Call(e.to_string())
}

fn read_board(b: board::Reader<'_>) -> Result<BoardInfo, ConnectError> {
    let id = read_object_id(
        b.get_envelope()
            .map_err(call_err)?
            .get_id()
            .map_err(call_err)?,
    )
    .map_err(call_err)?;
    let envelope = b.get_envelope().map_err(call_err)?;
    let name = envelope
        .get_name()
        .map_err(call_err)?
        .to_string()
        .map_err(call_err)?;
    let mut pages = 0;
    for p in b.get_pages().map_err(call_err)? {
        if !p.get_deleted() {
            pages += 1;
        }
    }
    Ok(BoardInfo {
        id: id.to_string(),
        name,
        pages,
    })
}

/// Run `work` on a thread-local runtime, with the root capability of the server on `socket`.
fn with_root<T>(
    socket: &Path,
    work: impl AsyncFnOnce(whiteboard::Client) -> Result<T, ConnectError>,
) -> Result<T, ConnectError> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(call_err)?;
    tokio::task::LocalSet::new().block_on(&runtime, async {
        let connection = connect::<whiteboard::Client>(socket)
            .await
            .map_err(call_err)?;
        work(connection.client.clone()).await
    })
}

/// The boards of the server on `socket`.
///
/// # Errors
/// The server cannot be reached or answers with a problem.
pub fn list_boards(socket: &Path) -> Result<Vec<BoardInfo>, ConnectError> {
    with_root(socket, async |root| {
        let boards = root
            .boards_request()
            .send()
            .promise
            .await
            .map_err(call_err)?;
        let boards = boards
            .get()
            .map_err(call_err)?
            .get_boards()
            .map_err(call_err)?;
        let page = boards
            .list_request()
            .send()
            .promise
            .await
            .map_err(call_err)?;
        let page = page.get().map_err(call_err)?.get_page().map_err(call_err)?;
        let items = page.get_items().map_err(call_err)?;
        items.iter().map(read_board).collect()
    })
}

/// Make a board named `name` on the server on `socket`.
///
/// # Errors
/// The server cannot be reached or answers with a problem.
pub fn create_board(socket: &Path, name: &str) -> Result<BoardInfo, ConnectError> {
    with_root(socket, async |root| {
        let boards = root
            .boards_request()
            .send()
            .promise
            .await
            .map_err(call_err)?;
        let boards = boards
            .get()
            .map_err(call_err)?
            .get_boards()
            .map_err(call_err)?;
        let mut request = boards.create_request();
        request.get().set_name(name);
        let made = request.send().promise.await.map_err(call_err)?;
        read_board(
            made.get()
                .map_err(call_err)?
                .get_board()
                .map_err(call_err)?,
        )
    })
}

/// Rename the board `id` (as text, from [`BoardInfo::id`]) on the server on `socket`.
///
/// # Errors
/// The id is not one, the server cannot be reached or answers with a problem.
pub fn rename_board(socket: &Path, id: &str, name: &str) -> Result<BoardInfo, ConnectError> {
    let id = parse_id(id)?;
    with_root(socket, async |root| {
        let boards = root
            .boards_request()
            .send()
            .promise
            .await
            .map_err(call_err)?;
        let boards = boards
            .get()
            .map_err(call_err)?
            .get_boards()
            .map_err(call_err)?;
        let mut request = boards.rename_request();
        {
            let mut p = request.get();
            write_object_id(p.reborrow().init_id(), id);
            p.set_name(name);
        }
        let done = request.send().promise.await.map_err(call_err)?;
        read_board(
            done.get()
                .map_err(call_err)?
                .get_board()
                .map_err(call_err)?,
        )
    })
}

/// Delete the board `id` (as text) with everything on it, on the server on `socket`.
///
/// # Errors
/// The id is not one, the server cannot be reached or answers with a problem.
pub fn delete_board(socket: &Path, id: &str) -> Result<(), ConnectError> {
    let id = parse_id(id)?;
    with_root(socket, async |root| {
        let boards = root
            .boards_request()
            .send()
            .promise
            .await
            .map_err(call_err)?;
        let boards = boards
            .get()
            .map_err(call_err)?
            .get_boards()
            .map_err(call_err)?;
        let mut request = boards.delete_request();
        write_object_id(request.get().init_id(), id);
        request.send().promise.await.map_err(call_err)?;
        Ok(())
    })
}

fn parse_id(text: &str) -> Result<ObjectId, ConnectError> {
    text.parse()
        .map_err(|_| call_err(format!("\"{text}\" is not a board id")))
}
