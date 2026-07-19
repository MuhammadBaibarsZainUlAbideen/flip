use std::path::Path;

use anyhow::Result;
use flip_ir::{Block, Document, Metadata};

pub fn parse(path: &Path) -> Result<Document> {
    let content = std::fs::read_to_string(path)?;

    let mut doc = Document::new();
    doc.metadata = Metadata {
        source_format: Some("rtf".to_string()),
        ..Default::default()
    };

    let mut text = String::new();
    let mut in_control = false;

    for ch in content.chars() {
        match ch {
            '\\' => {
                in_control = true;
            }
            '{' | '}' => {}
            _ if in_control => {
                if ch.is_alphabetic() {
                    continue;
                }
                in_control = false;
            }
            _ => text.push(ch),
        }
    }

    for line in text.lines() {
        let trimmed = line.trim().to_string();
        if !trimmed.is_empty() {
            doc.push_block(Block::paragraph(trimmed));
        }
    }

    if doc.blocks.is_empty() {
        doc.push_block(Block::paragraph("(RTF content extracted as plain text)"));
    }

    Ok(doc)
}
