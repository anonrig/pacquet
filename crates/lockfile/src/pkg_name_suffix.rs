use crate::{ParsePkgNameError, PkgName};
use derive_more::{Display, Error};
use serde::{Deserialize, Serialize};
use split_first_char::SplitFirstChar;
use std::str::FromStr;

/// Syntax: `{name}@{suffix}`
///
/// Examples:
/// * `ts-node@10.9.1`, `@types/node@18.7.19`, `typescript@5.1.6`
/// * `react-json-view@1.21.3(@types/react@17.0.49)(react-dom@17.0.2)(react@17.0.2)`
#[derive(Debug, Display, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[display("{name}@{suffix}")]
#[display(bound(Suffix: Display))]
#[serde(try_from = "&'de str", into = "String")]
#[serde(bound(
    deserialize = "Suffix: FromStr, Suffix::Err: Display",
    serialize = "Suffix: Display + Clone",
))]
pub struct PkgNameSuffix<Suffix> {
    pub name: PkgName,
    pub suffix: Suffix,
}

impl<Suffix> PkgNameSuffix<Suffix> {
    /// Construct a [`PkgNameSuffix`].
    pub fn new(name: PkgName, suffix: Suffix) -> Self {
        PkgNameSuffix { name, suffix }
    }
}

/// Error when parsing [`PkgNameSuffix`] from a string.
#[derive(Debug, Display, Error)]
#[display(bound(ParseSuffixError: Display))]
pub enum ParsePkgNameSuffixError<ParseSuffixError> {
    #[display("Input is empty")]
    EmptyInput,
    #[display("Suffix is missing")]
    MissingSuffix,
    #[display("Name is empty")]
    EmptyName,
    #[display("Failed to parse suffix: {_0}")]
    ParseSuffixFailure(#[error(source)] ParseSuffixError),
    #[display("Failed to parse name: {_0}")]
    ParseNameFailure(#[error(source)] ParsePkgNameError),
}

impl<Suffix: FromStr> FromStr for PkgNameSuffix<Suffix> {
    type Err = ParsePkgNameSuffixError<Suffix::Err>;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        // The parsing code of PkgName is insufficient for this, so the code have to be duplicated for now.
        // TODO: use parser combinator pattern to enable code reuse
        let (name, suffix) = match value.split_first_char() {
            None => return Err(ParsePkgNameSuffixError::EmptyInput),
            Some(('@', rest)) => {
                let (name_without_at, suffix) =
                    rest.split_once('@').ok_or(ParsePkgNameSuffixError::MissingSuffix)?;
                let name = &value[..name_without_at.len() + 1];
                debug_assert_eq!(name, format!("@{name_without_at}"));
                (name, suffix)
            }
            Some((_, _)) => value.split_once('@').ok_or(ParsePkgNameSuffixError::MissingSuffix)?,
        };
        if matches!(name, "" | "@" | "@/") {
            return Err(ParsePkgNameSuffixError::EmptyName);
        }
        if suffix.is_empty() {
            return Err(ParsePkgNameSuffixError::MissingSuffix);
        }
        let suffix =
            suffix.parse::<Suffix>().map_err(ParsePkgNameSuffixError::ParseSuffixFailure)?;
        let name = name.parse().map_err(ParsePkgNameSuffixError::ParseNameFailure)?;
        Ok(PkgNameSuffix { name, suffix })
    }
}

impl<'a, Suffix: FromStr> TryFrom<&'a str> for PkgNameSuffix<Suffix> {
    type Error = ParsePkgNameSuffixError<Suffix::Err>;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl<Suffix: Display> From<PkgNameSuffix<Suffix>> for String {
    fn from(value: PkgNameSuffix<Suffix>) -> Self {
        value.to_string()
    }
}
