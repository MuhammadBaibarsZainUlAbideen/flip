use std::io::Cursor;
use std::path::Path;

use anyhow::Result;
use flip_ir::{Block, Document, Inline};

pub fn render(doc: &Document, path: &Path) -> Result<()> {
    let bytes = render_bytes(doc)?;
    std::fs::write(path, bytes)?;
    Ok(())
}

pub fn render_bytes(doc: &Document) -> Result<Vec<u8>> {
    let mut builder = docx_rs::Docx::new();

    for block in &doc.blocks {
        match block {
            Block::Paragraph(inlines) => {
                let mut para = docx_rs::Paragraph::new();
                para = add_inlines_to_para(para, inlines);
                builder = builder.add_paragraph(para);
            }
            Block::Heading { level, content } => {
                let mut para = docx_rs::Paragraph::new();
                let style = match level {
                    1 => "Heading1",
                    2 => "Heading2",
                    3 => "Heading3",
                    4 => "Heading4",
                    5 => "Heading5",
                    _ => "Heading6",
                };
                para = para.style(style);
                para = add_inlines_to_para(para, content);
                builder = builder.add_paragraph(para);
            }
            Block::List { items, .. } => {
                for item in items {
                    let mut para = docx_rs::Paragraph::new();
                    let text: String = item.iter().map(|i| i.plain_text()).collect();
                    para = para.add_run(docx_rs::Run::new().add_text(text));
                    builder = builder.add_paragraph(para);
                }
            }
            Block::Code { content, .. } => {
                let mut para = docx_rs::Paragraph::new();
                para = para.add_run(
                    docx_rs::Run::new().add_text(content).fonts(
                        docx_rs::RunFonts::new()
                            .ascii("Courier New")
                            .hi_ansi("Courier New"),
                    ),
                );
                builder = builder.add_paragraph(para);
            }
            Block::Blockquote(content) => {
                let mut para = docx_rs::Paragraph::new();
                para = add_inlines_to_para(para, content);
                builder = builder.add_paragraph(para);
            }
            Block::Table { headers, rows } => {
                let mut table = docx_rs::Table::new(vec![]);
                let all_rows: Vec<&Vec<Vec<Inline>>> = headers.iter().chain(rows.iter()).collect();

                for row in &all_rows {
                    let cells: Vec<docx_rs::TableCell> = row
                        .iter()
                        .map(|cell| {
                            let text: String = cell.iter().map(|i| i.plain_text()).collect();
                            docx_rs::TableCell::new().add_paragraph(
                                docx_rs::Paragraph::new()
                                    .add_run(docx_rs::Run::new().add_text(text)),
                            )
                        })
                        .collect();
                    table = table.add_row(docx_rs::TableRow::new(cells));
                }
                builder = builder.add_table(table);
            }
            Block::HorizontalRule => {
                builder = builder.add_paragraph(
                    docx_rs::Paragraph::new().add_run(docx_rs::Run::new().add_text("─".repeat(50))),
                );
            }
            Block::Image { alt, src, .. } => {
                let mut para = docx_rs::Paragraph::new();
                para = para
                    .add_run(docx_rs::Run::new().add_text(format!("[Image: {} - {}]", alt, src)));
                builder = builder.add_paragraph(para);
            }
            Block::TableFromCsv(text) => {
                let mut para = docx_rs::Paragraph::new();
                para = para.add_run(docx_rs::Run::new().add_text(text));
                builder = builder.add_paragraph(para);
            }
        }
    }

    let mut cursor = Cursor::new(Vec::new());
    builder.build().pack(&mut cursor)?;

    Ok(cursor.into_inner())
}

fn add_inlines_to_para(mut para: docx_rs::Paragraph, inlines: &[Inline]) -> docx_rs::Paragraph {
    for inline in inlines {
        match inline {
            Inline::Text(text) => {
                para = para.add_run(docx_rs::Run::new().add_text(text));
            }
            Inline::Bold(content) => {
                let text: String = content.iter().map(|i| i.plain_text()).collect();
                para = para.add_run(
                    docx_rs::Run::new()
                        .add_text(text)
                        .bold()
                        .fonts(docx_rs::RunFonts::new().ascii("Arial").hi_ansi("Arial")),
                );
            }
            Inline::Italic(content) => {
                let text: String = content.iter().map(|i| i.plain_text()).collect();
                para = para.add_run(
                    docx_rs::Run::new()
                        .add_text(text)
                        .italic()
                        .fonts(docx_rs::RunFonts::new().ascii("Arial").hi_ansi("Arial")),
                );
            }
            Inline::Code(text) => {
                para = para.add_run(
                    docx_rs::Run::new().add_text(text).fonts(
                        docx_rs::RunFonts::new()
                            .ascii("Courier New")
                            .hi_ansi("Courier New"),
                    ),
                );
            }
            Inline::Link { text, .. } => {
                let text_str: String = text.iter().map(|i| i.plain_text()).collect();
                para = para.add_run(docx_rs::Run::new().add_text(text_str));
            }
            _ => {
                let text = inline.plain_text();
                if !text.is_empty() {
                    para = para.add_run(docx_rs::Run::new().add_text(text));
                }
            }
        }
    }
    para
}
