#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    name: String,
    entries: Vec<Entry>,
}

impl Section {
    #[must_use]
    pub(crate) fn new(name: String, entries: Vec<Entry>) -> Self {
        Self { name, entries }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    pub(crate) fn push(&mut self, value: Entry) {
        self.entries.push(value);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    Item(String, Value),
    Value(Value),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Raw(String),
    List(Vec<String>),
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::Raw(value)
    }
}

impl From<Vec<String>> for Value {
    fn from(value: Vec<String>) -> Self {
        Value::List(value)
    }
}
