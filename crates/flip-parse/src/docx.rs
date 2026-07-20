use std::io::Read;
use std::path::Path;

use anyhow::Result;
use flip_ir::{Block, Document, Inline, Metadata};

fn decode_xml_entities(text: &str) -> String {
    let mut result = text.to_string();
    result = result.replace("&amp;", "&");
    result = result.replace("&lt;", "<");
    result = result.replace("&gt;", ">");
    result = result.replace("&quot;", "\"");
    result = result.replace("&apos;", "'");
    result = result.replace('\u{00A0}', " ");
    result = result.replace('\u{200B}', "");
    result = result.replace('\u{FEFF}', "");
    result = result.replace('\u{2197}', "");
    result = result.replace('\u{2198}', "");
    result = result.replace('\u{2191}', "");
    result = result.replace('\u{2193}', "");
    result = result.replace('\u{2190}', "");
    result = result.replace('\u{2192}', "");
    result
}

pub fn parse(path: &Path) -> Result<Document> {
    let mut doc = Document::new();
    doc.metadata = Metadata {
        source_format: Some("docx".to_string()),
        ..Default::default()
    };

    let bytes = std::fs::read(path)?;
    let zip = zip::ZipArchive::new(std::io::Cursor::new(&bytes))?;
    let xml = extract_xml(zip, "word/document.xml");

    if xml.is_empty() {
        let plain = extract_office_plain_text(&bytes)?;
        for line in plain.lines() {
            let trimmed = line.trim().to_string();
            if !trimmed.is_empty() {
                doc.push_block(Block::paragraph(trimmed));
            }
        }
    } else {
        process_body_elements(&xml, &mut doc);
    }

    if doc.blocks.is_empty() {
        doc.push_block(Block::paragraph(
            "(No text content could be extracted from this document)",
        ));
    }

    Ok(doc)
}

fn table_has_visible_borders(table_xml: &str) -> bool {
    if let Some(start) = table_xml.find("<w:tblBorders") {
        if let Some(end) = table_xml[start..].find("</w:tblBorders>") {
            let borders = &table_xml[start..start + end + 16];
            return borders.contains("val=\"single\"")
                || borders.contains("val=\"double\"")
                || borders.contains("val=\"thick\"")
                || borders.contains("val=\"thickThinLargeGap\"")
                || borders.contains("val=\"wave\"");
        }
    }
    true
}

fn flatten_borderless_table(table_xml: &str, doc: &mut Document) {
    let mut remaining = table_xml;

    loop {
        let tr_start = match find_next_tag(remaining, "w:tr") {
            Some(s) => s,
            None => break,
        };

        let tr_end = match find_closing_tag(remaining, tr_start, "w:tr") {
            Some(e) => e,
            None => break,
        };

        let row_xml = &remaining[tr_start..tr_end];
        let mut tc_remaining = row_xml;
        let mut cell_paragraphs: Vec<Vec<Vec<Inline>>> = Vec::new();

        loop {
            let tc_start = match find_next_tag(tc_remaining, "w:tc") {
                Some(s) => s,
                None => break,
            };
            let tc_end = match find_closing_tag(tc_remaining, tc_start, "w:tc") {
                Some(e) => e,
                None => break,
            };

            let cell_xml = &tc_remaining[tc_start..tc_end];
            let mut paragraphs_in_cell: Vec<Vec<Inline>> = Vec::new();
            let mut p_remaining = cell_xml;

            loop {
                let p_start = match find_next_tag(p_remaining, "w:p") {
                    Some(s) => s,
                    None => break,
                };
                let p_end = match find_closing_tag(p_remaining, p_start, "w:p") {
                    Some(e) => e,
                    None => break,
                };
                let para_xml = &p_remaining[p_start..p_end];
                let inlines = extract_inlines(para_xml);
                let text: String = inlines.iter().map(|i| i.plain_text()).collect();
                if !text.trim().is_empty() {
                    paragraphs_in_cell.push(inlines);
                }
                p_remaining = &p_remaining[p_end..];
            }

            if paragraphs_in_cell.is_empty() {
                paragraphs_in_cell.push(vec![]);
            }
            cell_paragraphs.push(paragraphs_in_cell);
            tc_remaining = &tc_remaining[tc_end..];
        }

        if cell_paragraphs.is_empty() {
            remaining = &remaining[tr_end..];
            continue;
        }

        if cell_paragraphs.len() == 1 {
            for para in &cell_paragraphs[0] {
                if !para.is_empty() {
                    doc.push_block(Block::Paragraph(para.clone()));
                }
            }
        } else {
            let max_paras = cell_paragraphs.iter().map(|c| c.len()).max().unwrap_or(0);
            for pi in 0..max_paras {
                let mut parts: Vec<String> = Vec::new();
                for cell_paras in &cell_paragraphs {
                    if pi < cell_paras.len() {
                        let text: String = cell_paras[pi].iter().map(|i| i.plain_text()).collect();
                        let trimmed = text.trim().to_string();
                        if !trimmed.is_empty() {
                            parts.push(trimmed);
                        }
                    }
                }
                if !parts.is_empty() {
                    let joined = if parts.len() == 1 {
                        parts[0].clone()
                    } else {
                        parts.join(" | ")
                    };
                    doc.push_block(Block::paragraph(joined));
                }
            }
        }

        remaining = &remaining[tr_end..];
    }
}

fn emit_table(table_xml: &str, doc: &mut Document) {
    if !table_has_visible_borders(table_xml) {
        flatten_borderless_table(table_xml, doc);
        return;
    }

    let inlines_rows = extract_table(table_xml);
    if inlines_rows.is_empty() {
        return;
    }

    let first_row = &inlines_rows[0];
    let all_rows_have_same_len = inlines_rows.iter().all(|r| r.len() == first_row.len());

    if all_rows_have_same_len && inlines_rows.len() > 1 {
        let headers = vec![inlines_rows[0].clone()];
        let rows: Vec<Vec<Vec<Inline>>> = inlines_rows.into_iter().skip(1).collect();
        doc.push_block(Block::Table { headers, rows });
    } else {
        for row in &inlines_rows {
            let text: String = row
                .iter()
                .map(|c| {
                    c.iter()
                        .map(|i| i.plain_text())
                        .collect::<Vec<_>>()
                        .join(" | ")
                })
                .collect::<Vec<_>>()
                .join(" | ");
            doc.push_block(Block::paragraph(text));
        }
    }
}

fn process_body_elements(xml: &str, doc: &mut Document) {
    let mut remaining = xml;

    loop {
        let next_p = find_next_tag(remaining, "w:p");
        let next_tbl = find_next_tag(remaining, "w:tbl");

        match (next_p, next_tbl) {
            (Some(ppos), Some(tpos)) if tpos < ppos => {
                if let Some(end) = find_closing_tag(remaining, tpos, "w:tbl") {
                    let table_xml = &remaining[tpos..end];
                    emit_table(table_xml, doc);
                    remaining = &remaining[end..];
                } else {
                    break;
                }
            }
            (Some(ppos), _) => {
                if let Some(end) = find_closing_tag(remaining, ppos, "w:p") {
                    let para_xml = &remaining[ppos..end];
                    emit_paragraph(para_xml, doc);
                    remaining = &remaining[end..];
                } else {
                    break;
                }
            }
            (None, Some(tpos)) => {
                if let Some(end) = find_closing_tag(remaining, tpos, "w:tbl") {
                    let table_xml = &remaining[tpos..end];
                    emit_table(table_xml, doc);
                    remaining = &remaining[end..];
                } else {
                    break;
                }
            }
            (None, None) => break,
        }
    }
}

fn find_next_tag(text: &str, tag: &str) -> Option<usize> {
    let pattern1 = format!("<{}>", tag);
    let pattern2 = format!("<{} ", tag);
    let p1 = text.find(&pattern1);
    let p2 = text.find(&pattern2);
    match (p1, p2) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

fn para_has_bold_underline(para_xml: &str) -> bool {
    if let Some(ppr_start) = para_xml.find("<w:pPr>") {
        if let Some(ppr_end) = para_xml[ppr_start..].find("</w:pPr>") {
            let ppr = &para_xml[ppr_start..ppr_start + ppr_end + 9];
            if let Some(rpr_start) = ppr.find("<w:rPr>") {
                if let Some(rpr_end) = ppr[rpr_start..].find("</w:rPr>") {
                    let rpr = &ppr[rpr_start..rpr_start + rpr_end + 8];
                    let has_bold = rpr.contains("<w:b/>") || rpr.contains("<w:bCs/>");
                    let has_underline = rpr.contains("<w:u ");
                    return has_bold && has_underline;
                }
            }
        }
    }
    false
}

fn para_has_bold_in_ppr(para_xml: &str) -> bool {
    if let Some(ppr_start) = para_xml.find("<w:pPr>") {
        if let Some(ppr_end) = para_xml[ppr_start..].find("</w:pPr>") {
            let ppr = &para_xml[ppr_start..ppr_start + ppr_end + 9];
            if let Some(rpr_start) = ppr.find("<w:rPr>") {
                if let Some(rpr_end) = ppr[rpr_start..].find("</w:rPr>") {
                    let rpr = &ppr[rpr_start..rpr_start + rpr_end + 8];
                    return rpr.contains("<w:b/>") || rpr.contains("<w:bCs/>");
                }
            }
        }
    }
    false
}

fn is_short_header_text(text: &str) -> bool {
    let trimmed = text.trim();
    !trimmed.is_empty()
        && trimmed.len() < 40
        && !trimmed.starts_with('-')
        && !trimmed.contains('|')
        && !trimmed.contains('@')
        && trimmed.chars().filter(|c| c.is_alphabetic()).count() > 2
}

fn all_inlines_bold(inlines: &[Inline]) -> bool {
    if inlines.is_empty() {
        return false;
    }
    inlines.iter().all(|i| {
        matches!(i, Inline::Bold(_))
            || matches!(i, Inline::Text(t) if t.trim().is_empty())
    })
}

fn emit_paragraph(para_xml: &str, doc: &mut Document) {
    let style = extract_style(para_xml);
    let inlines = extract_inlines(para_xml);

    if inlines.is_empty() {
        return;
    }

    match style.as_deref() {
        Some("Heading1") | Some("heading1") => {
            doc.push_block(Block::heading(1, inlines_to_text(&inlines)));
        }
        Some("Heading2") | Some("heading2") => {
            doc.push_block(Block::heading(2, inlines_to_text(&inlines)));
        }
        Some("Heading3") | Some("heading3") => {
            doc.push_block(Block::heading(3, inlines_to_text(&inlines)));
        }
        Some("Heading4") | Some("heading4") => {
            doc.push_block(Block::heading(4, inlines_to_text(&inlines)));
        }
        Some("Heading5") | Some("heading5") | Some("Heading6") | Some("heading6") => {
            doc.push_block(Block::heading(5, inlines_to_text(&inlines)));
        }
        Some(s) if s.contains("ListParagraph") || s.contains("list") => {
            doc.push_block(Block::List {
                ordered: s.to_lowercase().contains("number"),
                items: vec![inlines],
            });
        }
        _ => {
            let has_numpr = para_xml.contains("<w:numPr>");
            if has_numpr {
                doc.push_block(Block::List {
                    ordered: false,
                    items: vec![inlines],
                });
            } else if para_has_bold_underline(para_xml) {
                doc.push_block(Block::heading(2, inlines_to_text(&inlines)));
            } else if para_has_bold_in_ppr(para_xml) && is_short_header_text(&inlines_to_text(&inlines)) {
                doc.push_block(Block::heading(2, inlines_to_text(&inlines)));
            } else if all_inlines_bold(&inlines) && is_short_header_text(&inlines_to_text(&inlines)) {
                doc.push_block(Block::heading(2, inlines_to_text(&inlines)));
            } else {
                doc.push_block(Block::Paragraph(inlines));
            }
        }
    }
}

fn extract_table(table_xml: &str) -> Vec<Vec<Vec<Inline>>> {
    let mut rows = Vec::new();
    let mut remaining = table_xml;

    loop {
        let tr_start = find_next_tag(remaining, "w:tr");
        let tr_start = match tr_start {
            Some(s) => s,
            None => break,
        };

        if let Some(tr_end) = find_closing_tag(remaining, tr_start, "w:tr") {
            let row_xml = &remaining[tr_start..tr_end];
            let cells = extract_table_row(row_xml);
            if !cells.is_empty() {
                rows.push(cells);
            }
            remaining = &remaining[tr_end..];
        } else {
            break;
        }
    }

    rows
}

fn extract_table_row(row_xml: &str) -> Vec<Vec<Inline>> {
    let mut cells = Vec::new();
    let mut remaining = row_xml;

    loop {
        let tc_start = find_next_tag(remaining, "w:tc");
        let tc_start = match tc_start {
            Some(s) => s,
            None => break,
        };

        if let Some(tc_end) = find_closing_tag(remaining, tc_start, "w:tc") {
            let cell_xml = &remaining[tc_start..tc_end];
            let mut cell_inlines = Vec::new();

            let mut cell_remaining = cell_xml;
            loop {
                let p_start = find_next_tag(cell_remaining, "w:p");
                let p_start = match p_start {
                    Some(s) => s,
                    None => break,
                };
                if let Some(p_end) = find_closing_tag(cell_remaining, p_start, "w:p") {
                    let para = &cell_remaining[p_start..p_end];
                    let inlines = extract_inlines(para);
                    for inline in inlines {
                        cell_inlines.push(inline);
                    }
                    cell_remaining = &cell_remaining[p_end..];
                } else {
                    break;
                }
            }

            cells.push(cell_inlines);
            remaining = &remaining[tc_end..];
        } else {
            break;
        }
    }

    cells
}

fn extract_xml(
    mut zip: zip::ZipArchive<std::io::Cursor<&Vec<u8>>>,
    xml_path: &str,
) -> String {
    let mut content = String::new();
    if let Ok(mut file) = zip.by_name(xml_path) {
        let _ = file.read_to_string(&mut content);
    }
    content
}

fn find_closing_tag(text: &str, start: usize, tag: &str) -> Option<usize> {
    let open_pattern = format!("<{} ", tag);
    let open_exact = format!("<{}>", tag);
    let close_pattern = format!("</{}>", tag);

    let mut depth = 0i32;
    let mut i = start;

    while i < text.len() {
        let slice = &text[i..];

        if slice.starts_with(&open_pattern) || slice.starts_with(&open_exact) {
            depth += 1;
            i += open_pattern.len();
            continue;
        }

        if slice.starts_with(&close_pattern) {
            depth -= 1;
            if depth <= 0 {
                return Some(i + close_pattern.len());
            }
            i += close_pattern.len();
            continue;
        }

        i += slice.chars().next().unwrap_or('\0').len_utf8();
    }

    None
}

fn extract_style(para: &str) -> Option<String> {
    if let Some(pstyle_start) = para.find("<w:pStyle ") {
        let rest = &para[pstyle_start..];
        if let Some(val_start) = rest.find("w:val=\"") {
            let val_start = val_start + 7;
            if let Some(val_end) = rest[val_start..].find('"') {
                return Some(rest[val_start..val_start + val_end].to_string());
            }
        }
    }
    None
}

fn extract_run_inlines(run: &str) -> Vec<Inline> {
    let mut inlines = Vec::new();

    let is_bold = run.contains("<w:b/>")
        || run.contains("<w:b w:val=\"true\">")
        || run.contains("<w:b w:val=\"1\">");
    let is_italic = run.contains("<w:i/>")
        || run.contains("<w:i w:val=\"true\">")
        || run.contains("<w:i w:val=\"1\">");
    let is_strike = run.contains("<w:strike/>");

    let mut run_text = String::new();
    let mut run_remaining = run;
    while let Some(t_start) = run_remaining.find("<w:t") {
        let after_t = &run_remaining[t_start..];
        let close_bracket = after_t.find('>').unwrap_or(0);
        let text_start = t_start + close_bracket + 1;

        if let Some(t_end) = run_remaining[text_start..].find("</w:t>") {
            run_text.push_str(&run_remaining[text_start..text_start + t_end]);
            run_remaining = &run_remaining[text_start + t_end..];
        } else {
            break;
        }
    }

    if run.contains("<w:tab/>") || run.contains("<w:tab />") {
        let pos = run.find("<w:tab").unwrap_or(0);
        let before = &run[..pos];
        if before.find("</w:t>").is_some() || before.find("<w:t").is_none() {
            run_text.push('\t');
        }
    }

    if run.contains("<w:br/>") || run.contains("<w:br />") || run.contains("<w:br ") {
        let pos = run.find("<w:br").unwrap_or(0);
        let before = &run[..pos];
        if before.find("</w:t>").is_some() || before.find("<w:t").is_none() {
            run_text.push('\n');
        }
    }

    if !run_text.is_empty() {
        let decoded = decode_xml_entities(&run_text);
        let mut inline = Inline::Text(decoded);
        if is_strike {
            inline = Inline::Strikethrough(vec![inline]);
        }
        if is_italic {
            inline = Inline::Italic(vec![inline]);
        }
        if is_bold {
            inline = Inline::Bold(vec![inline]);
        }
        inlines.push(inline);
    }

    inlines
}

fn extract_inlines(para: &str) -> Vec<Inline> {
    let mut inlines = Vec::new();
    let mut remaining = para;

    while !remaining.is_empty() {
        let next_run = find_run_start(remaining);
        let next_hl = find_hyperlink_start(remaining);
        let next_pos = match (next_run, next_hl) {
            (Some(r), Some(h)) => Some(r.min(h)),
            (Some(r), None) => Some(r),
            (None, Some(h)) => Some(h),
            (None, None) => None,
        };

        let pos = match next_pos {
            Some(p) => p,
            None => break,
        };

        let slice = &remaining[pos..];
        if slice.starts_with("<w:hyperlink") {
            if let Some(hl_end) = find_closing_tag(remaining, pos, "w:hyperlink") {
                let hl_xml = &remaining[pos..hl_end];
                let mut hl_remaining = hl_xml;
                while let Some(run_start) = find_run_start(hl_remaining) {
                    let run_tag_end = hl_remaining[run_start..].find('>').map(|e| run_start + e + 1).unwrap_or(hl_remaining.len());
                    let run_end = hl_remaining[run_tag_end..]
                        .find("</w:r>")
                        .map(|e| run_tag_end + e + 6)
                        .unwrap_or(hl_remaining.len());
                    let run = &hl_remaining[run_start..run_end];
                    let run_inlines = extract_run_inlines(run);
                    inlines.extend(run_inlines);
                    hl_remaining = &hl_remaining[run_end..];
                }
                remaining = &remaining[hl_end..];
            } else {
                break;
            }
        } else {
            let run_tag_end = remaining[pos..].find('>').map(|e| pos + e + 1).unwrap_or(remaining.len());
            let run_end = remaining[run_tag_end..]
                .find("</w:r>")
                .map(|e| run_tag_end + e + 6)
                .unwrap_or(remaining.len());
            let run = &remaining[pos..run_end];
            let run_inlines = extract_run_inlines(run);
            inlines.extend(run_inlines);
            remaining = &remaining[run_end..];
        }
    }

    inlines
}

fn find_run_start(text: &str) -> Option<usize> {
    let p1 = text.find("<w:r>");
    let p2 = text.find("<w:r ");
    match (p1, p2) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

fn find_hyperlink_start(text: &str) -> Option<usize> {
    let p1 = text.find("<w:hyperlink>");
    let p2 = text.find("<w:hyperlink ");
    match (p1, p2) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

fn inlines_to_text(inlines: &[Inline]) -> String {
    inlines.iter().map(|i| i.plain_text()).collect()
}

fn extract_office_plain_text(bytes: &[u8]) -> Result<String> {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes))?;
    let mut text = String::new();
    for i in 0..zip.len() {
        if let Ok(mut file) = zip.by_index(i) {
            let name = file.name().to_string();
            if name.ends_with(".xml") || name.ends_with(".rels") {
                continue;
            }
            if name.contains("document") || name.contains("slide") || name.contains("content") {
                let mut content = String::new();
                if file.read_to_string(&mut content).is_ok() {
                    text.push_str(&content);
                    text.push('\n');
                }
            }
        }
    }
    Ok(text)
}
