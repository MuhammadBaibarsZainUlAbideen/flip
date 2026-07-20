use anyhow::Result;
use flip_ir::{Block, Document, Inline};

fn cell_plain_text(cell: &[Inline]) -> String {
    cell.iter().map(|i| i.plain_text()).collect()
}

fn block_plain_text(block: &Block) -> String {
    match block {
        Block::Paragraph(inlines) => inlines.iter().map(|i| i.plain_text()).collect(),
        Block::Heading { level: _, content } => content.iter().map(|i| i.plain_text()).collect(),
        Block::List { ordered: _, items } => items
            .iter()
            .flat_map(|item| item.iter().map(|i| i.plain_text()))
            .collect(),
        Block::Code { content, .. } => content.clone(),
        Block::Table { headers, rows } => {
            let mut text = String::new();
            for row in headers {
                for cell in row {
                    text.push_str(&cell_plain_text(cell));
                    text.push('\t');
                }
                text.push('\n');
            }
            for row in rows {
                for cell in row {
                    text.push_str(&cell_plain_text(cell));
                    text.push('\t');
                }
                text.push('\n');
            }
            text
        }
        Block::Image { alt, .. } => alt.clone(),
        Block::Blockquote(inlines) => inlines.iter().map(|i| i.plain_text()).collect(),
        Block::HorizontalRule => String::new(),
        Block::TableFromCsv(content) => content.clone(),
    }
}

pub fn render(doc: &Document, path: &std::path::Path) -> Result<()> {
    let bytes = render_bytes(doc)?;
    std::fs::write(path, bytes)?;
    Ok(())
}

pub fn render_bytes(doc: &Document) -> Result<Vec<u8>> {
    let mut wtr = csv::WriterBuilder::new()
        .flexible(true)
        .from_writer(Vec::new());

    for block in &doc.blocks {
        match block {
            Block::Table { headers, rows } => {
                for row in headers {
                    let record: Vec<String> =
                        row.iter().map(|cell| cell_plain_text(cell)).collect();
                    wtr.write_record(&record)?;
                }
                for row in rows {
                    let record: Vec<String> =
                        row.iter().map(|cell| cell_plain_text(cell)).collect();
                    wtr.write_record(&record)?;
                }
            }
            _ => {
                let text = block_plain_text(block);
                if !text.trim().is_empty() {
                    let record = vec![text];
                    wtr.write_record(&record)?;
                }
            }
        }
    }

    Ok(wtr.into_inner()?)
}
