use std::path::Path;

use anyhow::Result;
use flip_ir::{Block, Document};

pub fn render(doc: &Document, path: &Path) -> Result<()> {
    let bytes = render_bytes(doc)?;
    std::fs::write(path, bytes)?;
    Ok(())
}

pub fn render_bytes(doc: &Document) -> Result<Vec<u8>> {
    let mut text = String::new();

    if let Some(ref title) = doc.metadata.title {
        text.push_str(title);
        text.push_str("\n");
        text.push_str(&"=".repeat(title.len()));
        text.push_str("\n\n");
    }

    for block in &doc.blocks {
        render_block(&mut text, block);
    }

    Ok(text.into_bytes())
}

fn render_block(text: &mut String, block: &Block) {
    match block {
        Block::Paragraph(inlines) => {
            let t: String = inlines.iter().map(|i| i.plain_text()).collect();
            text.push_str(&t);
            text.push_str("\n\n");
        }
        Block::Heading { level, content } => {
            let t: String = content.iter().map(|i| i.plain_text()).collect();
            let prefix = "#".repeat(*level as usize);
            text.push_str(&format!("{} {}\n\n", prefix, t));
        }
        Block::Code { content, .. } => {
            text.push_str(content);
            text.push_str("\n\n");
        }
        Block::List { ordered, items } => {
            for (i, item) in items.iter().enumerate() {
                let t: String = item.iter().map(|i| i.plain_text()).collect();
                if *ordered {
                    text.push_str(&format!("{}. {}\n", i + 1, t));
                } else {
                    text.push_str(&format!("- {}\n", t));
                }
            }
            text.push('\n');
        }
        Block::Blockquote(content) => {
            let t: String = content.iter().map(|i| i.plain_text()).collect();
            text.push_str(&format!("> {}\n\n", t));
        }
        Block::Table { headers, rows } => {
            for row in headers {
                let row_strs: Vec<String> = row
                    .iter()
                    .map(|cell| cell.iter().map(|i| i.plain_text()).collect())
                    .collect();
                text.push_str(&row_strs.join("\t"));
                text.push('\n');
            }
            if !headers.is_empty() {
                text.push_str(&"-".repeat(40));
                text.push('\n');
            }
            for row in rows {
                let row_strs: Vec<String> = row
                    .iter()
                    .map(|cell| cell.iter().map(|i| i.plain_text()).collect())
                    .collect();
                text.push_str(&row_strs.join("\t"));
                text.push('\n');
            }
            text.push('\n');
        }
        Block::Image { alt, .. } => {
            text.push_str(&format!("[Image: {}]\n\n", alt));
        }
        Block::HorizontalRule => {
            text.push_str(&"-".repeat(40));
            text.push_str("\n\n");
        }
        Block::TableFromCsv(csv_text) => {
            text.push_str(csv_text);
            text.push_str("\n\n");
        }
    }
}
