use std::path::Path;

use anyhow::Result;
use flip_ir::{Block, Document, Metadata};

pub fn parse(path: &Path) -> Result<Document> {
    let content = std::fs::read_to_string(path)?;

    let mut doc = Document::new();
    doc.metadata = Metadata {
        source_format: Some("svg".to_string()),
        ..Default::default()
    };

    doc.push_block(Block::paragraph("SVG Document"));

    let mut in_tag = false;
    let mut text_buf = String::new();
    for ch in content.chars() {
        match ch {
            '<' => {
                in_tag = true;
            }
            '>' => {
                in_tag = false;
                continue;
            }
            _ if in_tag => continue,
            _ => text_buf.push(ch),
        }
    }

    for line in text_buf.lines() {
        let trimmed = line.trim().to_string();
        if !trimmed.is_empty() {
            doc.push_block(Block::paragraph(trimmed));
        }
    }

    if doc.blocks.len() <= 1 {
        doc.push_block(Block::Code {
            language: Some("svg".to_string()),
            content: content,
        });
    }

    Ok(doc)
}
