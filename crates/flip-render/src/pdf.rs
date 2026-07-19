use std::path::Path;

use anyhow::Result;
use flip_ir::{Block, Document, Inline};
use printpdf::*;

const PAGE_WIDTH_MM: f32 = 210.0;
const PAGE_HEIGHT_MM: f32 = 297.0;
const MARGIN_MM: f32 = 20.0;
const LINE_HEIGHT: f32 = 14.0;

pub fn render(doc: &Document, path: &Path) -> Result<()> {
    let bytes = render_bytes(doc)?;
    std::fs::write(path, bytes)?;
    Ok(())
}

pub fn render_bytes(doc: &Document) -> Result<Vec<u8>> {
    let mut pdf_doc = PdfDocument::new("flip output");

    let mut pages: Vec<PdfPage> = Vec::new();
    let mut ops: Vec<Op> = Vec::new();
    let mut y_pos: f32 = PAGE_HEIGHT_MM - MARGIN_MM;

    for block in &doc.blocks {
        if y_pos < MARGIN_MM + 10.0 {
            pages.push(PdfPage::new(
                Mm(PAGE_WIDTH_MM),
                Mm(PAGE_HEIGHT_MM),
                std::mem::take(&mut ops),
            ));
            y_pos = PAGE_HEIGHT_MM - MARGIN_MM;
        }

        match block {
            Block::Paragraph(inlines) => {
                let text = inlines_to_plain(inlines);
                let font_size = 11.0;

                for line in text.lines() {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    if y_pos < MARGIN_MM + 10.0 {
                        pages.push(PdfPage::new(
                            Mm(PAGE_WIDTH_MM),
                            Mm(PAGE_HEIGHT_MM),
                            std::mem::take(&mut ops),
                        ));
                        y_pos = PAGE_HEIGHT_MM - MARGIN_MM;
                    }
                    ops.push(Op::StartTextSection);
                    ops.push(Op::SetFontSizeBuiltinFont {
                        size: Pt(font_size),
                        font: BuiltinFont::Helvetica,
                    });
                    ops.push(Op::SetTextCursor {
                        pos: Point {
                            x: Mm(MARGIN_MM).into(),
                            y: Mm(y_pos).into(),
                        },
                    });
                    ops.push(Op::WriteTextBuiltinFont {
                        items: vec![TextItem::Text(line.to_string())],
                        font: BuiltinFont::Helvetica,
                    });
                    ops.push(Op::EndTextSection);
                    y_pos -= LINE_HEIGHT;
                }
                y_pos -= 4.0;
            }
            Block::Heading { level, content } => {
                let text = inlines_to_plain(content);
                let font_size = match level {
                    1 => 20.0,
                    2 => 16.0,
                    3 => 13.0,
                    _ => 11.0,
                };
                let line_height = font_size * 1.4;

                if y_pos < MARGIN_MM + line_height {
                    pages.push(PdfPage::new(
                        Mm(PAGE_WIDTH_MM),
                        Mm(PAGE_HEIGHT_MM),
                        std::mem::take(&mut ops),
                    ));
                    y_pos = PAGE_HEIGHT_MM - MARGIN_MM;
                }
                ops.push(Op::StartTextSection);
                ops.push(Op::SetFontSizeBuiltinFont {
                    size: Pt(font_size),
                    font: BuiltinFont::HelveticaBold,
                });
                ops.push(Op::SetTextCursor {
                    pos: Point {
                        x: Mm(MARGIN_MM).into(),
                        y: Mm(y_pos).into(),
                    },
                });
                ops.push(Op::WriteTextBuiltinFont {
                    items: vec![TextItem::Text(text)],
                    font: BuiltinFont::HelveticaBold,
                });
                ops.push(Op::EndTextSection);
                y_pos -= line_height;
                y_pos -= 4.0;
            }
            Block::Code { content, .. } => {
                let font_size = 9.0;
                let line_height = 12.0;

                for line in content.lines() {
                    if y_pos < MARGIN_MM + 10.0 {
                        pages.push(PdfPage::new(
                            Mm(PAGE_WIDTH_MM),
                            Mm(PAGE_HEIGHT_MM),
                            std::mem::take(&mut ops),
                        ));
                        y_pos = PAGE_HEIGHT_MM - MARGIN_MM;
                    }
                    ops.push(Op::StartTextSection);
                    ops.push(Op::SetFontSizeBuiltinFont {
                        size: Pt(font_size),
                        font: BuiltinFont::Courier,
                    });
                    ops.push(Op::SetTextCursor {
                        pos: Point {
                            x: Mm(MARGIN_MM + 5.0).into(),
                            y: Mm(y_pos).into(),
                        },
                    });
                    ops.push(Op::WriteTextBuiltinFont {
                        items: vec![TextItem::Text(line.to_string())],
                        font: BuiltinFont::Courier,
                    });
                    ops.push(Op::EndTextSection);
                    y_pos -= line_height;
                }
                y_pos -= 8.0;
            }
            Block::Blockquote(content) => {
                let text = inlines_to_plain(content);
                let font_size = 11.0;

                if y_pos < MARGIN_MM + 10.0 {
                    pages.push(PdfPage::new(
                        Mm(PAGE_WIDTH_MM),
                        Mm(PAGE_HEIGHT_MM),
                        std::mem::take(&mut ops),
                    ));
                    y_pos = PAGE_HEIGHT_MM - MARGIN_MM;
                }
                ops.push(Op::StartTextSection);
                ops.push(Op::SetFontSizeBuiltinFont {
                    size: Pt(font_size),
                    font: BuiltinFont::HelveticaOblique,
                });
                ops.push(Op::SetTextCursor {
                    pos: Point {
                        x: Mm(MARGIN_MM + 10.0).into(),
                        y: Mm(y_pos).into(),
                    },
                });
                ops.push(Op::WriteTextBuiltinFont {
                    items: vec![TextItem::Text(text)],
                    font: BuiltinFont::HelveticaOblique,
                });
                ops.push(Op::EndTextSection);
                y_pos -= LINE_HEIGHT;
                y_pos -= 4.0;
            }
            Block::List { items, .. } => {
                let font_size = 11.0;

                for (i, item) in items.iter().enumerate() {
                    let text = inlines_to_plain(item);
                    let prefix = format!("{}. {}", i + 1, text);
                    if y_pos < MARGIN_MM + 10.0 {
                        pages.push(PdfPage::new(
                            Mm(PAGE_WIDTH_MM),
                            Mm(PAGE_HEIGHT_MM),
                            std::mem::take(&mut ops),
                        ));
                        y_pos = PAGE_HEIGHT_MM - MARGIN_MM;
                    }
                    ops.push(Op::StartTextSection);
                    ops.push(Op::SetFontSizeBuiltinFont {
                        size: Pt(font_size),
                        font: BuiltinFont::Helvetica,
                    });
                    ops.push(Op::SetTextCursor {
                        pos: Point {
                            x: Mm(MARGIN_MM + 5.0).into(),
                            y: Mm(y_pos).into(),
                        },
                    });
                    ops.push(Op::WriteTextBuiltinFont {
                        items: vec![TextItem::Text(prefix)],
                        font: BuiltinFont::Helvetica,
                    });
                    ops.push(Op::EndTextSection);
                    y_pos -= LINE_HEIGHT;
                }
                y_pos -= 4.0;
            }
            Block::HorizontalRule => {
                y_pos -= 4.0;
                let line = Line {
                    points: vec![
                        LinePoint {
                            p: Point {
                                x: Mm(MARGIN_MM).into(),
                                y: Mm(y_pos).into(),
                            },
                            bezier: false,
                        },
                        LinePoint {
                            p: Point {
                                x: Mm(PAGE_WIDTH_MM - MARGIN_MM).into(),
                                y: Mm(y_pos).into(),
                            },
                            bezier: false,
                        },
                    ],
                    is_closed: false,
                };
                ops.push(Op::SetOutlineColor {
                    col: Color::Rgb(Rgb {
                        r: 0.5,
                        g: 0.5,
                        b: 0.5,
                        icc_profile: None,
                    }),
                });
                ops.push(Op::SetOutlineThickness { pt: Pt(0.5) });
                ops.push(Op::DrawLine { line });
                y_pos -= 8.0;
            }
            Block::Table { headers, rows } => {
                let font_size = 9.0;
                let line_height = 12.0;

                let mut all_rows: Vec<&Vec<Vec<Inline>>> = Vec::new();
                for h in headers {
                    all_rows.push(h);
                }
                for r in rows {
                    all_rows.push(r);
                }

                for row in &all_rows {
                    let mut x = MARGIN_MM;
                    for cell in *row {
                        let text = inlines_to_plain(cell);
                        if y_pos < MARGIN_MM + 10.0 {
                            pages.push(PdfPage::new(
                                Mm(PAGE_WIDTH_MM),
                                Mm(PAGE_HEIGHT_MM),
                                std::mem::take(&mut ops),
                            ));
                            y_pos = PAGE_HEIGHT_MM - MARGIN_MM;
                        }
                        ops.push(Op::StartTextSection);
                        ops.push(Op::SetFontSizeBuiltinFont {
                            size: Pt(font_size),
                            font: BuiltinFont::Helvetica,
                        });
                        ops.push(Op::SetTextCursor {
                            pos: Point {
                                x: Mm(x).into(),
                                y: Mm(y_pos).into(),
                            },
                        });
                        ops.push(Op::WriteTextBuiltinFont {
                            items: vec![TextItem::Text(text)],
                            font: BuiltinFont::Helvetica,
                        });
                        ops.push(Op::EndTextSection);
                        x += 30.0;
                    }
                    y_pos -= line_height;
                }
                y_pos -= 8.0;
            }
            Block::Image { alt, .. } => {
                let font_size = 10.0;
                let placeholder = format!("[Image: {}]", alt);
                if y_pos < MARGIN_MM + 10.0 {
                    pages.push(PdfPage::new(
                        Mm(PAGE_WIDTH_MM),
                        Mm(PAGE_HEIGHT_MM),
                        std::mem::take(&mut ops),
                    ));
                    y_pos = PAGE_HEIGHT_MM - MARGIN_MM;
                }
                ops.push(Op::StartTextSection);
                ops.push(Op::SetFontSizeBuiltinFont {
                    size: Pt(font_size),
                    font: BuiltinFont::Helvetica,
                });
                ops.push(Op::SetTextCursor {
                    pos: Point {
                        x: Mm(MARGIN_MM).into(),
                        y: Mm(y_pos).into(),
                    },
                });
                ops.push(Op::WriteTextBuiltinFont {
                    items: vec![TextItem::Text(placeholder)],
                    font: BuiltinFont::Helvetica,
                });
                ops.push(Op::EndTextSection);
                y_pos -= 14.0;
            }
            Block::TableFromCsv(text) => {
                let font_size = 9.0;
                let line_height = 12.0;

                for line in text.lines() {
                    if y_pos < MARGIN_MM + 10.0 {
                        pages.push(PdfPage::new(
                            Mm(PAGE_WIDTH_MM),
                            Mm(PAGE_HEIGHT_MM),
                            std::mem::take(&mut ops),
                        ));
                        y_pos = PAGE_HEIGHT_MM - MARGIN_MM;
                    }
                    ops.push(Op::StartTextSection);
                    ops.push(Op::SetFontSizeBuiltinFont {
                        size: Pt(font_size),
                        font: BuiltinFont::Courier,
                    });
                    ops.push(Op::SetTextCursor {
                        pos: Point {
                            x: Mm(MARGIN_MM).into(),
                            y: Mm(y_pos).into(),
                        },
                    });
                    ops.push(Op::WriteTextBuiltinFont {
                        items: vec![TextItem::Text(line.to_string())],
                        font: BuiltinFont::Courier,
                    });
                    ops.push(Op::EndTextSection);
                    y_pos -= line_height;
                }
                y_pos -= 8.0;
            }
        }
    }

    if !ops.is_empty() || pages.is_empty() {
        pages.push(PdfPage::new(
            Mm(PAGE_WIDTH_MM),
            Mm(PAGE_HEIGHT_MM),
            ops,
        ));
    }

    pdf_doc.with_pages(pages);
    let mut warnings = Vec::new();
    let bytes = pdf_doc.save(&PdfSaveOptions::default(), &mut warnings);
    Ok(bytes)
}

fn inlines_to_plain(inlines: &[Inline]) -> String {
    inlines.iter().map(|i| i.plain_text()).collect()
}
