use std::io::Read;
use std::path::Path;

use anyhow::Result;
use flip_ir::{Block, Document, Metadata};

pub fn parse(path: &Path) -> Result<Document> {
    let mut doc = Document::new();
    doc.metadata = Metadata {
        source_format: Some("xlsx".to_string()),
        ..Default::default()
    };

    let bytes = std::fs::read(path)?;
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(&bytes))?;

    let mut texts = Vec::new();
    for i in 0..zip.len() {
        if let Ok(mut file) = zip.by_index(i) {
            let name = file.name().to_string();
            if name.starts_with("xl/worksheets/sheet") && name.ends_with(".xml") {
                let mut content = String::new();
                if file.read_to_string(&mut content).is_ok() {
                    let mut current = String::new();
                    let mut in_tag = false;
                    let mut in_v = false;
                    for ch in content.chars() {
                        match ch {
                            '<' => {
                                in_tag = true;
                                if in_v && !current.trim().is_empty() {
                                    texts.push(current.trim().to_string());
                                }
                                current.clear();
                            }
                            '>' => {
                                if current == "v" || current.starts_with("v ") {
                                    in_v = true;
                                }
                                if current.starts_with("/v") {
                                    in_v = false;
                                    if !texts.last().map_or(false, |t: &String| t.is_empty()) {
                                    }
                                }
                                in_tag = false;
                                current.clear();
                            }
                            _ if in_tag => current.push(ch),
                            _ if in_v => current.push(ch),
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    for text in &texts {
        let trimmed = text.trim().to_string();
        if !trimmed.is_empty() {
            doc.push_block(Block::paragraph(trimmed));
        }
    }

    if doc.blocks.is_empty() {
        doc.push_block(Block::paragraph(
            "(No text content could be extracted from this spreadsheet)",
        ));
    }

    Ok(doc)
}
