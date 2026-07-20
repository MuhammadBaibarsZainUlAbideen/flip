use std::path::Path;

use anyhow::Result;
use flip_ir::{Block, Document};

pub fn render(doc: &Document, path: &Path) -> Result<()> {
    let mut workbook = umya_spreadsheet::new_file();

    let sheet = workbook
        .get_sheet_by_name_mut("Sheet1")
        .ok_or_else(|| anyhow::anyhow!("Sheet1 not found"))?;

    let mut row_num = 1u32;

    for block in &doc.blocks {
        match block {
            Block::Table { headers, rows } => {
                for header_row in headers {
                    for (col, cell) in header_row.iter().enumerate() {
                        let text: String = cell.iter().map(|i| i.plain_text()).collect();
                        let addr = format!("{}{}", col_to_letter(col as u32 + 1), row_num);
                        sheet.get_cell_mut(addr).set_value(text);
                    }
                    row_num += 1;
                }

                for row in rows {
                    for (col, cell) in row.iter().enumerate() {
                        let text: String = cell.iter().map(|i| i.plain_text()).collect();
                        let addr = format!("{}{}", col_to_letter(col as u32 + 1), row_num);
                        sheet.get_cell_mut(addr).set_value(text);
                    }
                    row_num += 1;
                }
            }
            _ => {
                let text = block_plain_text(block);
                if !text.trim().is_empty() {
                    let addr = format!("A{}", row_num);
                    sheet.get_cell_mut(addr).set_value(text);
                    row_num += 1;
                }
            }
        }
    }

    let path = std::path::Path::new(path);
    let _ = umya_spreadsheet::writer::xlsx::write(&workbook, path);

    Ok(())
}

fn col_to_letter(col: u32) -> String {
    let mut result = String::new();
    let mut c = col;
    while c > 0 {
        c -= 1;
        result.push((b'A' + (c % 26) as u8) as char);
        c /= 26;
    }
    result.chars().rev().collect()
}

fn block_plain_text(block: &Block) -> String {
    match block {
        Block::Paragraph(inlines) => inlines.iter().map(|i| i.plain_text()).collect(),
        Block::Heading { content, .. } => content.iter().map(|i| i.plain_text()).collect(),
        Block::List { items, .. } => items
            .iter()
            .map(|item| item.iter().map(|i| i.plain_text()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n"),
        Block::Code { content, .. } => content.clone(),
        Block::Blockquote(content) => content.iter().map(|i| i.plain_text()).collect(),
        _ => String::new(),
    }
}
