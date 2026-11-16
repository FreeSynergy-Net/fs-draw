//! What the window says is there, comes next, or comes later. The same list as `todo/features/draw.md` of the
//! documentation (the Stage field there): when one changes, the other changes with it.
//!
//! Pattern: Value objects in a static table — no logic; the window and the tests read it.

/// How far a function is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    /// Works now.
    Today,
    /// The next ones.
    Next,
    /// Later.
    Later,
}

/// One function: the words in English and German.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Item {
    /// How far it is.
    pub stage: Stage,
    /// English.
    pub en: &'static str,
    /// German.
    pub de: &'static str,
}

const fn item(stage: Stage, en: &'static str, de: &'static str) -> Item {
    Item { stage, en, de }
}

/// Every function that is named, in order.
const ITEMS: [Item; 21] = [
    item(
        Stage::Today,
        "A window with name and icon, started by `fs-draw`",
        "Ein Fenster mit Name und Symbol, gestartet mit `fs-draw`",
    ),
    item(
        Stage::Today,
        "A server that lists, makes, renames and deletes boards",
        "Ein Server, der Tafeln auflistet, anlegt, umbenennt und löscht",
    ),
    item(
        Stage::Today,
        "The engine: boards, pages, shapes, text, connectors, pen strokes; merging, undo, SVG export (no screen for them yet)",
        "Der Kern: Tafeln, Seiten, Formen, Text, Verbinder, Stiftstriche; Zusammenführen, Rückgängig, SVG-Export (noch ohne Bildschirm)",
    ),
    item(
        Stage::Next,
        "An endless canvas to look at, with moving and zooming",
        "Eine unendliche Fläche zum Ansehen, mit Verschieben und Zoomen",
    ),
    item(
        Stage::Next,
        "Draw rectangles, ellipses, diamonds, lines and arrows",
        "Rechtecke, Ellipsen, Rauten, Linien und Pfeile zeichnen",
    ),
    item(
        Stage::Next,
        "Text and notes on the canvas",
        "Text und Notizen auf der Fläche",
    ),
    item(Stage::Next, "Freehand pen", "Freihand-Stift"),
    item(
        Stage::Next,
        "Select, move, resize and rotate",
        "Auswählen, verschieben, Größe ändern und drehen",
    ),
    item(
        Stage::Next,
        "Colours, line width, dashed lines",
        "Farben, Linienstärke, gestrichelte Linien",
    ),
    item(
        Stage::Next,
        "Undo and redo, copy, group, align, lock",
        "Rückgängig und Wiederholen, Kopieren, Gruppieren, Ausrichten, Sperren",
    ),
    item(
        Stage::Next,
        "Open and save boards; export as SVG",
        "Tafeln öffnen und speichern; Export als SVG",
    ),
    item(
        Stage::Later,
        "Import of .excalidraw, .drawio and .xopp files",
        "Import von .excalidraw-, .drawio- und .xopp-Dateien",
    ),
    item(
        Stage::Later,
        "Several people on one board at once, with live cursors",
        "Mehrere Menschen gleichzeitig auf einer Tafel, mit Live-Zeigern",
    ),
    item(
        Stage::Later,
        "Pages, frames and images from the Files program",
        "Seiten, Rahmen und Bilder aus dem Dateien-Programm",
    ),
    item(
        Stage::Later,
        "Connectors that follow their shapes",
        "Verbinder, die ihren Formen folgen",
    ),
    item(
        Stage::Later,
        "Libraries of shapes and templates",
        "Bibliotheken mit Formen und Vorlagen",
    ),
    item(
        Stage::Later,
        "Comments on elements",
        "Kommentare an Elementen",
    ),
    item(
        Stage::Later,
        "Sharing a board with people and groups",
        "Eine Tafel mit Menschen und Gruppen teilen",
    ),
    item(
        Stage::Later,
        "Export as PNG and PDF",
        "Export als PNG und PDF",
    ),
    item(
        Stage::Later,
        "Diagrams from text (flow charts, sequence diagrams)",
        "Diagramme aus Text (Ablaufpläne, Sequenzdiagramme)",
    ),
    item(
        Stage::Later,
        "Presenting a board page by page",
        "Eine Tafel Seite für Seite vorführen",
    ),
];

/// Every function that is named, in order: today first, then next, then later.
#[must_use]
pub fn items() -> &'static [Item] {
    &ITEMS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_list_is_ordered_by_stage_and_every_stage_has_items() {
        let all = items();
        assert!(all.windows(2).all(|w| w[0].stage as u8 <= w[1].stage as u8));
        for stage in [Stage::Today, Stage::Next, Stage::Later] {
            assert!(all.iter().any(|i| i.stage == stage));
        }
    }

    #[test]
    fn every_item_has_both_languages() {
        assert!(items().iter().all(|i| !i.en.is_empty() && !i.de.is_empty()));
    }
}
