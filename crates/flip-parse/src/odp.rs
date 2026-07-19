use std::io::Read;
use std::path::Path;

use anyhow::Result;
use flip_ir::{Block, Document, Metadata};

pub fn parse(path: &Path) -> Result<Document> {
    let mut doc = Document::new();
    doc.metadata = Metadata {
        source_format: Some("odp".to_string()),
        ..Default::default()
    };

    let bytes = std::fs::read(path)?;
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(&bytes))?;

    for i in 0..zip.len() {
        if let Ok(mut file) = zip.by_index(i) {
            let name = file.name().to_string();
            if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
                let mut content = String::new();
                if file.read_to_string(&mut content).is_ok() {
                    let text = extract_odp_text(&content);
                    for line in text.lines() {
                        let trimmed = line.trim().to_string();
                        if !trimmed.is_empty() {
                            doc.push_block(Block::paragraph(trimmed));
                        }
                    }
                }
            }
        }
    }

    if doc.blocks.is_empty() {
        doc.push_block(Block::paragraph(
            "(No text content could be extracted from this ODP presentation)",
        ));
    }

    Ok(doc)
}

fn extract_odp_text(xml: &str) -> String {
    let mut text = String::new();
    let mut current = String::new();
    let mut in_tag = false;
    let mut in_text = false;

    for ch in xml.chars() {
        match ch {
            '<' => {
                in_tag = true;
                if in_text && !current.trim().is_empty() {
                    text.push_str(current.trim());
                    text.push(' ');
                }
                current.clear();
            }
            '>' if in_tag => {
                let tag = current.trim();
                if tag.contains("text:p") {
                    in_text = true;
                } else if tag.starts_with("/text:") {
                    if in_text {
                        text.push('\n');
                    }
                    in_text = false;
                }
                in_tag = false;
                current.clear();
            }
            _ if in_tag => current.push(ch),
            _ if in_text => current.push(ch),
            _ => {}
        }
    }

    text
}
