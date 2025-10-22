#[derive(Debug, Clone)]
pub struct Metadata {
    title: Option<String>,
    author: Option<String>,
}

impl Metadata {
    pub const fn new(title: Option<String>, author: Option<String>) -> Self {
        Self { title, author }
    }

    /// The name of the cursor, if available.
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// The author of the cursor, if available.
    pub fn author(&self) -> Option<&str> {
        self.author.as_deref()
    }
}
