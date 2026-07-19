use std::path::Path;

use anyhow::Result;
use flip_ir::{Block, Document, Metadata};

pub fn parse(path: &Path) -> Result<Document> {
    let bytes = std::fs::read(path)?;
    let pdf = pdf_oxide::PdfDocument::from_bytes(bytes)?;

    let mut doc = Document::new();

    let page_count = pdf.page_count().unwrap_or(0);

    doc.metadata = Metadata {
        page_count: Some(page_count as u32),
        source_format: Some("pdf".to_string()),
        ..Default::default()
    };

    for i in 0..page_count {
        if let Ok(text) = pdf.extract_text(i) {
            for line in text.lines() {
                let line = line.trim().to_string();
                if !line.is_empty() {
                    doc.push_block(Block::paragraph(line));
                }
            }
        }
    }

    if doc.blocks.is_empty() {
        doc.push_block(Block::paragraph(
            "(No text content could be extracted from this PDF)",
        ));
    }

    Ok(doc)
}
