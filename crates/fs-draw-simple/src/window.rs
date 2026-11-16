//! The window: name, icon placeholder, what works, what comes; menus whose entries are locked with "coming".
//!
//! Pattern: Builder — `app` puts the surfaces and the menus into the shell of `fs-components-app`; nothing here knows
//! Blinc. The server line comes from the connection the caller made (the window never connects by itself).

use std::sync::Arc;

use fs_components_app::{AppError, AppWidget, Menu, MenuEntry, Screen, Status};
use fs_draw_client::Info;

use crate::surface::{Comes, Start};
use crate::words::{NAME, pick};

/// What the window knows about the server.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Connection {
    /// The handshake worked.
    Connected(Info),
    /// No usable server, and why (the window opens anyway and says so).
    NotConnected(String),
    /// A picture of the window without a server.
    Preview,
}

impl Connection {
    /// The sentence about the server.
    #[must_use]
    pub fn say(&self, german: bool) -> String {
        match self {
            Self::Connected(info) => {
                let (en, de) = (
                    format!(
                        "Connected. Version {}, contract {}: {} functions of the contract work, {} are named as coming.",
                        info.server_version,
                        info.contract,
                        info.supported.len(),
                        info.unsupported.len()
                    ),
                    format!(
                        "Verbunden. Version {}, Vertrag {}: {} Funktionen des Vertrags gehen, {} sind als kommend benannt.",
                        info.server_version,
                        info.contract,
                        info.supported.len(),
                        info.unsupported.len()
                    ),
                );
                if german { de } else { en }
            }
            Self::NotConnected(why) => format!(
                "{} {why}",
                pick(german, "Not connected:", "Nicht verbunden:")
            ),
            Self::Preview => {
                pick(german, "Preview without a server.", "Vorschau ohne Server.").to_owned()
            }
        }
    }
}

/// A locked entry: it says that it is coming.
fn locked(id: &str, label: &str, german: bool) -> MenuEntry {
    MenuEntry::new(
        id,
        format!("{label} ({})", pick(german, "coming", "kommt noch")),
    )
    .disabled()
}

fn menus(german: bool) -> Vec<Menu> {
    let t = |en, de| pick(german, en, de);
    vec![
        Menu::new(
            t("Board", "Tafel"),
            vec![
                locked("new", t("New board", "Neue Tafel"), german),
                locked("open", t("Open", "Öffnen"), german),
                locked("export", t("Export as SVG", "Als SVG exportieren"), german),
            ],
        ),
        Menu::new(
            t("View", "Ansicht"),
            vec![
                MenuEntry::new("show-start", t("Start", "Start")),
                MenuEntry::new("show-comes", t("What comes", "Was kommt")),
            ],
        ),
    ]
}

/// The window's shell, in the language asked, with the line about the server.
///
/// # Errors
/// The shell refused the screens (never with these).
pub fn app(german: bool, connection: &Connection) -> Result<AppWidget, AppError> {
    let app = AppWidget::new(
        NAME,
        vec![
            Screen::new(
                "start",
                pick(german, "Start", "Start"),
                Arc::new(Start {
                    german,
                    server: connection.say(german),
                }),
            ),
            Screen::new(
                "comes",
                pick(german, "What comes", "Was kommt"),
                Arc::new(Comes { german }),
            ),
        ],
    )?;
    let this = app.clone();
    app.set_menus(menus(german), move |id| match id {
        "show-start" => {
            this.select("start");
        }
        "show-comes" => {
            this.select("comes");
        }
        _ => {}
    })?;
    let status = match connection {
        Connection::Connected(_) | Connection::Preview => Status::Info(connection.say(german)),
        Connection::NotConnected(_) => Status::Problem(connection.say(german)),
    };
    app.set_status(Some(status));
    Ok(app)
}
