use std::path::Path;

use anyhow::Result;
use flip_ir::{Block, Document, Inline};
use serde_json::{json, Value};

pub fn render(doc: &Document, path: &Path) -> Result<()> {
    let bytes = render_bytes(doc)?;
    std::fs::write(path, bytes)?;
    Ok(())
}

pub fn render_bytes(doc: &Document) -> Result<Vec<u8>> {
    let mut content_blocks = Vec::new();

    for block in &doc.blocks {
        content_blocks.push(block_to_json(block));
    }

    let output = json!({
        "metadata": {
            "title": doc.metadata.title,
            "author": doc.metadata.author,
            "source_format": doc.metadata.source_format,
        },
        "blocks": content_blocks,
    });

    Ok(serde_json::to_string_pretty(&output)?.into_bytes())
}

fn block_to_json(block: &Block) -> Value {
    match block {
        Block::Paragraph(inlines) => json!({
            "type": "paragraph",
            "content": inlines_to_json(inlines),
        }),
        Block::Heading { level, content } => json!({
            "type": "heading",
            "level": level,
            "content": inlines_to_json(content),
        }),
        Block::Code { language, content } => json!({
            "type": "code",
            "language": language,
            "content": content,
        }),
        Block::List { ordered, items } => {
            let item_values: Vec<Value> = items
                .iter()
                .map(|item| inlines_to_json(item))
                .collect();
            json!({
                "type": "list",
                "ordered": ordered,
                "items": item_values,
            })
        }
        Block::Table { headers, rows } => {
            let header_values: Vec<Value> = headers
                .iter()
                .map(|row| {
                    let cell_values: Vec<Value> = row.iter().map(|cell| inlines_to_json(cell)).collect();
                    json!(cell_values)
                })
                .collect();
            let row_values: Vec<Value> = rows
                .iter()
                .map(|row| {
                    let cell_values: Vec<Value> = row.iter().map(|cell| inlines_to_json(cell)).collect();
                    json!(cell_values)
                })
                .collect();
            json!({
                "type": "table",
                "headers": header_values,
                "rows": row_values,
            })
        }
        Block::Blockquote(content) => json!({
            "type": "blockquote",
            "content": inlines_to_json(content),
        }),
        Block::Image { src, alt, .. } => json!({
            "type": "image",
            "src": src,
            "alt": alt,
        }),
        Block::HorizontalRule => json!({
            "type": "horizontal_rule",
        }),
        Block::TableFromCsv(text) => json!({
            "type": "csv",
            "content": text,
        }),
    }
}

fn inlines_to_json(inlines: &[Inline]) -> Value {
    let values: Vec<Value> = inlines
        .iter()
        .map(|inline| match inline {
            Inline::Text(text) => json!({"type": "text", "value": text}),
            Inline::Bold(content) => json!({"type": "bold", "content": inlines_to_json(content)}),
            Inline::Italic(content) => {
                json!({"type": "italic", "content": inlines_to_json(content)})
            }
            Inline::Strikethrough(content) => {
                json!({"type": "strikethrough", "content": inlines_to_json(content)})
            }
            Inline::Code(text) => json!({"type": "code", "value": text}),
            Inline::Link { text, url } => {
                json!({"type": "link", "text": inlines_to_json(text), "url": url})
            }
            Inline::Image { alt, src } => json!({"type": "image", "alt": alt, "src": src}),
            Inline::Superscript(content) => {
                json!({"type": "superscript", "content": inlines_to_json(content)})
            }
            Inline::Subscript(content) => {
                json!({"type": "subscript", "content": inlines_to_json(content)})
            }
        })
        .collect();
    json!(values)
}
