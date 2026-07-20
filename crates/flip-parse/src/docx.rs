use std::io::Read;
use std::path::Path;

use anyhow::Result;
use flip_ir::{Block, Document, Inline, Metadata};

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
        let paragraphs = extract_paragraphs(&xml);

        for para in &paragraphs {
            let style = extract_style(para);
            let inlines = extract_inlines(para);

            if inlines.is_empty() {
                continue;
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
                    let has_numpr = para.contains("<w:numPr>");
                    if has_numpr {
                        doc.push_block(Block::List {
                            ordered: false,
                            items: vec![inlines],
                        });
                    } else {
                        doc.push_block(Block::Paragraph(inlines));
                    }
                }
            }
        }
    }

    if doc.blocks.is_empty() {
        doc.push_block(Block::paragraph(
            "(No text content could be extracted from this document)",
        ));
    }

    Ok(doc)
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

fn extract_paragraphs(xml: &str) -> Vec<String> {
    let mut paragraphs = Vec::new();
    let mut remaining = xml;

    loop {
        // Find either <w:p> or <w:p (with attributes)
        let start = remaining.find("<w:p>").or_else(|| remaining.find("<w:p "));
        let start = match start {
            Some(s) => s,
            None => break,
        };

        if let Some(end) = find_closing_tag(remaining, start, "w:p") {
            paragraphs.push(remaining[start..end].to_string());
            remaining = &remaining[end..];
        } else {
            break;
        }
    }

    paragraphs
}

fn find_closing_tag(text: &str, start: usize, tag: &str) -> Option<usize> {
    let open_pattern = format!("<{} ", tag);
    let open_exact = format!("<{}>", tag);
    let close_pattern = format!("</{}>", tag);

    let mut depth = 0i32;
    let mut i = start;

    while i < text.len() {
        if text[i..].starts_with(&open_pattern) || text[i..].starts_with(&open_exact) {
            depth += 1;
            i += 1;
            continue;
        }

        if text[i..].starts_with(&close_pattern) {
            depth -= 1;
            if depth <= 0 {
                return Some(i + close_pattern.len());
            }
            i += 1;
            continue;
        }

        i += 1;
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

fn extract_inlines(para: &str) -> Vec<Inline> {
    let mut inlines = Vec::new();
    let mut remaining = para;

    while let Some(run_start) = remaining.find("<w:r>") {
        let run_end = remaining[run_start..]
            .find("</w:r>")
            .map(|e| run_start + e + 6)
            .unwrap_or(remaining.len());

        let run = &remaining[run_start..run_end];

        let is_bold = run.contains("<w:b/>") || run.contains("<w:b w:val=\"true\">")
            || run.contains("<w:b w:val=\"1\">");
        let is_italic = run.contains("<w:i/>") || run.contains("<w:i w:val=\"true\">")
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

        if !run_text.is_empty() {
            let mut inline = Inline::Text(run_text);
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

        remaining = &remaining[run_end..];
    }

    inlines
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
