use std::path::Path;

use anyhow::Result;
use flip_ir::{Block, Document, Metadata};
use image::GenericImageView;

pub fn parse(path: &Path) -> Result<Document> {
    let img = image::open(path)?;

    let mut doc = Document::new();

    let (width, height) = img.dimensions();
    let filename = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    doc.metadata = Metadata {
        source_format: Some("image".to_string()),
        custom: {
            let mut m = std::collections::HashMap::new();
            m.insert("width".to_string(), width.to_string());
            m.insert("height".to_string(), height.to_string());
            m
        },
        ..Default::default()
    };

    doc.push_block(Block::Image {
        src: path.display().to_string(),
        alt: filename,
        width: Some(width),
        height: Some(height),
    });

    doc.push_block(Block::paragraph(format!(
        "Image: {}x{} pixels",
        width, height
    )));

    Ok(doc)
}
