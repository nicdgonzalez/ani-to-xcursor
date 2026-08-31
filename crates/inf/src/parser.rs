use std::iter::Peekable;
use std::str::Chars;

use crate::error::ParseError;
use crate::section::{Entry, Section};

/// Represents an on-going parse.
#[derive(Debug, Clone)]
pub struct Parser<'a> {
    // TODO: Change to `&'a str`:
    //  data: &'a str,
    //  position: usize,
    chars: Peekable<Chars<'a>>,
    // TODO: Track current line number for better error messages.
    //  line: usize,
}

impl<'a> Parser<'a> {
    #[must_use]
    pub fn new(text: &'a str) -> Self {
        Self {
            chars: text.chars().peekable(),
        }
    }
}

impl Parser<'_> {
    // Moves `self` because we cannot call this function again after reaching the end of `chars`.
    // Not a big fan of this, as the name is sort of misleading with how involved this method
    // actually is.
    //
    // TODO: Prefer moving `sections` into caller and use as a helper to extract the sections.
    pub fn into_sections(mut self) -> Result<Vec<Section>, ParseError> {
        let mut sections = Vec::<Section>::with_capacity(16);

        while let Some(c) = self.chars.next() {
            match c {
                ';' => self.skip_comment(),
                '[' => self.parse_section(&mut sections)?,
                _ => {}
            }
        }

        Ok(sections)
    }

    /// Read to the end of the line since comments start from ';' and end at '\n'.
    fn skip_comment(&mut self) {
        _ = self.chars.find(|&c| c == '\n');
    }

    /// Read each line until the next section or end of file.
    fn parse_section(&mut self, sections: &mut Vec<Section>) -> Result<(), ParseError> {
        let section_name = self.parse_section_name()?;

        // Duplicate section names are allowed; the specification states we should merge their entries.
        let entries = if let Some(i) = sections
            .iter()
            .position(|section| section_name == section.name())
        {
            // If a section with the same name already exists, extend it.
            sections.get_mut(i).unwrap()
        } else {
            // Otherwise, create a new section.
            sections.push(Section::new(section_name, Vec::with_capacity(32)));
            sections.last_mut().unwrap()
        };

        while self.chars.peek().is_some_and(|&c| c != '[') {
            if let Some(line) = self.read_next_entry()? {
                let entry = parse_section_entry(&line)?;
                entries.push(entry);
            }
        }

        Ok(())
    }

    /// Read the line containing the section name.
    fn parse_section_name(&mut self) -> Result<String, ParseError> {
        let section_name = self
            .chars
            .by_ref()
            .take_while(|&c| c != ']')
            .collect::<String>();

        if section_name.is_empty() {
            return Err(ParseError::SectionNameEmpty);
        } else if section_name.len() > 255 {
            return Err(ParseError::SectionNameTooLong);
        }

        // Strip excess whitespace and inline comments; break the loop after consuming the newline.
        while let Some(c) = self.chars.next() {
            match c {
                ';' => {
                    self.skip_comment();
                    break;
                }
                '\n' => break, // Will also consume any Carriage Returns (\r).
                c if c.is_ascii_whitespace() => {
                    assert_ne!(c, '\n', r"\n should have been handled separately");
                }
                c => return Err(ParseError::UnexpectedCharacter { c }),
            }
        }

        Ok(section_name)
    }

    /// Read the next entry while flattening Line Continuators (\) and stripping inline comments.
    fn read_next_entry(&mut self) -> Result<Option<String>, ParseError> {
        let mut line = String::with_capacity(4096);
        let mut within_quotes = false;

        loop {
            let current = self
                .chars
                .by_ref()
                .take_while(|&c| {
                    if c == '"' {
                        within_quotes = !within_quotes;
                    }

                    // If within double quotes, consume everything (including newlines).
                    // TODO: This might be special to the [Strings] section; we are applying it
                    // here to all sections. Additional research required.
                    within_quotes || c != '\n'
                })
                .collect::<String>();
            let mut current = current
                .strip_suffix('\r')
                .unwrap_or(current.as_str())
                .trim_end();

            if within_quotes {
                return Err(ParseError::UnterminatedString);
            }

            // Trim inline comments
            for (i, c) in current.char_indices() {
                match c {
                    '"' => within_quotes = !within_quotes,
                    ';' if !within_quotes => {
                        current = current[..i].trim_end();
                        break;
                    }
                    _ => {}
                }
            }

            if within_quotes {
                return Err(ParseError::UnterminatedString);
            }

            // If the line ends with a Line Continuator, strip it and continue to next line.
            if let Some(s) = current.strip_suffix('\\') {
                line.push_str(s);
                continue;
            }

            line.push_str(current);
            break;
        }

        Ok(if line.is_empty() { None } else { Some(line) })
    }
}

fn parse_section_entry(line: &str) -> Result<Entry, ParseError> {
    assert!(!line.is_empty());
    assert!(!line.ends_with('\\'));
    assert!(!line.contains('\r'));
    assert!(!line.contains('\n'));

    let mut values = Vec::<String>::new();
    let mut within_quotes = false;
    let mut key = None::<String>;
    let mut start = 0;

    for (i, c) in line.char_indices() {
        match c {
            '"' => within_quotes = !within_quotes,
            ',' if !within_quotes => {
                if key.is_some() {
                    assert_ne!(start, 0, "expected start to be after the equal sign");
                }

                let value = normalize_value(&line[start..i])?;
                values.push(value);
                start = i + 1;
            }
            '=' if !within_quotes => {
                if !values.is_empty() {
                    // I'm not sure if unquoted equal signs are allowed, but since it's not
                    // mentioned anywhere in the documentation, we'll assume it is for now.
                    continue;
                }

                key = Some(line[start..i].trim().to_owned());
                start = i + 1;
            }
            _ => {}
        }
    }

    let last = normalize_value(line[start..].trim())?;
    values.push(last);

    let value = if values.len() == 1 {
        values.remove(0).into()
    } else {
        values.into()
    };

    Ok(if let Some(k) = key {
        Entry::Item(k, value)
    } else {
        Entry::Value(value)
    })
}

fn normalize_value(mut value: &str) -> Result<String, ParseError> {
    value = value.trim();
    value = match (value.starts_with('"'), value.ends_with('"')) {
        (true, true) => &value[1..value.len() - 1],
        (false, false) => value,
        _ => return Err(ParseError::UnterminatedString),
    };
    let value = value.replace("\"\"", "\"").replace("\\\\", "\\");

    // NOTE: We do not un-escape percent signs here since it will become ambiguous later whether
    // they were supposed to be for string substitution or simply escaped percent signs.

    Ok(value)
}
