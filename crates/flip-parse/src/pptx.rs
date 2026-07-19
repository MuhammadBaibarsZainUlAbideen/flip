use std::io::Read;
use std::path::Path;

use anyhow::Result;
use flip_ir::{Block, Document, Metadata};

pub fn parse(path: &Path) -> Result<Document> {
    let mut doc = Document::new();
    doc.metadata = Metadata {
        source_format: Some("pptx".to_string()),
        ..Default::default()
    };

    let bytes = std::fs::read(path)?;
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(&bytes))?;

    let mut slide_files: Vec<String> = Vec::new();
    for i in 0..zip.len() {
        let name = zip.by_index(i)?.name().to_string();
        if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
            slide_files.push(name);
        }
    }
    slide_files.sort();

    for slide_name in &slide_files {
        if let Ok(mut file) = zip.by_name(slide_name) {
            let mut content = String::new();
            if file.read_to_string(&mut content).is_ok() {
                let text = extract_xml_text(&content);
                for line in text.lines() {
                    let trimmed = line.trim().to_string();
                    if trimmed.is_empty() {
                        continue;
                    }
                    if trimmed.starts_with("# ") {
                        doc.push_block(Block::heading(1, &trimmed[2..]));
                    } else {
                        doc.push_block(Block::paragraph(trimmed));
                    }
                }
            }
        }
    }

    if doc.blocks.is_empty() {
        doc.push_block(Block::paragraph(
            "(No text content could be extracted from this presentation)",
        ));
    }

    Ok(doc)
}

fn extract_xml_text(xml: &str) -> String {
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
                let tag = current.trim().to_string();
                if tag.starts_with("a:t") || tag == "a:t" {
                    in_text = true;
                } else if tag.starts_with("/a:t") || tag == "/a:t" {
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

    if in_text && !current.trim().is_empty() {
        text.push_str(current.trim());
    }

    text
}
