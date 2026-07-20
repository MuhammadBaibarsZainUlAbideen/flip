use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Inline {
    Text(String),
    Bold(Vec<Inline>),
    Italic(Vec<Inline>),
    Strikethrough(Vec<Inline>),
    Code(String),
    Link { text: Vec<Inline>, url: String },
    Image { alt: String, src: String },
    Superscript(Vec<Inline>),
    Subscript(Vec<Inline>),
}

impl Inline {
    pub fn plain_text(&self) -> String {
        match self {
            Inline::Text(s) => s.clone(),
            Inline::Bold(inner) | Inline::Italic(inner) | Inline::Strikethrough(inner) => {
                inner.iter().map(|i| i.plain_text()).collect()
            }
            Inline::Code(s) => s.clone(),
            Inline::Link { text, .. } => text.iter().map(|i| i.plain_text()).collect(),
            Inline::Image { alt, .. } => alt.clone(),
            Inline::Superscript(inner) | Inline::Subscript(inner) => {
                inner.iter().map(|i| i.plain_text()).collect()
            }
        }
    }
}
