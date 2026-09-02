#![warn(
    clippy::correctness,
    clippy::suspicious,
    clippy::complexity,
    clippy::perf,
    clippy::style,
    clippy::pedantic
)]

use std::fs::File;
use std::io::Read;
use std::path::Path;

pub use error::InfError as ParseError;
pub use section::{AddRegistry, AddRegistryEntry, Entry, InvalidAddRegistryEntry, Section, Value};

use crate::parser::Parser;

mod error;
mod parser;
mod section;
pub mod util;

/// The Byte Order Mark (BOM) is used to signal the endianness of an encoding.
/// The order `FF FE` strongly suggests that the data is encoded using little-endian byte order.
///
/// <https://en.wikipedia.org/wiki/Byte_order_mark>
const BOM_LE: [u8; 2] = [0xFF, 0xFE];

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Inf {
    // Using `Vec` instead of `HashMap` to preserve ordering.
    sections: Vec<Section>,
}

impl Inf {
    pub fn open<P>(path: P) -> Result<Self, ParseError>
    where
        P: AsRef<Path>,
    {
        File::open(path)
            .map_err(|source| ParseError::ReadFailure { source })
            .and_then(Self::from_reader)
    }

    pub fn from_reader<R>(mut reader: R) -> Result<Self, ParseError>
    where
        R: Read,
    {
        let mut buffer = Vec::new();
        reader
            .read_to_end(&mut buffer)
            .map_err(|err| ParseError::ReadFailure { source: err })?;

        Self::try_from(buffer.as_slice())
    }

    pub fn from_bytes(buffer: &[u8]) -> Result<Self, ParseError> {
        Self::try_from(buffer)
    }

    #[must_use]
    pub fn sections(&self) -> &[Section] {
        &self.sections
    }

    /// Returns the first section whose name matches `name`, ignoring ASCII case.
    ///
    /// This function iterates over each section from the top of the INF file to the bottom.
    /// The comparison is ASCII case-insensitive and does not allocate. If no section matches
    /// the name provided, `None` is returned instead.
    ///
    /// # Performance
    ///
    /// This function performs a linear scan over all sections and has a **O(n)** time complexity.
    ///
    /// # Notes
    ///
    /// This method performs ASCII-only case folding. Non-ASCII characters must match exactly.
    ///
    /// # Examples
    ///
    /// ```
    /// use inf::Inf;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let inf = Inf::from_bytes(b"[Version]\nSignature=\"$Chicago$\"")?;
    /// let section = inf.get("version").unwrap();
    /// assert_eq!(section.name(), "Version");
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Section> {
        self.sections
            .iter()
            .find(|section| name.eq_ignore_ascii_case(section.name()))
    }

    #[must_use]
    pub fn strings(&self) -> Section {
        self.get("Strings")
            .cloned()
            .unwrap_or_else(|| Section::new("Strings".to_owned(), vec![]))
    }
}

impl TryFrom<&[u8]> for Inf {
    type Error = ParseError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let text = decode_data(data);
        let parser = Parser::new(&text);
        let sections = parser.into_sections()?;

        Ok(Self { sections })
    }
}

/// Converts a slice of bytes into a UTF-8 string that we can iterate over.
fn decode_data(data: &[u8]) -> String {
    // INF files must be saved with UTF-16 LE or ANSI file encodings. Because ANSI is a subset
    // of UTF-8 and endianness is irrelevant to UTF-8, the BOM being present strongly suggests
    // that the data was encoded with UTF-16 LE.
    if data.starts_with(&BOM_LE) {
        let utf16 = data[BOM_LE.len()..]
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<u16>>();

        char::decode_utf16(utf16)
            .map(|c| c.unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect::<String>()
    } else {
        String::from_utf8_lossy(data).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiline_value_with_inline_comments() {
        let buffer = b"\
            [Section]\n\
            key = value1,\"value2;not-a-comment\"\\ ; This is an inline comment.\n\
            ,value3,,value5
        ";
        let inf = Inf::from_bytes(buffer).expect("failed to parse hardcoded INF file");

        assert_eq!(
            inf.sections(),
            &vec![Section::new(
                "Section".to_owned(),
                vec![Entry::Item(
                    "key".to_owned(),
                    Value::List(vec![
                        "value1".to_owned(),
                        "value2;not-a-comment".to_owned(),
                        "value3".to_owned(),
                        String::new(),
                        "value5".to_owned()
                    ]),
                )]
            )]
        );
    }

    #[test]
    fn lines_end_with_crlf() {
        let buffer = b"\
            [Version] ; This section is required\r\n\
            signature = \"$CHICAGO$\"\r\
        ";
        let inf = Inf::from_bytes(buffer).expect("failed to parse hardcoded INF file");

        assert_eq!(
            inf.sections(),
            &vec![Section::new(
                "Version".to_owned(),
                vec![Entry::Item(
                    "signature".to_owned(),
                    Value::Raw("$CHICAGO$".to_owned())
                )]
            )]
        );
    }

    #[test]
    fn multiple_sections() {
        let buffer = b"\
            [Section1]\n\
            [Section2]\n\
            [Section3]\
        ";
        let inf = Inf::from_bytes(buffer).expect("failed to parse hardcoded INF file");

        assert_eq!(
            inf.sections(),
            vec![
                Section::new("Section1".to_owned(), vec![]),
                Section::new("Section2".to_owned(), vec![]),
                Section::new("Section3".to_owned(), vec![]),
            ]
        );
    }

    #[test]
    fn multiple_entries() {
        let buffer = b"\
            [Section]\n\
            key1 = value1\n\
            key2 = value2\n\
            key3 = value3\
        ";
        let inf = Inf::from_bytes(buffer).expect("failed to parse hardcoded INF file");

        assert_eq!(
            inf.sections(),
            &vec![Section::new(
                "Section".to_owned(),
                vec![
                    Entry::Item("key1".to_owned(), Value::Raw("value1".to_owned())),
                    Entry::Item("key2".to_owned(), Value::Raw("value2".to_owned())),
                    Entry::Item("key3".to_owned(), Value::Raw("value3".to_owned())),
                ]
            )]
        );
    }

    #[test]
    fn mixed_entry_kinds() {
        let buffer = b"\
            [Section]\n\
            value\n\
            \"value1\",value2,,\"value4\\\"\n\
            key = value\
        ";
        let inf = Inf::from_bytes(buffer).expect("failed to parse hardcoded INF file");

        assert_eq!(
            inf.sections(),
            &vec![Section::new(
                "Section".to_owned(),
                vec![
                    Entry::Value(Value::Raw("value".to_owned())),
                    Entry::Value(Value::List(vec![
                        "value1".to_owned(),
                        "value2".to_owned(),
                        String::new(),
                        "value4\\".to_owned()
                    ])),
                    Entry::Item("key".to_owned(), Value::Raw("value".to_owned())),
                ]
            )]
        );
    }

    #[test]
    fn item_value_quoted() {
        let buffer = b"\
            [Section]\n\
            key = \"value\"\
        ";
        let inf = Inf::from_bytes(buffer).expect("failed to parse hardcoded INF file");

        assert_eq!(
            inf.sections(),
            &vec![Section::new(
                "Section".to_owned(),
                vec![Entry::Item(
                    "key".to_owned(),
                    Value::Raw("value".to_owned())
                )]
            )]
        );
    }

    #[test]
    fn item_value_unquoted() {
        let buffer = b"\
            [Section]\n\
            key = value\n\
        ";
        let inf = Inf::from_bytes(buffer).expect("failed to parse hardcoded INF file");

        assert_eq!(
            inf.sections(),
            &vec![Section::new(
                "Section".to_owned(),
                vec![Entry::Item(
                    "key".to_owned(),
                    Value::Raw("value".to_owned())
                )]
            )]
        );
    }

    #[test]
    fn item_value_unquoted_with_spaces() {
        let buffer = b"\
            [Section]\n\
            key = unquoted value with spaces\
        ";
        let inf = Inf::from_bytes(buffer).expect("failed to parse hardcoded INF file");

        assert_eq!(
            inf.sections(),
            &vec![Section::new(
                "Section".to_owned(),
                vec![Entry::Item(
                    "key".to_owned(),
                    Value::Raw("unquoted value with spaces".to_owned())
                )]
            )]
        );
    }

    #[test]
    fn item_value_quoted_with_leading_spaces() {
        let buffer = b"\
            [Section]\n\
            key = \"    with 4 leading spaces\"\
        ";
        let inf = Inf::from_bytes(buffer).expect("failed to parse hardcoded INF file");

        assert_eq!(
            inf.sections(),
            &vec![Section::new(
                "Section".to_owned(),
                vec![Entry::Item(
                    "key".to_owned(),
                    Value::Raw("    with 4 leading spaces".to_owned())
                )]
            )]
        );
    }

    #[test]
    fn item_value_quoted_with_trailing_spaces() {
        let buffer = b"\
            [Section]\n\
            key = \"with 5 trailing spaces     \"\
        ";
        let inf = Inf::from_bytes(buffer).expect("failed to parse hardcoded INF file");

        assert_eq!(
            inf.sections(),
            &vec![Section::new(
                "Section".to_owned(),
                vec![Entry::Item(
                    "key".to_owned(),
                    Value::Raw("with 5 trailing spaces     ".to_owned())
                )]
            )]
        );
    }

    #[test]
    fn item_value_quoted_with_equal_sign() {
        let buffer = b"\
            [Section]\n\
            \"1+1=2\"
        ";
        let inf = Inf::from_bytes(buffer).expect("failed to parse hardcoded INF file");

        assert_eq!(
            inf.sections(),
            &vec![Section::new(
                "Section".to_owned(),
                vec![Entry::Value(Value::Raw("1+1=2".to_owned()))]
            )]
        );
    }

    #[test]
    fn starts_with_a_comment() {
        let buffer = b"\
            ; This is a comment!\r\n\
            [Section]\r\n\
            key = value\
        ";
        let inf = Inf::from_bytes(buffer).expect("failed to parse hardcoded INF file");

        assert_eq!(
            inf.sections(),
            &vec![Section::new(
                "Section".to_owned(),
                vec![Entry::Item(
                    "key".to_owned(),
                    Value::Raw("value".to_owned())
                )]
            )]
        );
    }
}
