use std::num::ParseIntError;
use std::str::FromStr;

use serde::Deserialize;

#[must_use]
pub const fn default_sizes() -> [Size; 6] {
    [Size(24), Size(32), Size(48), Size(64), Size(96), Size(128)]
}

/// Value that is known to be a valid cursor size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct Size(u8);

impl Size {
    /// Valid cursor sizes.
    pub const VALID: [u8; 6] = [24, 32, 48, 64, 96, 128];

    /// Creates a new size.
    #[must_use]
    pub const fn new(value: u8) -> Self {
        Self(value)
    }

    /// Creates a new size if `value` is a standard cursor size.
    ///
    /// Standard sizes include 24, 32, 48, 64, 96, and 128.
    #[must_use]
    pub fn checked_new(value: u8) -> Option<Self> {
        Self::VALID.contains(&value).then_some(Self(value))
    }

    /// Gives ownership of the stored inner value.
    #[must_use]
    pub fn into_inner(self) -> u8 {
        self.0
    }
}

/// Errors that can occur while trying to parse a cursor size.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CursorSizeError {
    #[error("not a number")]
    NotANumber(#[source] ParseIntError),

    #[error("non-standard cursor size")]
    NonStandard,
}

impl FromStr for Size {
    type Err = CursorSizeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s.parse().map_err(CursorSizeError::NotANumber)?;
        Self::checked_new(value).ok_or(CursorSizeError::NonStandard)
    }
}

impl From<Size> for u8 {
    fn from(value: Size) -> Self {
        value.into_inner()
    }
}

impl From<Size> for u16 {
    fn from(value: Size) -> Self {
        value.into_inner().into()
    }
}

impl From<Size> for u32 {
    fn from(value: Size) -> Self {
        value.into_inner().into()
    }
}

impl From<Size> for u64 {
    fn from(value: Size) -> Self {
        value.into_inner().into()
    }
}

impl<'de> serde::Deserialize<'de> for Size {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        Self::checked_new(value).ok_or(serde::de::Error::custom("non-standard cursor size"))
    }
}

/// Collection of valid cursor sizes.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct TomlSizes(pub Vec<Size>);

impl TomlSizes {
    /// Gives ownership of the stored inner value.
    #[must_use]
    pub fn into_inner(self) -> Vec<Size> {
        self.0
    }
}

impl AsRef<[Size]> for TomlSizes {
    fn as_ref(&self) -> &[Size] {
        self.0.as_ref()
    }
}

impl Default for TomlSizes {
    fn default() -> Self {
        Self(Vec::from_iter([
            Size(24),
            Size(32),
            Size(48),
            Size(64),
            Size(96),
            Size(128),
        ]))
    }
}

impl<'de> Deserialize<'de> for TomlSizes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Vec::deserialize(deserializer)?;

        if value.is_empty() {
            Err(serde::de::Error::custom("sizes must not be empty"))
        } else {
            Ok(Self(value))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_sizes() {
        for value in [0, 23, 25, 31, 33, 47, 49, 63, 65, 95, 97, 127, 129, 255] {
            assert_eq!(Size::checked_new(value), None, "value: {value}");
        }
    }

    #[test]
    fn valid_sizes() {
        for value in Size::VALID {
            assert_eq!(Size::checked_new(value), Some(Size(value)));
        }
    }

    #[test]
    fn inner_value() {
        let value = 32;
        let cursor_size = Size::checked_new(value).expect("hardcoded value to be valid");
        assert_eq!(cursor_size.into_inner(), value);
    }

    #[test]
    fn parse_valid() {
        for value in Size::VALID {
            let s = value.to_string();
            let cursor_size = s.parse::<Size>().unwrap();
            assert_eq!(cursor_size, Size(value));
        }
    }

    #[test]
    fn parse_non_standard() {
        let error = "69".parse::<Size>().unwrap_err();
        assert_eq!(error, CursorSizeError::NonStandard);
    }

    #[test]
    fn parse_not_a_number() {
        let error = "NaN".parse::<Size>().unwrap_err();
        assert!(matches!(error, CursorSizeError::NotANumber(_)));
    }

    #[test]
    fn parse_empty_string() {
        let error = "".parse::<Size>().unwrap_err();
        assert!(matches!(error, CursorSizeError::NotANumber(_)));
    }
}
