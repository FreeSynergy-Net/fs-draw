//! The contract version a frontend was built for, and the rule that decides whether a server fits it.
//!
//! Pattern: Value object + pure function — `"whiteboard 0.1 (draft)"` is the name and `major.minor`; the major stage must
//! be equal and the minor stage of the server must be at least the one asked for (ADR 0094 draft, item 4).

use std::fmt;

/// The contract this client (and the frontend on it) was built for.
pub const CONTRACT: &str = "whiteboard 0.1";

/// Why a server's contract does not fit.
#[derive(Debug, PartialEq, Eq)]
pub enum ContractError {
    /// The name or the major stage differs.
    Major {
        /// What the frontend needs.
        needed: String,
        /// What the server reported.
        found: String,
    },
    /// The server's minor stage is older than needed.
    TooOld {
        /// What the frontend needs.
        needed: String,
        /// What the server reported.
        found: String,
    },
    /// The report names no contract version that can be read.
    Unreadable(String),
}

impl fmt::Display for ContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Major { needed, found } => write!(
                f,
                "the server speaks the contract \"{found}\", this frontend needs \"{needed}\" (another major stage)"
            ),
            Self::TooOld { needed, found } => write!(
                f,
                "the server speaks the contract \"{found}\", which is older than the \"{needed}\" this frontend needs"
            ),
            Self::Unreadable(found) => write!(
                f,
                "the server's contract version \"{found}\" cannot be read"
            ),
        }
    }
}

impl std::error::Error for ContractError {}

/// `("whiteboard", 0, 1)` from `"whiteboard 0.1 (draft)"`.
fn parts(text: &str) -> Option<(&str, u32, u32)> {
    let mut words = text.split_whitespace();
    let name = words.next()?;
    let (major, minor) = words.next()?.split_once('.')?;
    Some((name, major.parse().ok()?, minor.parse().ok()?))
}

/// Whether a server that reported `found` fits a frontend built for `needed`: same name, same major stage, minor stage at
/// least as high.
///
/// # Errors
/// [`ContractError`], which words itself for the person.
pub fn check_contract(needed: &str, found: &str) -> Result<(), ContractError> {
    let (Some(n), Some(f)) = (parts(needed), parts(found)) else {
        return Err(ContractError::Unreadable(found.to_owned()));
    };
    let (needed, found) = (needed.to_owned(), found.to_owned());
    if n.0 != f.0 || n.1 != f.1 {
        Err(ContractError::Major { needed, found })
    } else if f.2 < n.2 {
        Err(ContractError::TooOld { needed, found })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_own_contract_fits_and_a_newer_minor_fits() {
        assert_eq!(check_contract(CONTRACT, "whiteboard 0.1 (draft)"), Ok(()));
        assert_eq!(check_contract(CONTRACT, "whiteboard 0.2"), Ok(()));
    }

    #[test]
    fn another_major_stage_or_name_or_an_older_minor_is_refused() {
        assert!(matches!(
            check_contract(CONTRACT, "whiteboard 1.0"),
            Err(ContractError::Major { .. })
        ));
        assert!(matches!(
            check_contract(CONTRACT, "texts 0.1"),
            Err(ContractError::Major { .. })
        ));
        assert!(matches!(
            check_contract("whiteboard 0.3", "whiteboard 0.2"),
            Err(ContractError::TooOld { .. })
        ));
        assert!(matches!(
            check_contract(CONTRACT, "nonsense"),
            Err(ContractError::Unreadable(_))
        ));
    }
}
