use std::path::Path;

use anyhow::Result;
use flip_ir::{Block, Document, Metadata};

pub fn parse(path: &Path) -> Result<Document> {
    let content = std::fs::read_to_string(path)?;
    let parsed: serde_yaml::Value = serde_yaml::from_str(&content)?;

    let mut doc = Document::new();
    doc.metadata = Metadata {
        source_format: Some("yaml".to_string()),
        ..Default::default()
    };

    let formatted = serde_yaml::to_string(&parsed)?;
    for line in formatted.lines() {
        let trimmed = line.trim().to_string();
        if !trimmed.is_empty() {
            doc.push_block(Block::paragraph(trimmed));
        }
    }

    Ok(doc)
}
