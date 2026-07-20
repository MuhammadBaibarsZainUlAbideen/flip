use std::io::Read;
use std::path::Path;

use anyhow::Result;
use flip_ir::{Block, Document, Metadata};

pub fn parse(path: &Path) -> Result<Document> {
    let mut doc = Document::new();
    doc.metadata = Metadata {
        source_format: Some("epub".to_string()),
        ..Default::default()
    };

    let bytes = std::fs::read(path)?;
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(&bytes))?;

    let mut xhtml_files: Vec<String> = Vec::new();
    for i in 0..zip.len() {
        let name = zip.by_index(i)?.name().to_string();
        if name.ends_with(".xhtml") || name.ends_with(".html") || name.ends_with(".htm") {
            xhtml_files.push(name);
        }
    }
    xhtml_files.sort();

    for file_name in &xhtml_files {
        if let Ok(mut file) = zip.by_name(file_name) {
            let mut content = String::new();
            if file.read_to_string(&mut content).is_ok() {
                let text = strip_html_tags(&content);
                for line in text.lines() {
                    let trimmed = line.trim().to_string();
                    if !trimmed.is_empty() {
                        doc.push_block(Block::paragraph(trimmed));
                    }
                }
            }
        }
    }

    if doc.blocks.is_empty() {
        doc.push_block(Block::paragraph(
            "(No text content could be extracted from this EPUB)",
        ));
    }

    Ok(doc)
}

fn strip_html_tags(html: &str) -> String {
    let mut text = String::new();
    let mut in_tag = false;

    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if in_tag => continue,
            _ => text.push(ch),
        }
    }

    text.split_whitespace().collect::<Vec<&str>>().join(" ")
}
