#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    name: String,
    entries: Vec<Entry>,
}

impl Section {
    #[must_use]
    pub const fn new(name: String, entries: Vec<Entry>) -> Self {
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

    #[must_use]
    pub fn as_add_registry_section(&self) -> AddRegistry<'_> {
        AddRegistry { inner: self }
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

/// Section whose name was a value to the `AddReg` directive.
///
/// This is a convenience struct that allows us to make assumptions about our entries.
pub struct AddRegistry<'a> {
    inner: &'a Section,
}

#[derive(Debug, Clone, Copy)]
pub struct AddRegistryEntry<'a> {
    pub registry_root: &'a str,
    pub subkey: &'a str,
    pub entry_name: &'a str,
    pub flags: &'a str,
    pub value: &'a str,
    pub additional: &'a [String],
}

impl<'a> TryFrom<&'a [String]> for AddRegistryEntry<'a> {
    type Error = InvalidAddRegistryEntry;

    fn try_from(value: &'a [String]) -> Result<Self, Self::Error> {
        let [
            registry_root,
            subkey,
            value_entry_name,
            flags,
            value,
            additional @ ..,
        ] = value
        else {
            return Err(InvalidAddRegistryEntry);
        };

        Ok(Self {
            registry_root,
            subkey,
            entry_name: value_entry_name,
            flags,
            value,
            additional,
        })
    }
}

#[derive(Debug, thiserror::Error)]
#[error(
    "invalid add registry entry: \
    https://learn.microsoft.com/en-us/windows-hardware/drivers/install/inf-addreg-directive"
)]
pub struct InvalidAddRegistryEntry;

impl<'a> AddRegistry<'a> {
    #[must_use]
    pub fn entries(&self) -> Vec<Result<AddRegistryEntry<'a>, InvalidAddRegistryEntry>> {
        self.inner
            .entries
            .iter()
            .map(|entry| {
                let Entry::Value(Value::List(values)) = entry else {
                    return Err(InvalidAddRegistryEntry);
                };

                AddRegistryEntry::try_from(values.as_slice())
            })
            .collect()
    }
}
