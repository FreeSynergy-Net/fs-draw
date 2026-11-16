//! The words after `fs-draw`.
//!
//! Pattern: Command — the words become one [`Command`]; an unknown option is an error before anything is started (exit 2).

use std::ffi::OsString;

use crate::frontend::{STANDARD, valid_frontend_id};

/// The help text.
pub const HELP: &str = "fs-draw [--frontend ID] [--socket PATH] [--lang de|en] [--size WxH]\n\
fs-draw serve [--socket PATH] [--data DIR] [--actor IRI] [--instance DOMAIN]\n\
fs-draw --version | --help\n\
\n\
FreeSynergy Draw. Without arguments it makes sure the server runs and opens the frontend `simple`.\n\
\n\
Options:\n\
  --frontend ID   the frontend to start (default: simple); if its contract does not fit the server, `simple` is started\n\
  --socket PATH   the server's socket (default $XDG_RUNTIME_DIR/freesynergy/draw.sock)\n\
  --lang de|en    the language of the window (default: the environment)\n\
  --size WxH      the window size in pixels\n\
  serve           run the server only (`fs-draw serve --help` lists its options)\n\
  --version       print the version and end\n\
  --help          print this text and end\n\
\n\
Exit codes: 0 ok, 1 failure, 2 wrong words, 3 no frontend fits the server's contract.";

/// What the words ask for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    /// `--version`.
    Version,
    /// `--help`.
    Help,
    /// `serve ...`: the words go to the server.
    Serve(Vec<OsString>),
    /// The starter: the frontend id, the socket (if given) and the words the frontend gets too.
    Start {
        /// The frontend id.
        frontend: String,
        /// `--socket`, if given.
        socket: Option<String>,
        /// `--lang` and `--size`, as words for the frontend.
        pass: Vec<String>,
    },
}

/// Read the words.
///
/// # Errors
/// An unknown option, a missing value or a frontend id that is not allowed.
pub fn parse(words: &[OsString]) -> Result<Command, String> {
    let text: Vec<String> = words
        .iter()
        .map(|w| w.to_string_lossy().into_owned())
        .collect();
    match text.first().map(String::as_str) {
        Some("--version" | "-V") => return Ok(Command::Version),
        Some("--help" | "-h") => return Ok(Command::Help),
        Some("serve") => return Ok(Command::Serve(words[1..].to_vec())),
        _ => {}
    }
    let (mut frontend, mut socket, mut pass) = (STANDARD.to_owned(), None, Vec::new());
    let mut it = text.iter();
    while let Some(word) = it.next() {
        let mut value = |name: &str| {
            it.next()
                .cloned()
                .ok_or_else(|| format!("{name} needs a value"))
        };
        match word.as_str() {
            "--version" | "-V" => return Ok(Command::Version),
            "--help" | "-h" => return Ok(Command::Help),
            "--frontend" => {
                frontend = value("--frontend")?;
                if !valid_frontend_id(&frontend) {
                    return Err(format!("--frontend: \"{frontend}\" is not a frontend id"));
                }
            }
            "--socket" => socket = Some(value("--socket")?),
            "--lang" | "--size" => {
                let v = value(word)?;
                pass.extend([word.clone(), v]);
            }
            other => return Err(format!("unknown option {other}")),
        }
    }
    Ok(Command::Start {
        frontend,
        socket,
        pass,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(words: &[&str]) -> Result<Command, String> {
        parse(&words.iter().map(OsString::from).collect::<Vec<_>>())
    }

    #[test]
    fn no_words_start_the_standard_frontend() {
        assert_eq!(
            p(&[]),
            Ok(Command::Start {
                frontend: "simple".into(),
                socket: None,
                pass: vec![]
            })
        );
    }

    #[test]
    fn version_help_and_serve_are_read() {
        assert_eq!(p(&["--version"]), Ok(Command::Version));
        assert_eq!(p(&["--help"]), Ok(Command::Help));
        assert_eq!(
            p(&["serve", "--socket", "x"]),
            Ok(Command::Serve(vec!["--socket".into(), "x".into()]))
        );
    }

    #[test]
    fn unknown_options_and_bad_frontend_ids_are_errors() {
        assert!(p(&["--bogus"]).unwrap_err().contains("unknown option"));
        assert!(p(&["--frontend"]).is_err());
        assert!(p(&["--frontend", "core"]).is_err());
        assert!(p(&["--frontend", "../x"]).is_err());
    }

    #[test]
    fn options_for_the_frontend_are_passed_on() {
        assert_eq!(
            p(&["--lang", "de", "--socket", "/s", "--frontend", "paper"]),
            Ok(Command::Start {
                frontend: "paper".into(),
                socket: Some("/s".into()),
                pass: vec!["--lang".into(), "de".into()],
            })
        );
    }
}
