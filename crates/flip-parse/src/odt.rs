use std::io::Read;
use std::path::Path;

use anyhow::Result;
use flip_ir::{Block, Document, Metadata};

pub fn parse(path: &Path) -> Result<Document> {
    let mut doc = Document::new();
    doc.metadata = Metadata {
        source_format: Some("odt".to_string()),
        ..Default::default()
    };

    let bytes = std::fs::read(path)?;
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(&bytes))?;

    if let Ok(mut file) = zip.by_name("content.xml") {
        let mut content = String::new();
        if file.read_to_string(&mut content).is_ok() {
            let text = extract_odf_text(&content);
            for line in text.lines() {
                let l = line.trim().to_string();
                if !l.is_empty() {
                    doc.push_block(Block::paragraph(l));
                }
            }
        }
    }

    if doc.blocks.is_empty() {
        doc.push_block(Block::paragraph(
            "(No text content could be extracted from this ODT document)",
        ));
    }

    Ok(doc)
}

fn extract_odf_text(xml: &str) -> String {
    let mut text = String::new();
    let mut current = String::new();
    let mut in_tag = false;
    let mut in_text_elem = false;

    for ch in xml.chars() {
        match ch {
            '<' => {
                in_tag = true;
                if in_text_elem && !current.trim().is_empty() {
                    text.push_str(current.trim());
                    text.push(' ');
                }
                current.clear();
            }
            '>' if in_tag => {
                let tag = current.trim();
                if tag.contains("text:p") || tag.contains("text:h") {
                    in_text_elem = true;
                } else if tag.starts_with("/text:") {
                    if in_text_elem {
                        text.push('\n');
                    }
                    in_text_elem = false;
                }
                in_tag = false;
                current.clear();
            }
            _ if in_tag => current.push(ch),
            _ if in_text_elem => current.push(ch),
            _ => {}
        }
    }

    text
}
