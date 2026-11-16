//! Core values to wire values and core errors to RPC errors.
//!
//! Pattern: Adapter — the contract's `Board` is written from the core's `Board`; a core error leaves as a typed problem.

use fs_draw_core::{Board, Error};
use fs_interfaces::rpc::{failed, invalid_request, unsupported, write_object_id};
use fs_interfaces::whiteboard_capnp::board;

/// Write `b` as the contract's `Board`.
pub(crate) fn write_board(mut out: board::Builder<'_>, b: &Board) {
    {
        let mut envelope = out.reborrow().init_envelope();
        write_object_id(envelope.reborrow().init_id(), b.id);
        envelope.set_iri(&b.iri);
        envelope.set_type("Collection");
        envelope.set_name(&b.name);
    }
    out.set_as2(&b.as2);
    let mut pages = out.init_pages(u32::try_from(b.pages.len()).unwrap_or(0));
    for (i, page) in (0u32..).zip(&b.pages) {
        let mut p = pages.reborrow().get(i);
        write_object_id(p.reborrow().init_id(), page.id);
        p.set_name(&page.name);
        p.set_order(&page.order);
        p.set_deleted(page.deleted);
    }
}

/// The RPC error for a core error: "not possible", a bad request, or a failure.
pub(crate) fn error_of(e: &Error) -> capnp::Error {
    match e {
        Error::Unsupported(u) => unsupported(u),
        Error::Invalid(_) | Error::TooLarge(_) | Error::NotFound { .. } => {
            invalid_request(e.to_string())
        }
        _ => failed(e),
    }
}
