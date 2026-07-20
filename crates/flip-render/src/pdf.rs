use std::path::Path;

use anyhow::Result;
use flip_ir::{Block, Document, Inline};
use printpdf::*;

const PAGE_WIDTH_MM: f32 = 210.0;
const PAGE_HEIGHT_MM: f32 = 297.0;
const MARGIN_MM: f32 = 15.0;
const USABLE_WIDTH_MM: f32 = PAGE_WIDTH_MM - 2.0 * MARGIN_MM;

struct StyledSegment {
    text: String,
    font: BuiltinFont,
}

fn sanitize(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        let cp = ch as u32;
        if cp <= 0x7F || (cp >= 0xA0 && cp <= 0xFF) {
            out.push(ch);
        } else if cp == 0x2013 {
            out.push('-');
        } else if cp == 0x2014 {
            out.push_str("--");
        } else if cp == 0x2018 || cp == 0x2019 {
            out.push('\'');
        } else if cp == 0x201C || cp == 0x201D {
            out.push('"');
        } else if cp == 0x2026 {
            out.push_str("...");
        } else if cp == 0x2022 || cp == 0x25CF || cp == 0x25CB {
            out.push('-');
        } else {
            out.push('?');
        }
    }
    out
}

fn flatten_inlines(inlines: &[Inline]) -> Vec<StyledSegment> {
    let mut segs = Vec::new();
    flatten_inner(inlines, BuiltinFont::Helvetica, &mut segs);
    segs
}

fn flatten_inner(inlines: &[Inline], font: BuiltinFont, segs: &mut Vec<StyledSegment>) {
    for inline in inlines {
        match inline {
            Inline::Text(t) if !t.is_empty() => {
                let sanitized = sanitize(t);
                if !sanitized.is_empty() {
                    segs.push(StyledSegment { text: sanitized, font });
                }
            }
            Inline::Bold(inner) => flatten_inner(inner, BuiltinFont::HelveticaBold, segs),
            Inline::Italic(inner) => flatten_inner(inner, BuiltinFont::HelveticaOblique, segs),
            Inline::Strikethrough(inner) => flatten_inner(inner, font, segs),
            Inline::Code(t) if !t.is_empty() => {
                let sanitized = sanitize(t);
                if !sanitized.is_empty() {
                    segs.push(StyledSegment { text: sanitized, font: BuiltinFont::Courier });
                }
            }
            Inline::Link { text, .. } => flatten_inner(text, BuiltinFont::HelveticaOblique, segs),
            Inline::Image { alt, .. } => {
                let sanitized = sanitize(alt);
                segs.push(StyledSegment { text: format!("[{}]", sanitized), font });
            }
            Inline::Superscript(inner) | Inline::Subscript(inner) => {
                flatten_inner(inner, font, segs);
            }
            _ => {}
        }
    }
}

fn segments_to_plain(segs: &[StyledSegment]) -> String {
    segs.iter().map(|s| s.text.as_str()).collect()
}

fn wrap_plain(text: &str, font_size: f32, max_width_mm: f32) -> Vec<String> {
    if text.is_empty() {
        return vec![String::new()];
    }
    let char_width = font_size * 0.18;
    let max_chars = ((max_width_mm / char_width).floor() as usize).max(1);

    let mut result = Vec::new();
    for raw_line in text.split('\n') {
        if raw_line.trim().is_empty() {
            result.push(String::new());
            continue;
        }
        let mut remaining = raw_line;
        while !remaining.is_empty() {
            if remaining.len() <= max_chars {
                result.push(remaining.to_string());
                break;
            }
            let break_at = remaining[..max_chars].rfind(' ').unwrap_or(max_chars);
            result.push(remaining[..break_at].trim_end().to_string());
            remaining = remaining[break_at..].trim_start();
        }
    }
    if result.is_empty() {
        result.push(String::new());
    }
    result
}

fn split_segs_to_lines(segs: &[StyledSegment], plain_lines: &[String]) -> Vec<Vec<StyledSegment>> {
    let mut result = Vec::new();

    let mut seg_offsets: Vec<(usize, usize)> = Vec::new();
    let mut pos = 0usize;
    for seg in segs {
        let start = pos;
        pos += seg.text.len();
        seg_offsets.push((start, pos));
    }
    let total_len = pos;

    let mut char_pos = 0usize;
    for line in plain_lines {
        let line_len = line.len();
        let line_start = char_pos;
        let line_end = char_pos + line_len;
        let mut line_segs = Vec::new();

        for (i, seg) in segs.iter().enumerate() {
            let (seg_start, seg_end) = seg_offsets[i];
            if seg_end <= line_start || seg_start >= line_end {
                continue;
            }
            let vis_start = line_start.max(seg_start) - seg_start;
            let vis_end = line_end.min(seg_end) - seg_start;
            if vis_start < vis_end && vis_end <= seg.text.len() {
                let snippet = seg.text[vis_start..vis_end].to_string();
                if !snippet.is_empty() {
                    line_segs.push(StyledSegment { text: snippet, font: seg.font });
                }
            }
        }

        result.push(line_segs);
        char_pos = line_end;
        if char_pos >= total_len {
            break;
        }
    }

    result
}

fn emit_styled(ops: &mut Vec<Op>, segs: &[StyledSegment], x: f32, y: f32, font_size: f32) {
    if segs.is_empty() {
        return;
    }
    ops.push(Op::StartTextSection);
    ops.push(Op::SetTextCursor {
        pos: Point { x: Mm(x).into(), y: Mm(y).into() },
    });
    for seg in segs {
        if seg.text.is_empty() { continue; }
        ops.push(Op::SetFontSizeBuiltinFont { size: Pt(font_size), font: seg.font });
        ops.push(Op::WriteTextBuiltinFont {
            items: vec![TextItem::Text(seg.text.clone())],
            font: seg.font,
        });
    }
    ops.push(Op::EndTextSection);
}

fn estimate_text_width_mm(text: &str, font_size: f32) -> f32 {
    let char_width = font_size * 0.18;
    text.len() as f32 * char_width
}

fn draw_underline(ops: &mut Vec<Op>, x: f32, y: f32, width_mm: f32) {
    let underline_y = y - 1.0;
    let line = Line {
        points: vec![
            LinePoint { p: Point { x: Mm(x).into(), y: Mm(underline_y).into() }, bezier: false },
            LinePoint { p: Point { x: Mm(x + width_mm).into(), y: Mm(underline_y).into() }, bezier: false },
        ],
        is_closed: false,
    };
    ops.push(Op::SetOutlineColor { col: Color::Rgb(Rgb { r: 0.0, g: 0.0, b: 0.0, icc_profile: None }) });
    ops.push(Op::SetOutlineThickness { pt: Pt(0.5) });
    ops.push(Op::DrawLine { line });
}

fn ensure_page(ops: &mut Vec<Op>, pages: &mut Vec<PdfPage>, y_pos: &mut f32, needed: f32) {
    if *y_pos < MARGIN_MM + needed {
        pages.push(PdfPage::new(Mm(PAGE_WIDTH_MM), Mm(PAGE_HEIGHT_MM), std::mem::take(ops)));
        *y_pos = PAGE_HEIGHT_MM - MARGIN_MM;
    }
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
    let mut y: f32 = PAGE_HEIGHT_MM - MARGIN_MM;

    for block in &doc.blocks {
        match block {
            Block::Heading { level, content } => {
                let (font_size, is_underlined) = match level {
                    1 => (14.0, true),
                    2 => (11.0, true),
                    3 => (10.5, true),
                    _ => (10.0, true),
                };
                let lh = font_size * 0.45;
                let gap_before = if *level <= 2 { 4.0 } else { 2.0 };
                let gap_after = 2.0;
                y -= gap_before;
                ensure_page(&mut ops, &mut pages, &mut y, lh + gap_after);
                let segs = flatten_inlines(content);
                let styled: Vec<StyledSegment> = segs.into_iter()
                    .map(|s| StyledSegment { font: BuiltinFont::HelveticaBold, ..s })
                    .collect();
                let plain = segments_to_plain(&styled);
                let lines = wrap_plain(&plain, font_size, USABLE_WIDTH_MM);
                let styled_lines = split_segs_to_lines(&styled, &lines);
                let mut total_text_width: f32 = 0.0;
                for line_segs in &styled_lines {
                    ensure_page(&mut ops, &mut pages, &mut y, lh);
                    emit_styled(&mut ops, line_segs, MARGIN_MM, y, font_size);
                    let line_text = segments_to_plain(line_segs);
                    let w = estimate_text_width_mm(&line_text, font_size);
                    if w > total_text_width {
                        total_text_width = w;
                    }
                    y -= lh;
                }
                if is_underlined {
                    draw_underline(&mut ops, MARGIN_MM, y + lh, total_text_width.min(USABLE_WIDTH_MM));
                }
                y -= gap_after;
            }
            Block::Paragraph(inlines) => {
                let font_size = 9.5;
                let lh = font_size * 0.42;
                let segs = flatten_inlines(inlines);
                let plain = segments_to_plain(&segs);
                let lines = wrap_plain(&plain, font_size, USABLE_WIDTH_MM);
                let styled_lines = split_segs_to_lines(&segs, &lines);
                ensure_page(&mut ops, &mut pages, &mut y, lh * lines.len() as f32 + 2.0);
                for line_segs in &styled_lines {
                    emit_styled(&mut ops, &line_segs, MARGIN_MM, y, font_size);
                    y -= lh;
                }
                y -= 1.5;
            }
            Block::Code { content, .. } => {
                let font_size = 8.0;
                let lh = 10.0;
                for raw_line in content.split('\n') {
                    ensure_page(&mut ops, &mut pages, &mut y, lh);
                    let sanitized = sanitize(raw_line);
                    emit_styled(
                        &mut ops,
                        &[StyledSegment { text: sanitized, font: BuiltinFont::Courier }],
                        MARGIN_MM + 3.0, y, font_size,
                    );
                    y -= lh;
                }
                y -= 3.0;
            }
            Block::Blockquote(content) => {
                let font_size = 9.5;
                let lh = font_size * 0.42;
                let segs = flatten_inlines(content);
                let plain = segments_to_plain(&segs);
                let lines = wrap_plain(&plain, font_size, USABLE_WIDTH_MM - 8.0);
                let styled_lines = split_segs_to_lines(&segs, &lines);
                ensure_page(&mut ops, &mut pages, &mut y, lh * lines.len() as f32 + 2.0);
                for line_segs in &styled_lines {
                    emit_styled(&mut ops, &line_segs, MARGIN_MM + 8.0, y, font_size);
                    y -= lh;
                }
                y -= 1.5;
            }
            Block::List { items, ordered } => {
                let font_size = 9.5;
                let lh = font_size * 0.42;
                for (i, item) in items.iter().enumerate() {
                    let prefix = if *ordered { format!("{}. ", i + 1) } else { "\u{2022} ".to_string() };
                    let mut segs = vec![StyledSegment { text: sanitize(&prefix), font: BuiltinFont::Helvetica }];
                    segs.extend(flatten_inlines(item));
                    let plain = segments_to_plain(&segs);
                    let lines = wrap_plain(&plain, font_size, USABLE_WIDTH_MM - 5.0);
                    let styled_lines = split_segs_to_lines(&segs, &lines);
                    for (j, line_segs) in styled_lines.iter().enumerate() {
                        ensure_page(&mut ops, &mut pages, &mut y, lh);
                        let indent = if j == 0 { 5.0 } else { 9.0 };
                        emit_styled(&mut ops, line_segs, MARGIN_MM + indent, y, font_size);
                        y -= lh;
                    }
                }
                y -= 1.0;
            }
            Block::HorizontalRule => {
                y -= 2.0;
                let line = Line {
                    points: vec![
                        LinePoint { p: Point { x: Mm(MARGIN_MM).into(), y: Mm(y).into() }, bezier: false },
                        LinePoint { p: Point { x: Mm(PAGE_WIDTH_MM - MARGIN_MM).into(), y: Mm(y).into() }, bezier: false },
                    ],
                    is_closed: false,
                };
                ops.push(Op::SetOutlineColor { col: Color::Rgb(Rgb { r: 0.7, g: 0.7, b: 0.7, icc_profile: None }) });
                ops.push(Op::SetOutlineThickness { pt: Pt(0.3) });
                ops.push(Op::DrawLine { line });
                y -= 3.0;
            }
            Block::Table { headers, rows } => {
                let font_size = 8.0;
                let lh = 3.2;

                let mut all_rows: Vec<&Vec<Vec<Inline>>> = Vec::new();
                for h in headers { all_rows.push(h); }
                for r in rows { all_rows.push(r); }

                let max_cols = all_rows.iter().map(|r| r.len()).max().unwrap_or(1).max(1);
                let col_width = USABLE_WIDTH_MM / max_cols as f32;

                for (row_idx, row) in all_rows.iter().enumerate() {
                    let is_header = row_idx == 0 && !headers.is_empty();
                    let font = if is_header { BuiltinFont::HelveticaBold } else { BuiltinFont::Helvetica };

                    let mut cell_lines: Vec<Vec<String>> = Vec::new();
                    for cell in *row {
                        let plain: String = cell.iter().map(|i| sanitize(&i.plain_text())).collect();
                        let lines: Vec<String> = plain.split('\n')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        cell_lines.push(lines);
                    }

                    let max_lines = cell_lines.iter().map(|l| l.len()).max().unwrap_or(1).max(1);
                    for line_idx in 0..max_lines {
                        ensure_page(&mut ops, &mut pages, &mut y, lh);
                        let mut x = MARGIN_MM;
                        for cell in &cell_lines {
                            let text = cell.get(line_idx).cloned().unwrap_or_default();
                            emit_styled(
                                &mut ops,
                                &[StyledSegment { text, font }],
                                x, y, font_size,
                            );
                            x += col_width;
                        }
                        y -= lh;
                    }
                }
                y -= 1.5;
            }
            Block::Image { alt, .. } => {
                ensure_page(&mut ops, &mut pages, &mut y, 5.0);
                let sanitized = sanitize(alt);
                emit_styled(
                    &mut ops,
                    &[StyledSegment { text: format!("[Image: {}]", sanitized), font: BuiltinFont::Helvetica }],
                    MARGIN_MM, y, 8.0,
                );
                y -= 5.0;
            }
            Block::TableFromCsv(text) => {
                let font_size = 7.0;
                let lh = 3.0;
                for raw_line in text.lines() {
                    ensure_page(&mut ops, &mut pages, &mut y, lh);
                    let sanitized = sanitize(raw_line);
                    emit_styled(
                        &mut ops,
                        &[StyledSegment { text: sanitized, font: BuiltinFont::Courier }],
                        MARGIN_MM, y, font_size,
                    );
                    y -= lh;
                }
                y -= 1.0;
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
