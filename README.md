# FreeSynergy Draw — think visually

**A board for ideas that several people can change at once without a referee.**

FreeSynergy Draw is an endless board with pages, shapes, text, connectors and drawings. You can group things, copy, align, lock and reorder them, and undo your own steps.

What is different here: two people changing the same board are merged property by property. If you move a shape while I recolour it, both changes stay. The merge is written so that the order of arrival does not matter, and property tests check this. Your undo only undoes your own steps and skips a property someone else has since changed. A board can be exported as SVG.

## What it can do today

Latest tag: `v0.0.1` (the engine, no window). Version 0.0.2 (the basics) has no release yet.

Start it with `fs-draw`. It makes sure the server runs, opens the window and, when the server speaks another contract than the window was built for, says so and opens the standard frontend `simple`.

- **A window** (`fs-draw-simple`, the frontend `simple`) with the name, an icon placeholder (a letter in a circle; the real icon comes later), what works now, and the list of what comes. Entries that do not work yet are locked and say "coming" (German: "kommt noch"). German and English.
- **A server** (`fs-draw serve`) that lists, makes, renames and deletes boards over a local socket (the contract `whiteboard.capnp`, a draft). Everything else of the contract answers "not possible yet", and says so in its report.
- **The engine** behind it (`fs-draw-core`): boards, pages, shapes, text, connectors and pen strokes; merging of changes from two sides per property, undo for each person, groups, copy, align, lock, reorder, export to SVG. The window does not show these yet.
- `fs-draw --version` and `fs-draw --help` (also for `fs-draw-simple`) print and end; an unknown option ends with code 2.

Picture of the window: after the picture check (placeholder until then, no foreign pictures).

## What is not there yet

- You cannot draw in the window yet: the canvas, the tools, selecting and moving, colours, open and save come next; the full list, sorted into today / next / later, is in the documentation (`todo/features/draw.md`).
- Nothing yet sends changes between devices, so the merge is not used anywhere. Live presence, sharing, libraries, templates and comments are not built.
- Import of `.excalidraw`, `.drawio` and `.xopp` files and export to PNG and PDF are later.
- The interface is a draft, marked provisional; there is no release of the basics before it is locked.

**Deutsch.** `fs-draw` startet das Programm: es sorgt dafür, dass der Server läuft, öffnet das Fenster und weicht, wenn der Server einen anderen Vertrag spricht als das Fenster, mit einer Meldung auf das Standard-Frontend `simple` aus. Heute gibt es ein Fenster mit Name, Symbol-Platzhalter (Buchstabe im Kreis), „Was heute geht“ und „Was kommt“ (nicht fertige Einträge sind gesperrt und sagen „kommt noch“), einen Server, der Tafeln auflistet, anlegt, umbenennt und löscht, und den Kern dahinter (Tafeln, Seiten, Formen, Text, Verbinder, Stiftstriche, Zusammenführen, Rückgängig, SVG-Export). Zeichnen im Fenster geht noch nicht: Fläche, Werkzeuge, Auswählen, Farben, Öffnen und Speichern kommen als Nächstes; Import fremder Formate, PNG/PDF, Live-Zusammenarbeit und Teilen später. Version 0.0.2 ist noch nicht veröffentlicht. Bild des Fensters: nach der Bildabnahme.

## Part of FreeSynergy

Draw belongs to the project-management suite and is part of the "together" step of the plan. FreeSynergy is a family of programs that are free for everyone, forever, and that keep your data yours. The documentation, the manifesto and the overall plan live in [fs-documentation](https://github.com/FreeSynergy/fs-documentation); the reading version is at <https://freesynergy.github.io/fs-documentation/>. Start with the [manifesto](https://freesynergy.github.io/fs-documentation/manifesto/) if you want to know why we do this.


## For developers

- Workspace crates (engine and frontend are separate processes, ADR 0021): `fs-draw-core` (core, no GUI, no Blinc), `fs-draw-server` (`whiteboard.capnp` over a Unix socket), `fs-draw-client` (what a frontend holds: handshake, lease, boards), `fs-draw-simple` (the frontend `simple`, on fs-components; depends on the client only, never on the core or the server), `fs-draw-bin` (the binary `fs-draw`: starter and `serve`).
- The frontend is a package of its own (ADR 0094 draft, stage 1): `packaging/frontends/simple.metainfo.xml` (AppStream addon). A frontend that finds the server's contract does not fit exits with code 3; the starter then says so and starts `simple`.
- The contract is `whiteboard.capnp` in [fs-interfaces](https://github.com/FreeSynergy/fs-interfaces) (draft, provisional). Draw owns it.
- Build and test: `cargo build --workspace` and `cargo test --workspace --all-features`.
- Design and decisions: [DESIGN.md](DESIGN.md). Licence: MIT OR Apache-2.0.
