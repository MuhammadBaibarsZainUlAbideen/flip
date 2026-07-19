use crate::Inline;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Block {
    Paragraph(Vec<Inline>),
    Heading {
        level: u8,
        content: Vec<Inline>,
    },
    Table {
        headers: Vec<Vec<Vec<Inline>>>,
        rows: Vec<Vec<Vec<Inline>>>,
    },
    List {
        ordered: bool,
        items: Vec<Vec<Inline>>,
    },
    Code {
        language: Option<String>,
        content: String,
    },
    Blockquote(Vec<Inline>),
    Image {
        src: String,
        alt: String,
        width: Option<u32>,
        height: Option<u32>,
    },
    HorizontalRule,
    TableFromCsv(String),
}

impl Block {
    pub fn paragraph(text: impl Into<String>) -> Self {
        Block::Paragraph(vec![Inline::Text(text.into())])
    }

    pub fn heading(level: u8, text: impl Into<String>) -> Self {
        Block::Heading {
            level,
            content: vec![Inline::Text(text.into())],
        }
    }

    pub fn code(content: impl Into<String>) -> Self {
        Block::Code {
            language: None,
            content: content.into(),
        }
    }

    pub fn code_with_lang(lang: impl Into<String>, content: impl Into<String>) -> Self {
        Block::Code {
            language: Some(lang.into()),
            content: content.into(),
        }
    }
}
