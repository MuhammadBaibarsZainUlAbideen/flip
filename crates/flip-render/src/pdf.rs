use std::path::Path;

use anyhow::Result;
use flip_ir::{Block, Document, Inline};
use printpdf::*;

const PAGE_WIDTH_MM: f32 = 210.0;
const PAGE_HEIGHT_MM: f32 = 297.0;
const MARGIN_MM: f32 = 20.0;
const LINE_HEIGHT: f32 = 14.0;
const USABLE_WIDTH_MM: f32 = PAGE_WIDTH_MM - 2.0 * MARGIN_MM;

struct StyledSegment {
    text: String,
    font: BuiltinFont,
    bold: bool,
}

fn flatten_inlines(inlines: &[Inline]) -> Vec<StyledSegment> {
    let mut segs = Vec::new();
    flatten_inlines_inner(inlines, BuiltinFont::Helvetica, false, &mut segs);
    segs
}

fn flatten_inlines_inner(
    inlines: &[Inline],
    current_font: BuiltinFont,
    current_bold: bool,
    segs: &mut Vec<StyledSegment>,
) {
    for inline in inlines {
        match inline {
            Inline::Text(t) => {
                if !t.is_empty() {
                    segs.push(StyledSegment {
                        text: t.clone(),
                        font: current_font,
                        bold: current_bold,
                    });
                }
            }
            Inline::Bold(inner) => {
                flatten_inlines_inner(inner, BuiltinFont::HelveticaBold, true, segs);
            }
            Inline::Italic(inner) => {
                flatten_inlines_inner(inner, BuiltinFont::HelveticaOblique, current_bold, segs);
            }
            Inline::Strikethrough(inner) => {
                flatten_inlines_inner(inner, current_font, current_bold, segs);
            }
            Inline::Code(t) => {
                if !t.is_empty() {
                    segs.push(StyledSegment {
                        text: t.clone(),
                        font: BuiltinFont::Courier,
                        bold: false,
                    });
                }
            }
            Inline::Link { text, .. } => {
                flatten_inlines_inner(text, BuiltinFont::HelveticaOblique, current_bold, segs);
            }
            Inline::Image { alt, .. } => {
                segs.push(StyledSegment {
                    text: format!("[{}]", alt),
                    font: current_font,
                    bold: current_bold,
                });
            }
            Inline::Superscript(inner) | Inline::Subscript(inner) => {
                flatten_inlines_inner(inner, current_font, current_bold, segs);
            }
        }
    }
}

fn segments_plain(segs: &[StyledSegment]) -> String {
    segs.iter().map(|s| s.text.as_str()).collect()
}

struct VisualLine {
    segments: Vec<StyledSegment>,
}

fn wrap_styled_segments(segs: &[StyledSegment], font_size: f32) -> Vec<VisualLine> {
    let plain = segments_plain(segs);
    if plain.is_empty() {
        return vec![VisualLine { segments: vec![] }];
    }

    let char_width = font_size * 0.18;
    let max_chars = ((USABLE_WIDTH_MM / char_width).floor() as usize).max(1);

    let mut plain_lines: Vec<String> = Vec::new();
    for raw_line in plain.split('\n') {
        if raw_line.trim().is_empty() {
            plain_lines.push(String::new());
            continue;
        }
        let mut remaining = raw_line;
        while !remaining.is_empty() {
            if remaining.len() <= max_chars {
                plain_lines.push(remaining.to_string());
                break;
            }
            let break_at = remaining[..max_chars].rfind(' ').unwrap_or(max_chars);
            plain_lines.push(remaining[..break_at].trim_end().to_string());
            remaining = remaining[break_at..].trim_start();
        }
    }

    let mut visual_lines = Vec::new();
    for plain_line in &plain_lines {
        if plain_line.is_empty() {
            visual_lines.push(VisualLine { segments: vec![] });
            continue;
        }

        let mut line_segs = Vec::new();
        let mut chars_so_far: usize = 0;
        let line_end = chars_so_far + plain_line.len();

        for seg in segs {
            if chars_so_far >= line_end {
                break;
            }
            let seg_end = chars_so_far + seg.text.len();
            if seg_end <= chars_so_far {
                continue;
            }

            let overlap_start = chars_so_far.max(line_end.saturating_sub(seg.text.len()));
            let overlap_end = seg_end.min(line_end);

            if overlap_start < overlap_end {
                let local_start = overlap_start.saturating_sub(chars_so_far);
                let local_end = overlap_end.saturating_sub(chars_so_far);
                if local_end > local_start && local_end <= seg.text.len() {
                    let snippet = seg.text[local_start..local_end].to_string();
                    if !snippet.is_empty() {
                        line_segs.push(StyledSegment {
                            text: snippet,
                            font: seg.font,
                            bold: seg.bold,
                        });
                    }
                }
            }
            chars_so_far = seg_end;
        }

        visual_lines.push(VisualLine { segments: line_segs });
    }

    if visual_lines.is_empty() {
        visual_lines.push(VisualLine { segments: vec![] });
    }

    visual_lines
}

fn emit_styled_line(
    ops: &mut Vec<Op>,
    segs: &[StyledSegment],
    x: f32,
    y: f32,
    font_size: f32,
) {
    ops.push(Op::StartTextSection);
    ops.push(Op::SetTextCursor {
        pos: Point {
            x: Mm(x).into(),
            y: Mm(y).into(),
        },
    });
    for seg in segs {
        if seg.text.is_empty() {
            continue;
        }
        ops.push(Op::SetFontSizeBuiltinFont {
            size: Pt(font_size),
            font: seg.font,
        });
        ops.push(Op::WriteTextBuiltinFont {
            items: vec![TextItem::Text(seg.text.clone())],
            font: seg.font,
        });
    }
    ops.push(Op::EndTextSection);
}

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
                let font_size = 11.0;
                let segs = flatten_inlines(inlines);
                let visual_lines = wrap_styled_segments(&segs, font_size);

                for vl in &visual_lines {
                    if y_pos < MARGIN_MM + 10.0 {
                        pages.push(PdfPage::new(
                            Mm(PAGE_WIDTH_MM),
                            Mm(PAGE_HEIGHT_MM),
                            std::mem::take(&mut ops),
                        ));
                        y_pos = PAGE_HEIGHT_MM - MARGIN_MM;
                    }
                    emit_styled_line(&mut ops, &vl.segments, MARGIN_MM, y_pos, font_size);
                    y_pos -= LINE_HEIGHT;
                }
                y_pos -= 4.0;
            }
            Block::Heading { level, content } => {
                let font_size = match level {
                    1 => 20.0,
                    2 => 16.0,
                    3 => 13.0,
                    _ => 11.0,
                };
                let line_height = font_size * 1.4;
                let segs = flatten_inlines(content);
                let styled: Vec<StyledSegment> = segs
                    .into_iter()
                    .map(|s| StyledSegment {
                        font: BuiltinFont::HelveticaBold,
                        bold: true,
                        ..s
                    })
                    .collect();
                let visual_lines = wrap_styled_segments(&styled, font_size);

                for vl in &visual_lines {
                    if y_pos < MARGIN_MM + line_height {
                        pages.push(PdfPage::new(
                            Mm(PAGE_WIDTH_MM),
                            Mm(PAGE_HEIGHT_MM),
                            std::mem::take(&mut ops),
                        ));
                        y_pos = PAGE_HEIGHT_MM - MARGIN_MM;
                    }
                    emit_styled_line(&mut ops, &vl.segments, MARGIN_MM, y_pos, font_size);
                    y_pos -= line_height;
                }
                y_pos -= 4.0;
            }
            Block::Code { content, .. } => {
                let font_size = 9.0;
                let line_height = 12.0;
                let char_width = font_size * 0.6;
                let max_chars = ((USABLE_WIDTH_MM / char_width).floor() as usize).max(1);

                for raw_line in content.split('\n') {
                    let mut remaining = raw_line;
                    loop {
                        if remaining.is_empty() {
                            break;
                        }
                        if y_pos < MARGIN_MM + 10.0 {
                            pages.push(PdfPage::new(
                                Mm(PAGE_WIDTH_MM),
                                Mm(PAGE_HEIGHT_MM),
                                std::mem::take(&mut ops),
                            ));
                            y_pos = PAGE_HEIGHT_MM - MARGIN_MM;
                        }
                        let line = if remaining.len() <= max_chars {
                            let l = remaining.to_string();
                            remaining = "";
                            l
                        } else {
                            let l = remaining[..max_chars].to_string();
                            remaining = &remaining[max_chars..];
                            l
                        };
                        emit_styled_line(
                            &mut ops,
                            &[StyledSegment {
                                text: line,
                                font: BuiltinFont::Courier,
                                bold: false,
                            }],
                            MARGIN_MM + 5.0,
                            y_pos,
                            font_size,
                        );
                        y_pos -= line_height;
                    }
                }
                y_pos -= 8.0;
            }
            Block::Blockquote(content) => {
                let font_size = 11.0;
                let segs = flatten_inlines(content);
                let styled: Vec<StyledSegment> = segs
                    .into_iter()
                    .map(|s| StyledSegment {
                        font: BuiltinFont::HelveticaOblique,
                        ..s
                    })
                    .collect();
                let visual_lines = wrap_styled_segments(&styled, font_size);

                for vl in &visual_lines {
                    if y_pos < MARGIN_MM + 10.0 {
                        pages.push(PdfPage::new(
                            Mm(PAGE_WIDTH_MM),
                            Mm(PAGE_HEIGHT_MM),
                            std::mem::take(&mut ops),
                        ));
                        y_pos = PAGE_HEIGHT_MM - MARGIN_MM;
                    }
                    emit_styled_line(&mut ops, &vl.segments, MARGIN_MM + 10.0, y_pos, font_size);
                    y_pos -= LINE_HEIGHT;
                }
                y_pos -= 4.0;
            }
            Block::List { items, ordered } => {
                let font_size = 11.0;

                for (i, item) in items.iter().enumerate() {
                    let prefix = if *ordered {
                        format!("{}. ", i + 1)
                    } else {
                        "- ".to_string()
                    };
                    let prefix_segs = vec![StyledSegment {
                        text: prefix,
                        font: BuiltinFont::Helvetica,
                        bold: false,
                    }];
                    let item_segs = flatten_inlines(item);
                    let all_segs: Vec<StyledSegment> =
                        prefix_segs.into_iter().chain(item_segs).collect();
                    let visual_lines = wrap_styled_segments(&all_segs, font_size);

                    for (j, vl) in visual_lines.iter().enumerate() {
                        if y_pos < MARGIN_MM + 10.0 {
                            pages.push(PdfPage::new(
                                Mm(PAGE_WIDTH_MM),
                                Mm(PAGE_HEIGHT_MM),
                                std::mem::take(&mut ops),
                            ));
                            y_pos = PAGE_HEIGHT_MM - MARGIN_MM;
                        }
                        let indent = if j == 0 { MARGIN_MM + 5.0 } else { MARGIN_MM + 15.0 };
                        emit_styled_line(&mut ops, &vl.segments, indent, y_pos, font_size);
                        y_pos -= LINE_HEIGHT;
                    }
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

                let max_cols = all_rows.iter().map(|r| r.len()).max().unwrap_or(1).max(1);
                let col_width = USABLE_WIDTH_MM / max_cols as f32;

                for (row_idx, row) in all_rows.iter().enumerate() {
                    let mut row_cell_lines: Vec<Vec<String>> = Vec::new();
                    for cell in *row {
                        let plain: String = cell.iter().map(|i| i.plain_text()).collect();
                        let cell_lines: Vec<String> = plain
                            .split('\n')
                            .map(|s| s.trim().to_string())
                            .collect();
                        row_cell_lines.push(cell_lines);
                    }

                    let max_row_lines = row_cell_lines.iter().map(|l| l.len()).max().unwrap_or(1);
                    for line_idx in 0..max_row_lines {
                        if y_pos < MARGIN_MM + 10.0 {
                            pages.push(PdfPage::new(
                                Mm(PAGE_WIDTH_MM),
                                Mm(PAGE_HEIGHT_MM),
                                std::mem::take(&mut ops),
                            ));
                            y_pos = PAGE_HEIGHT_MM - MARGIN_MM;
                        }
                        let mut x = MARGIN_MM;
                        for cell_lines in row_cell_lines.iter() {
                            let line_text = cell_lines.get(line_idx).cloned().unwrap_or_default();
                            let is_header = row_idx == 0 && !headers.is_empty();
                            let font = if is_header {
                                BuiltinFont::HelveticaBold
                            } else {
                                BuiltinFont::Helvetica
                            };
                            let seg = StyledSegment {
                                text: line_text,
                                font,
                                bold: is_header,
                            };
                            emit_styled_line(&mut ops, &[seg], x, y_pos, font_size);
                            x += col_width;
                        }
                        y_pos -= line_height;
                    }
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
                emit_styled_line(
                    &mut ops,
                    &[StyledSegment {
                        text: placeholder,
                        font: BuiltinFont::Helvetica,
                        bold: false,
                    }],
                    MARGIN_MM,
                    y_pos,
                    font_size,
                );
                y_pos -= 14.0;
            }
            Block::TableFromCsv(text) => {
                let font_size = 9.0;
                let line_height = 12.0;
                let char_width = font_size * 0.6;
                let max_chars = ((USABLE_WIDTH_MM / char_width).floor() as usize).max(1);

                for raw_line in text.lines() {
                    let mut remaining = raw_line;
                    loop {
                        if remaining.is_empty() {
                            break;
                        }
                        if y_pos < MARGIN_MM + 10.0 {
                            pages.push(PdfPage::new(
                                Mm(PAGE_WIDTH_MM),
                                Mm(PAGE_HEIGHT_MM),
                                std::mem::take(&mut ops),
                            ));
                            y_pos = PAGE_HEIGHT_MM - MARGIN_MM;
                        }
                        let line = if remaining.len() <= max_chars {
                            let l = remaining.to_string();
                            remaining = "";
                            l
                        } else {
                            let l = remaining[..max_chars].to_string();
                            remaining = &remaining[max_chars..];
                            l
                        };
                        emit_styled_line(
                            &mut ops,
                            &[StyledSegment {
                                text: line,
                                font: BuiltinFont::Courier,
                                bold: false,
                            }],
                            MARGIN_MM,
                            y_pos,
                            font_size,
                        );
                        y_pos -= line_height;
                    }
                }
                y_pos -= 8.0;
            }
        }
    }

    if !ops.is_empty() || pages.is_empty() {
        pages.push(PdfPage::new(Mm(PAGE_WIDTH_MM), Mm(PAGE_HEIGHT_MM), ops));
    }

    pdf_doc.with_pages(pages);
    let mut warnings = Vec::new();
    let bytes = pdf_doc.save(&PdfSaveOptions::default(), &mut warnings);
    Ok(bytes)
}
