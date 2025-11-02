use std::path::PathBuf;

#[derive(Clone)]
pub enum SourceType {
    RustCode,
    PythonCode,
    // JavaScriptCode,
    // TypeScriptCode,
    // HTMLCode,
    // CSSCode,
    // JSONCode,
    // YAMLCode,
    // XMLCode,
    // MarkdownCode,
    // TextCode,
    // OtherCode,
    Text,
    Image,
    Other(String),
}

#[derive(Clone)]
pub struct Source {
    pub path: PathBuf,
    pub source_type: SourceType,
    pub content: String,
}

pub enum ReferenceType {
    Imports,
    Uses,
    Implements,
}

pub struct Reference {
    pub source: Source,
    pub reference_type: ReferenceType,
    pub route: PathBuf,
}
