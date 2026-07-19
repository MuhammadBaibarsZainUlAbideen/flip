use std::path::Path;

use anyhow::Result;
use flip_ir::{Block, Document, Metadata};

pub fn parse(path: &Path) -> Result<Document> {
    let content = std::fs::read_to_string(path)?;

    let mut doc = Document::new();
    doc.metadata = Metadata {
        source_format: Some("text".to_string()),
        ..Default::default()
    };

    for line in content.lines() {
        let trimmed = line.trim().to_string();
        if !trimmed.is_empty() {
            doc.push_block(Block::paragraph(trimmed));
        }
    }

    Ok(doc)
}
