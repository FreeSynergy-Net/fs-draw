//! The root capability `Whiteboard`: what the server can do, and the door to `Boards`.
//!
//! Pattern: Adapter + Null Object — the report names every capability id of the contract; the ones the basics do not
//! carry say so plainly (ADR 0045), and their methods answer the same "not possible".

use std::rc::Rc;
use std::sync::Arc;

use capnp_rpc::new_client;
use fs_draw_core::Boards;
use fs_interfaces::rpc::unsupported;
use fs_interfaces::whiteboard_capnp::whiteboard;
use fs_interfaces::{Capabilities, Unsupported};

use crate::boards::BoardsServer;

/// The contract version a report names. The frontend compares it with the one it was built for (`fs-draw-client`).
pub const CONTRACT_VERSION: &str = "whiteboard 0.1 (draft)";

/// The reason given for everything the basics do not carry.
const NOT_YET: &str = "not in the basics of FreeSynergy Draw yet";

/// Capability ids the basics carry.
const SUPPORTED: [&str; 3] = ["board.create", "board.delete", "board.list"];

/// Capability ids of the contract that are not carried yet.
const NOT_CARRIED: [&str; 29] = [
    "board.open",
    "board.import",
    "board.export",
    "board.save",
    "board.view",
    "board.subscribe",
    "canvas.navigate",
    "page.add",
    "page.remove",
    "shape.add",
    "text.add",
    "note.add",
    "stroke.draw",
    "connector.add",
    "image.add",
    "frame.add",
    "element.move",
    "element.style",
    "element.lock",
    "element.delete",
    "element.group",
    "element.ungroup",
    "element.reorder",
    "element.align",
    "text.edit",
    "connector.edit",
    "stroke.erase",
    "history.undo",
    "history.redo",
];

/// What this server can do, per capability id.
#[must_use]
pub fn report() -> Capabilities {
    let mut caps = Capabilities::new("fs-draw");
    for id in SUPPORTED {
        caps.supported(id);
    }
    for id in NOT_CARRIED {
        caps.unsupported(id, NOT_YET);
    }
    caps
}

/// The root capability of the draw program.
pub struct DrawServer {
    boards: Arc<dyn Boards>,
}

impl DrawServer {
    /// Serve this provider.
    pub fn new(boards: Arc<dyn Boards>) -> Self {
        Self { boards }
    }

    /// The capability to hand to `fs_interfaces::rpc::run_until_signal_as`.
    #[must_use]
    pub fn into_client(self) -> whiteboard::Client {
        new_client(self)
    }
}

type Done = Result<(), capnp::Error>;

impl whiteboard::Server for DrawServer {
    async fn capabilities(
        self: Rc<Self>,
        _: whiteboard::CapabilitiesParams,
        mut results: whiteboard::CapabilitiesResults,
    ) -> Done {
        let mut list = results.get().init_capabilities();
        list.set_contract_version(CONTRACT_VERSION);
        let ids = SUPPORTED;
        let mut out = list.init_ids(u32::try_from(ids.len()).unwrap_or(0));
        for (i, id) in (0u32..).zip(ids) {
            out.set(i, id);
        }
        Ok(())
    }

    async fn support_report(
        self: Rc<Self>,
        _: whiteboard::SupportReportParams,
        mut results: whiteboard::SupportReportResults,
    ) -> Done {
        report().write(CONTRACT_VERSION, results.get().init_report());
        Ok(())
    }

    async fn boards(
        self: Rc<Self>,
        _: whiteboard::BoardsParams,
        mut results: whiteboard::BoardsResults,
    ) -> Done {
        results
            .get()
            .set_boards(new_client(BoardsServer::new(self.boards.clone())));
        Ok(())
    }

    async fn subscribe(
        self: Rc<Self>,
        _: whiteboard::SubscribeParams,
        _: whiteboard::SubscribeResults,
    ) -> Done {
        Err(not_carried("board.subscribe"))
    }

    async fn types(
        self: Rc<Self>,
        _: whiteboard::TypesParams,
        _: whiteboard::TypesResults,
    ) -> Done {
        Err(not_carried("types"))
    }
}

/// The explicit "not possible" as an RPC error.
pub(crate) fn not_carried(capability: &str) -> capnp::Error {
    unsupported(&Unsupported::new(capability, NOT_YET))
}
