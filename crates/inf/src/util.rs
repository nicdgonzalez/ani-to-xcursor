use std::fmt;

use crate::section::{Entry, Section, Value};

pub fn expand_vars(value: &str, strings: &Section) -> Result<String, ExpandVarsError> {
    let mut result = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();

    while let Some(c) = chars.next() {
        if c != '%' {
            result.push(c);
            continue;
        }

        // Escaped percent: %%
        if matches!(chars.peek(), Some('%')) {
            _ = chars.next();
            result.push('%');
            continue;
        }

        // Start of %strkey%
        let mut var = String::new();

        loop {
            match chars.next() {
                Some('%') => break,
                Some(ch) => var.push(ch),
                None => return Err(ExpandVarsError::Unterminated),
            }
        }

        let var_lowercase = var.to_lowercase();

        let replacement = strings
            .entries()
            .iter()
            .find_map(|entry| match entry {
                Entry::Item(key, value) if var_lowercase == key.to_lowercase() => {
                    match value {
                        Value::Raw(s) => Some(s.as_str()),
                        // TODO: [Strings] section is special and should not be allowed to have
                        // Value::List. Not an urgent problem since we are only reading INF files,
                        // but this needs to be fixed if we ever want to implement an INF writer.
                        Value::List(..) => None,
                    }
                }
                _ => None,
            })
            .ok_or(ExpandVarsError::NotFound(var))?;

        result.push_str(replacement);
    }

    Ok(result)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpandVarsError {
    Unterminated,
    NotFound(String),
}

impl std::error::Error for ExpandVarsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl fmt::Display for ExpandVarsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Unterminated => "unterminated %strkey% sequence".fmt(f),
            Self::NotFound(ref strkey) => write!(f, "key {strkey:?} not found in [Strings]"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand() {
        let strings = Section::new(
            "Strings".to_owned(),
            vec![Entry::Item(
                "name".to_owned(),
                Value::Raw("Stinky".to_owned()),
            )],
        );

        let expanded =
            expand_vars("Hello, %name%!", &strings).expect("expected hardcoded string to be valid");

        assert_eq!(expanded, "Hello, Stinky!".to_owned());
    }

    #[test]
    fn expand_and_escape() {
        let strings = Section::new(
            "Strings".to_owned(),
            vec![Entry::Item(
                "percentage".to_owned(),
                Value::Raw("50".to_owned()),
            )],
        );

        let expanded = expand_vars("There is a %percentage%%% chance of rain today", &strings)
            .expect("expected hardcoded string to be valid");

        assert_eq!(expanded, "There is a 50% chance of rain today".to_owned());
    }

    #[test]
    fn multiple_expands() {
        let strings = Section::new(
            "Strings".to_owned(),
            vec![Entry::Item(
                "color".to_owned(),
                Value::Raw("Blue".to_owned()),
            )],
        );

        let expanded = expand_vars("%color% %color% %color%", &strings)
            .expect("expected hardcoded string to be valid");

        assert_eq!(expanded, "Blue Blue Blue".to_owned());
    }

    #[test]
    fn unterminated_strkey() {
        let strings = Section::new("Strings".to_owned(), vec![]);
        let result = expand_vars("%unterminated", &strings);

        assert!(matches!(result, Err(ExpandVarsError::Unterminated)));
    }

    #[test]
    fn case_insensitive() {
        let strings = Section::new(
            "Strings".to_owned(),
            vec![Entry::Item(
                "name".to_owned(),
                Value::Raw("Stinky".to_owned()),
            )],
        );

        let expanded =
            expand_vars("Hello, %Name%!", &strings).expect("expected hardcoded string to be valid");

        assert_eq!(expanded, "Hello, Stinky!".to_owned());
    }
}
