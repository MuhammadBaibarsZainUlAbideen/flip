use std::io::Read;
use std::path::Path;

use anyhow::Result;
use flip_ir::{Block, Document, Inline, Metadata};

pub fn parse(path: &Path) -> Result<Document> {
    let mut doc = Document::new();
    doc.metadata = Metadata {
        source_format: Some("docx".to_string()),
        ..Default::default()
    };

    let bytes = std::fs::read(path)?;
    let zip = zip::ZipArchive::new(std::io::Cursor::new(&bytes))?;
    let text = extract_office_xml_text(zip, "word/document.xml");

    if text.is_empty() {
        let plain = extract_office_plain_text(&bytes)?;
        for line in plain.lines() {
            let trimmed = line.trim().to_string();
            if !trimmed.is_empty() {
                doc.push_block(Block::paragraph(trimmed));
            }
        }
    } else {
        for line in text.lines() {
            let trimmed = line.trim().to_string();
            if trimmed.is_empty() {
                continue;
            }
            if let Some(rest) = trimmed.strip_prefix("# ") {
                doc.push_block(Block::heading(1, rest));
            } else if let Some(rest) = trimmed.strip_prefix("## ") {
                doc.push_block(Block::heading(2, rest));
            } else if let Some(rest) = trimmed.strip_prefix("### ") {
                doc.push_block(Block::heading(3, rest));
            } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                doc.push_block(Block::List {
                    ordered: false,
                    items: vec![vec![Inline::Text(trimmed[2..].to_string())]],
                });
            } else {
                doc.push_block(Block::paragraph(trimmed));
            }
        }
    }

    if doc.blocks.is_empty() {
        doc.push_block(Block::paragraph(
            "(No text content could be extracted from this document)",
        ));
    }

    Ok(doc)
}

fn extract_office_xml_text(
    mut zip: zip::ZipArchive<std::io::Cursor<&Vec<u8>>>,
    xml_path: &str,
) -> String {
    let mut text = String::new();
    if let Ok(mut file) = zip.by_name(xml_path) {
        let mut content = String::new();
        if file.read_to_string(&mut content).is_ok() {
            let mut in_text = false;
            let mut current = String::new();
            for ch in content.chars() {
                match ch {
                    '<' => {
                        if !current.trim().is_empty() {
                            text.push_str(current.trim());
                            text.push(' ');
                        }
                        current.clear();
                        in_text = true;
                    }
                    '>' if in_text => {
                        in_text = false;
                        current.clear();
                    }
                    _ if in_text => {}
                    _ => {
                        current.push(ch);
                    }
                }
            }
            if !current.trim().is_empty() {
                text.push_str(current.trim());
            }
        }
    }
    text
}

fn extract_office_plain_text(bytes: &[u8]) -> Result<String> {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes))?;
    let mut text = String::new();
    for i in 0..zip.len() {
        if let Ok(mut file) = zip.by_index(i) {
            let name = file.name().to_string();
            if name.ends_with(".xml") || name.ends_with(".rels") {
                continue;
            }
            if name.contains("document") || name.contains("slide") || name.contains("content") {
                let mut content = String::new();
                if file.read_to_string(&mut content).is_ok() {
                    text.push_str(&content);
                    text.push('\n');
                }
            }
        }
    }
    Ok(text)
}
