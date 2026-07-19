use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Metadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub creator: Option<String>,
    pub producer: Option<String>,
    pub date: Option<String>,
    pub page_count: Option<u32>,
    pub word_count: Option<u32>,
    pub char_count: Option<u64>,
    pub source_format: Option<String>,
    pub custom: std::collections::HashMap<String, String>,
}

impl Metadata {
    pub fn new() -> Self {
        Self::default()
    }
}
