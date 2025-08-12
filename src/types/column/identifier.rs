use anyhow::{Context, bail};
use derive_more::Display;
use itertools::Itertools;
use once_cell::sync::Lazy;
use std::str::FromStr;
use thiserror::Error;

use regex::Regex;

use crate::types::{helpers::invalid_char::InvalidChar, properties::systemfolder::SystemFolder};

static INVALID_FIRST_CHARACTER: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[^A-Za-z_]").unwrap());
static INVALID_CHARACTER: Lazy<Regex> = Lazy::new(|| Regex::new(r"[^A-Za-z0-9_\.]").unwrap());

/// May only contain ASCII characters of the set [A-Za-z0-9_\.]
/// Must start with either a letter or underscore.
///
/// Reference: https://learn.microsoft.com/en-us/windows/win32/msi/identifier
#[derive(Clone, Debug, Display, PartialEq)]
pub struct Identifier {
    inner: String,
}

impl FromStr for Identifier {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(hit) = INVALID_FIRST_CHARACTER.find(s) {
            bail!(IdentifierConversionError::InvalidFirstCharacter {
                first_character: InvalidChar::new(hit.as_str().chars().next().unwrap(), 0),
            });
        }

        if INVALID_CHARACTER.is_match(s) {
            let characters = INVALID_CHARACTER
                .find_iter(s)
                .enumerate()
                .map(|(index, hit)| InvalidChar::new(hit.as_str().chars().next().unwrap(), index))
                .collect_vec();
            bail!(IdentifierConversionError::InvalidCharacters { characters });
        }

        Ok(Identifier {
            inner: s.to_string(),
        })
    }
}

impl From<SystemFolder> for Identifier {
    fn from(value: SystemFolder) -> Self {
        value
            .to_string()
            .parse()
            .context(format!(
                "Failed to parse system folder {value:?} to identifier"
            ))
            .unwrap()
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum IdentifierConversionError {
    #[error("Identifier has invalid first character: [{first_character}]")]
    InvalidFirstCharacter { first_character: InvalidChar },
    #[error("Identifier contains invalid characters")]
    InvalidCharacters { characters: Vec<InvalidChar> },
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use test_case::test_case;

    use crate::types::{
        column::identifier::IdentifierConversionError, helpers::invalid_char::InvalidChar,
    };

    use super::Identifier;
    #[test_case("Test8."; "starts with letter")]
    #[test_case("_Test8."; "starts with underscore")]
    fn valid_identifier(input: &str) {
        let expected = Identifier {
            inner: input.to_owned(),
        };
        let actual = Identifier::from_str(input).expect("Valid identifier returning as invalid");
        assert_eq!(expected, actual);
    }

    #[test_case(".Test"; "starts with period")]
    #[test_case("8Test"; "starts with number")]
    fn invalid_first_character(input: &str) {
        let actual = Identifier::from_str(input)
            .expect_err("Invalid identifier is returning as valid")
            .downcast()
            .unwrap();
        let expected = IdentifierConversionError::InvalidFirstCharacter {
            first_character: InvalidChar::new(input.chars().next().unwrap(), 0),
        };
        assert_eq!(expected, actual);
    }
}
