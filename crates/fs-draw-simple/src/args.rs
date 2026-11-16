//! The words after `fs-draw-simple`.
//!
//! Pattern: none — a small hand parser; an unknown word is an error (the caller exits with 2).

use std::path::PathBuf;

/// The help text.
pub const HELP: &str = "fs-draw-simple [--socket PATH] [--lease FILE] [--lang de|en] [--size WxH]\n\
\n\
The frontend `simple` of FreeSynergy Draw. Start it with `fs-draw`, which makes sure the server runs.\n\
\n\
Options:\n\
  --socket PATH         the server's socket (default $XDG_RUNTIME_DIR/freesynergy/draw.sock)\n\
  --lease FILE          hold this lease as long as the window lives, so the server does not end under it\n\
  --lang de|en          the language (default: the environment)\n\
  --size WxH            the window size in pixels (default 1000x640)\n\
  --needs-contract TEXT the contract version this frontend asks for (for tests of the handshake)\n\
  --picture FILE        draw the window to a PNG without a server and end\n\
  --screen start|comes  the page that --picture draws (default start)\n\
  --version             print the version and end\n\
  --help                print this text and end\n\
\n\
Exit codes: 0 ok, 1 failure, 2 wrong words, 3 the server's contract does not fit (the starter then starts the standard frontend).";

/// What the words ask for.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Args {
    /// `--version`.
    pub version: bool,
    /// `--help`.
    pub help: bool,
    /// `--socket`.
    pub socket: Option<PathBuf>,
    /// `--lease`.
    pub lease: Option<PathBuf>,
    /// `--lang`: `Some(true)` = German.
    pub german: Option<bool>,
    /// `--size`.
    pub size: Option<(u32, u32)>,
    /// `--needs-contract`.
    pub needs_contract: Option<String>,
    /// `--picture`.
    pub picture: Option<PathBuf>,
    /// `--screen`: the id of the page that `--picture` draws.
    pub screen: Option<String>,
}

/// Read the words.
///
/// # Errors
/// An unknown option, a missing or wrong value.
pub fn parse(words: &[String]) -> Result<Args, String> {
    let mut args = Args::default();
    let mut it = words.iter();
    while let Some(word) = it.next() {
        let mut value = |name: &str| -> Result<&String, String> {
            it.next().ok_or_else(|| format!("{name} needs a value"))
        };
        match word.as_str() {
            "--version" | "-V" => args.version = true,
            "--help" | "-h" => args.help = true,
            "--socket" => args.socket = Some(PathBuf::from(value("--socket")?)),
            "--lease" => args.lease = Some(PathBuf::from(value("--lease")?)),
            "--picture" => args.picture = Some(PathBuf::from(value("--picture")?)),
            "--screen" => {
                let id = value("--screen")?;
                if id != "start" && id != "comes" {
                    return Err(format!("--screen needs start or comes, not {id}"));
                }
                args.screen = Some(id.clone());
            }
            "--needs-contract" => args.needs_contract = Some(value("--needs-contract")?.clone()),
            "--lang" => {
                args.german = Some(match value("--lang")?.as_str() {
                    "de" => true,
                    "en" => false,
                    other => return Err(format!("--lang needs de or en, not {other}")),
                });
            }
            "--size" => args.size = Some(size(value("--size")?)?),
            other => return Err(format!("unknown option {other}")),
        }
    }
    Ok(args)
}

fn size(text: &str) -> Result<(u32, u32), String> {
    let bad = || format!("--size needs WIDTHxHEIGHT, not {text}");
    let (w, h) = text.split_once('x').ok_or_else(bad)?;
    let (w, h): (u32, u32) = (w.parse().map_err(|_| bad())?, h.parse().map_err(|_| bad())?);
    if w == 0 || h == 0 {
        return Err(bad());
    }
    Ok((w, h))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(text: &str) -> Vec<String> {
        text.split_whitespace().map(str::to_owned).collect()
    }

    #[test]
    fn the_known_words_are_read() {
        let a = parse(&words(
            "--socket /s --lease /l --lang de --size 800x600 --version",
        ))
        .unwrap();
        assert_eq!(a.socket, Some("/s".into()));
        assert_eq!(a.lease, Some("/l".into()));
        assert_eq!(a.german, Some(true));
        assert_eq!(a.size, Some((800, 600)));
        assert!(a.version && !a.help);
    }

    #[test]
    fn unknown_words_and_missing_values_are_errors() {
        assert!(
            parse(&words("--bogus"))
                .unwrap_err()
                .contains("unknown option")
        );
        assert!(parse(&words("--socket")).is_err());
        assert!(parse(&words("--lang fr")).is_err());
        assert!(parse(&words("--size 0x5")).is_err());
        assert!(parse(&words("--size big")).is_err());
    }
}
