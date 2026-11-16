//! `Boards`: list, make, rename and delete boards. Opening a board for editing, importing and the rest are not carried yet.
//!
//! Pattern: Adapter — wire request in, call of the core's `Boards`, wire answer out. Every write goes through
//! `run_to_end`, so a client that goes away cannot leave a change half done.

use std::rc::Rc;
use std::sync::Arc;

use fs_draw_core::{Boards, Cursor};
use fs_interfaces::rpc::{read_object_id, run_to_end};
use fs_interfaces::whiteboard_capnp::boards;

use crate::service::not_carried;
use crate::wire::{error_of, write_board};

/// The capability `Boards` of the basics.
pub(crate) struct BoardsServer {
    boards: Arc<dyn Boards>,
}

impl BoardsServer {
    pub(crate) fn new(boards: Arc<dyn Boards>) -> Self {
        Self { boards }
    }
}

type Done = Result<(), capnp::Error>;

impl boards::Server for BoardsServer {
    async fn create(
        self: Rc<Self>,
        params: boards::CreateParams,
        mut results: boards::CreateResults,
    ) -> Done {
        let name = params.get()?.get_name()?.to_str()?.to_owned();
        let boards = self.boards.clone();
        let made = run_to_end(async move { boards.create(&name).await }).await?;
        write_board(results.get().init_board(), &made.map_err(|e| error_of(&e))?);
        Ok(())
    }

    async fn rename(
        self: Rc<Self>,
        params: boards::RenameParams,
        mut results: boards::RenameResults,
    ) -> Done {
        let params = params.get()?;
        let id = read_object_id(params.get_id()?)?;
        let name = params.get_name()?.to_str()?.to_owned();
        let boards = self.boards.clone();
        let renamed = run_to_end(async move { boards.rename(id, &name).await }).await?;
        write_board(
            results.get().init_board(),
            &renamed.map_err(|e| error_of(&e))?,
        );
        Ok(())
    }

    async fn delete(
        self: Rc<Self>,
        params: boards::DeleteParams,
        _: boards::DeleteResults,
    ) -> Done {
        let id = read_object_id(params.get()?.get_id()?)?;
        let boards = self.boards.clone();
        run_to_end(async move { boards.delete(id).await })
            .await?
            .map_err(|e| error_of(&e))
    }

    /// Lists every board: the basics have no paging yet (`next` stays unset).
    async fn list(self: Rc<Self>, _: boards::ListParams, mut results: boards::ListResults) -> Done {
        let mut all = Vec::new();
        let mut cursor: Option<Cursor> = None;
        loop {
            let page = self
                .boards
                .list(cursor.as_ref())
                .await
                .map_err(|e| error_of(&e))?;
            all.extend(page.items);
            match page.next {
                Some(next) => cursor = Some(next),
                None => break,
            }
        }
        let mut out = results
            .get()
            .init_page()
            .init_items(u32::try_from(all.len()).unwrap_or(0));
        for (i, b) in (0u32..).zip(&all) {
            write_board(out.reborrow().get(i), b);
        }
        Ok(())
    }

    async fn open(self: Rc<Self>, _: boards::OpenParams, _: boards::OpenResults) -> Done {
        Err(not_carried("board.open"))
    }

    async fn import(self: Rc<Self>, _: boards::ImportParams, _: boards::ImportResults) -> Done {
        Err(not_carried("board.import"))
    }
}
