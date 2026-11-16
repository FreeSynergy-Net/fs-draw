//! The two surfaces of the window: the start page (icon placeholder, name, what it is, what works now) and the list of
//! what comes. Nothing is painted by hand: the icon is the avatar block, the text is the text view of `fs-components`.
//!
//! Pattern: Strategy — each surface builds its element when the shell asks; it keeps no state of the element.

use std::sync::{Arc, Mutex};

use fs_components::blinc::layout::div::div;
use fs_components::blinc::layout::image::img;
use fs_components::blinc::layout::widgets::scroll::{Scroll, ScrollPhysics};
use fs_components_widgets::file_preview::picture_address;
use fs_components_widgets::{Block, Element, Inline, ListItem, TextView, styles};

use crate::what_comes::{Item, Stage, items};
use crate::words::{NAME, pick};
use fs_components_app::Surface;

/// The icon of the program (our own drawing, MIT; `packaging/ICON.md`), compiled in as a data address.
const ICON_PNG: &[u8] = include_bytes!("../../../packaging/net.freesynergy.Draw.png");

/// Side of the icon in the window, in pixels.
const ICON_SIZE: f32 = 64.0;

fn icon() -> impl Element + use<> {
    img(picture_address("image/png", ICON_PNG))
        .w(ICON_SIZE)
        .h(ICON_SIZE)
        .contain()
}

fn heading(level: u8, text: &str) -> Block {
    Block::Heading {
        level,
        text: Inline::plain(text),
    }
}

fn paragraph(text: &str) -> Block {
    Block::Paragraph(Inline::plain(text))
}

fn say(german: bool, item: &Item) -> &'static str {
    pick(german, item.en, item.de)
}

/// A list of functions: done ones are ticked, the others are open boxes and carry the words "coming" (ADR 0066: locked
/// with a reason, never silent).
fn list(german: bool, stage: Stage) -> Block {
    let coming = pick(german, " (coming)", " (kommt noch)");
    let rows = items()
        .iter()
        .filter(|i| i.stage == stage)
        .map(|i| {
            let done = stage == Stage::Today;
            let text = if done {
                say(german, i).to_owned()
            } else {
                format!("{}{coming}", say(german, i))
            };
            ListItem::new(vec![paragraph(&text)]).task(done)
        })
        .collect();
    Block::List {
        ordered: false,
        start: 1,
        items: rows,
    }
}

fn page(width: f32, height: f32, blocks: Vec<Block>, with_icon: bool) -> Box<dyn Element> {
    let mut column = div().flex_col().gap(16.0).p(24.0);
    if with_icon {
        column = column.child(
            div()
                .flex_row()
                .items_center()
                .gap(16.0)
                .child(icon())
                .child(TextView::new(vec![heading(1, NAME)]).build()),
        );
    }
    let column = column.child(TextView::new(blocks).tasks_locked(0).build());
    // The page scrolls when it is longer than the window, so nothing is cut off at the bottom.
    let physics = Arc::new(Mutex::new(ScrollPhysics::default()));
    Box::new(
        div().w(width).h(height).child(
            Scroll::with_physics(physics)
                .w(width)
                .h(height)
                .content(column),
        ),
    )
}

/// The start page.
pub struct Start {
    /// Whether the words are German.
    pub german: bool,
    /// One line about the server: connected or not, and why.
    pub server: String,
}

impl Surface for Start {
    fn build(&self, width: f32, height: f32) -> Box<dyn Element> {
        let g = self.german;
        let blocks = vec![
            paragraph(pick(
                g,
                "An endless board for ideas: shapes, text, arrows and drawings, changed by several people at once without a referee.",
                "Eine unendliche Tafel für Ideen: Formen, Text, Pfeile und Zeichnungen, von mehreren Menschen gleichzeitig ohne Schiedsrichter geändert.",
            )),
            paragraph(pick(
                g,
                "This is the first version, the basics: the program starts, the server runs, and the engine behind it is ready. Drawing on the screen comes next.",
                "Das ist die erste Fassung, die Grundlage: das Programm startet, der Server läuft, und der Kern dahinter ist fertig. Das Zeichnen auf dem Bildschirm kommt als Nächstes.",
            )),
            heading(2, pick(g, "What works now", "Was heute geht")),
            list(g, Stage::Today),
            heading(2, pick(g, "Server", "Server")),
            paragraph(&self.server),
        ];
        page(width, height, blocks, true)
    }

    fn styles(&self) -> String {
        styles()
    }
}

/// The list of what comes.
pub struct Comes {
    /// Whether the words are German.
    pub german: bool,
}

impl Surface for Comes {
    fn build(&self, width: f32, height: f32) -> Box<dyn Element> {
        let g = self.german;
        let blocks = vec![
            heading(1, pick(g, "What comes", "Was kommt")),
            paragraph(pick(
                g,
                "Everything below is not there yet. The next ones are the plan for the coming versions; the later ones are ideas that are named so that nobody has to guess.",
                "Alles hier ist noch nicht da. Die nächsten sind der Plan für die kommenden Versionen; die späteren sind Ideen, die genannt werden, damit niemand raten muss.",
            )),
            heading(2, pick(g, "Next", "Als Nächstes")),
            list(g, Stage::Next),
            heading(2, pick(g, "Later", "Später")),
            list(g, Stage::Later),
        ];
        page(width, height, blocks, false)
    }

    fn styles(&self) -> String {
        styles()
    }
}
