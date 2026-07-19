use std::path::Path;

use anyhow::Result;
use flip_ir::{Block, Document, Inline, Metadata};

pub fn parse(path: &Path) -> Result<Document> {
    let content = std::fs::read_to_string(path)?;
    let mut rdr = csv::Reader::from_reader(content.as_bytes());

    let mut doc = Document::new();
    doc.metadata = Metadata {
        source_format: Some("csv".to_string()),
        ..Default::default()
    };

    let header_row: Vec<Vec<Inline>> = rdr
        .headers()?
        .iter()
        .map(|h| vec![Inline::Text(h.to_string())])
        .collect();
    let headers = if header_row.is_empty() {
        Vec::new()
    } else {
        vec![header_row]
    };

    let mut rows = Vec::new();
    for result in rdr.records() {
        let record = result?;
        let row: Vec<Vec<Inline>> = record
            .iter()
            .map(|field| vec![Inline::Text(field.to_string())])
            .collect();
        rows.push(row);
    }

    if !headers.is_empty() || !rows.is_empty() {
        doc.push_block(Block::Table { headers, rows });
    }

    Ok(doc)
}
