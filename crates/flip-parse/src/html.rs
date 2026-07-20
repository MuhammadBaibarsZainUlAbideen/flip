use std::path::Path;

use anyhow::Result;
use flip_ir::{Block, Document, Inline, Metadata};

pub fn parse(path: &Path) -> Result<Document> {
    let html_content = std::fs::read_to_string(path)?;
    parse_html_str(&html_content)
}

pub fn parse_html_str(html_content: &str) -> Result<Document> {
    let mut doc = Document::new();
    doc.metadata = Metadata {
        source_format: Some("html".to_string()),
        ..Default::default()
    };

    extract_blocks(html_content, &mut doc);

    if doc.blocks.is_empty() {
        let text = strip_tags(html_content);
        if !text.trim().is_empty() {
            for line in text.lines() {
                let l = line.trim().to_string();
                if !l.is_empty() {
                    doc.push_block(Block::paragraph(l));
                }
            }
        }
    }

    Ok(doc)
}

fn strip_tags(html: &str) -> String {
    let mut text = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if in_tag => continue,
            '&' => {}
            _ => text.push(ch),
        }
    }
    text
}

fn extract_text_content(html: &str) -> String {
    let mut text = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if in_tag => continue,
            _ => text.push(ch),
        }
    }
    text.split_whitespace().collect::<Vec<&str>>().join(" ")
}

fn find_tag_content<'a>(html: &'a str, tag: &str) -> Vec<&'a str> {
    let open = format!("<{}", tag);
    let close = format!("</{}>", tag);
    let mut results = Vec::new();
    let mut pos = 0;

    while pos < html.len() {
        if let Some(start) = html[pos..].find(&open) {
            let tag_start = pos + start;
            if let Some(tag_end_offset) = html[tag_start..].find('>') {
                let content_start = tag_start + tag_end_offset + 1;
                if let Some(close_pos) = html[content_start..].find(&close) {
                    let content = &html[content_start..content_start + close_pos];
                    results.push(content);
                    pos = content_start + close_pos + close.len();
                } else {
                    break;
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }
    results
}

fn extract_blocks(html: &str, doc: &mut Document) {
    let tags = [
        ("h1", 1u8),
        ("h2", 2),
        ("h3", 3),
        ("h4", 4),
        ("h5", 5),
        ("h6", 6),
    ];

    for (tag, level) in &tags {
        for content in find_tag_content(html, tag) {
            let text = extract_text_content(content);
            if !text.trim().is_empty() {
                doc.push_block(Block::heading(*level, text.trim()));
            }
        }
    }

    for content in find_tag_content(html, "p") {
        let text = extract_text_content(content);
        if !text.trim().is_empty() {
            doc.push_block(Block::paragraph(text.trim()));
        }
    }

    for content in find_tag_content(html, "pre") {
        let text = extract_text_content(content);
        if !text.trim().is_empty() {
            doc.push_block(Block::code(text.trim()));
        }
    }

    for content in find_tag_content(html, "blockquote") {
        let text = extract_text_content(content);
        if !text.trim().is_empty() {
            doc.push_block(Block::Blockquote(vec![Inline::Text(
                text.trim().to_string(),
            )]));
        }
    }

    for content in find_tag_content(html, "li") {
        let text = extract_text_content(content);
        if !text.trim().is_empty() {
            doc.push_block(Block::List {
                ordered: false,
                items: vec![vec![Inline::Text(text.trim().to_string())]],
            });
        }
    }

    if doc.blocks.is_empty() {
        let text = strip_tags(html);
        for line in text.lines() {
            let l = line.trim().to_string();
            if !l.is_empty() {
                doc.push_block(Block::paragraph(l));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_html() {
        let html = r#"<html><body><h1>Title</h1><p>Hello world</p></body></html>"#;
        let doc = parse_html_str(html).unwrap();
        assert!(!doc.blocks.is_empty());
    }
}
