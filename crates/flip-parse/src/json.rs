use std::path::Path;

use anyhow::Result;
use flip_ir::{Block, Document, Metadata};

pub fn parse(path: &Path) -> Result<Document> {
    let content = std::fs::read_to_string(path)?;
    let parsed: serde_json::Value = serde_json::from_str(&content)?;

    let mut doc = Document::new();
    doc.metadata = Metadata {
        source_format: Some("json".to_string()),
        ..Default::default()
    };

    let formatted = serde_json::to_string_pretty(&parsed)?;
    for line in formatted.lines() {
        let trimmed = line.trim().to_string();
        if !trimmed.is_empty() {
            doc.push_block(Block::paragraph(trimmed));
        }
    }

    Ok(doc)
}
