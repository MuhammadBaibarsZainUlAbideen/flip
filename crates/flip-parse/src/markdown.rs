use std::path::Path;

use anyhow::Result;
use flip_ir::{Block, Document, Inline, Metadata};

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

pub fn parse(path: &Path) -> Result<Document> {
    let content = std::fs::read_to_string(path)?;
    parse_markdown_str(&content)
}

pub fn parse_markdown_str(content: &str) -> Result<Document> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(content, options);
    let mut doc = Document::new();
    doc.metadata = Metadata {
        source_format: Some("markdown".to_string()),
        ..Default::default()
    };

    let mut current_paragraph = Vec::new();
    let mut in_code_block = false;
    let mut code_content = String::new();
    let mut code_language = None;
    let mut list_items = Vec::new();
    let mut in_list = false;
    let mut list_ordered = false;
    let mut table_headers = Vec::new();
    let mut table_rows = Vec::new();
    let mut current_row = Vec::new();
    let mut in_table = false;
    let mut in_table_header = false;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                if !current_paragraph.is_empty() {
                    doc.push_block(Block::Paragraph(current_paragraph.clone()));
                    current_paragraph.clear();
                }
                if in_list {
                    doc.push_block(Block::List {
                        ordered: list_ordered,
                        items: list_items.clone(),
                    });
                    list_items.clear();
                    in_list = false;
                }
                let level_num = match level {
                    pulldown_cmark::HeadingLevel::H1 => 1,
                    pulldown_cmark::HeadingLevel::H2 => 2,
                    pulldown_cmark::HeadingLevel::H3 => 3,
                    pulldown_cmark::HeadingLevel::H4 => 4,
                    pulldown_cmark::HeadingLevel::H5 => 5,
                    pulldown_cmark::HeadingLevel::H6 => 6,
                };
                current_paragraph.push(Inline::Text(String::new()));
                let heading_level = level_num;
                let mut content = Vec::new();
                std::mem::swap(&mut content, &mut current_paragraph);
                doc.push_block(Block::Heading {
                    level: heading_level,
                    content,
                });
            }
            Event::End(TagEnd::Heading(_)) => {
                current_paragraph.clear();
            }
            Event::Start(Tag::Paragraph) => {
                current_paragraph.clear();
            }
            Event::End(TagEnd::Paragraph) => {
                if !current_paragraph.is_empty() {
                    doc.push_block(Block::Paragraph(current_paragraph.clone()));
                    current_paragraph.clear();
                }
            }
            Event::Start(Tag::CodeBlock(lang)) => {
                in_code_block = true;
                code_content.clear();
                code_language = match lang {
                    pulldown_cmark::CodeBlockKind::Fenced(l) => {
                        let s = l.to_string();
                        if s.is_empty() {
                            None
                        } else {
                            Some(s)
                        }
                    }
                    pulldown_cmark::CodeBlockKind::Indented => None,
                };
            }
            Event::Text(text) => {
                if in_code_block {
                    code_content.push_str(&text);
                } else if in_table {
                    if !current_row.is_empty() {
                        let last: &mut Vec<Inline> = current_row.last_mut().unwrap();
                        last.push(Inline::Text(text.to_string()));
                    }
                } else {
                    current_paragraph.push(Inline::Text(text.to_string()));
                }
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                doc.push_block(Block::Code {
                    language: code_language.clone(),
                    content: code_content.clone(),
                });
                code_content.clear();
                code_language = None;
            }
            Event::Start(Tag::List(ordered)) => {
                if !current_paragraph.is_empty() {
                    doc.push_block(Block::Paragraph(current_paragraph.clone()));
                    current_paragraph.clear();
                }
                in_list = true;
                list_ordered = ordered.is_some();
                list_items.clear();
            }
            Event::End(TagEnd::List(_)) => {
                if in_list && !list_items.is_empty() {
                    doc.push_block(Block::List {
                        ordered: list_ordered,
                        items: list_items.clone(),
                    });
                    list_items.clear();
                    in_list = false;
                }
            }
            Event::Start(Tag::Item) => {
                current_paragraph.clear();
            }
            Event::End(TagEnd::Item) => {
                if !current_paragraph.is_empty() {
                    list_items.push(current_paragraph.clone());
                    current_paragraph.clear();
                }
            }
            Event::Start(Tag::Table(_)) => {
                in_table = true;
                in_table_header = true;
                table_headers.clear();
                table_rows.clear();
                current_row.clear();
            }
            Event::Start(Tag::TableHead) => {
                in_table_header = true;
                current_row.clear();
            }
            Event::End(TagEnd::TableHead) => {
                if !current_row.is_empty() {
                    table_headers.push(current_row.clone());
                }
                current_row.clear();
                in_table_header = false;
            }
            Event::Start(Tag::TableRow) => {
                if !in_table_header {
                    current_row.clear();
                }
            }
            Event::End(TagEnd::TableRow) => {
                if !in_table_header && !current_row.is_empty() {
                    table_rows.push(current_row.clone());
                }
                current_row.clear();
            }
            Event::Start(Tag::TableCell) => {
                current_row.push(Vec::new());
            }
            Event::End(TagEnd::TableCell) => {}
            Event::End(TagEnd::Table) => {
                in_table = false;
                doc.push_block(Block::Table {
                    headers: table_headers.clone(),
                    rows: table_rows.clone(),
                });
                table_headers.clear();
                table_rows.clear();
            }
            Event::Start(Tag::BlockQuote(_)) => {
                current_paragraph.clear();
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                if !current_paragraph.is_empty() {
                    doc.push_block(Block::Blockquote(current_paragraph.clone()));
                    current_paragraph.clear();
                }
            }
            Event::HardBreak => {
                current_paragraph.push(Inline::Text("\n".to_string()));
            }
            Event::SoftBreak => {
                current_paragraph.push(Inline::Text(" ".to_string()));
            }
            Event::Rule => {
                doc.push_block(Block::HorizontalRule);
            }
            Event::Start(Tag::Emphasis) => {
                current_paragraph.push(Inline::Text(String::new()));
            }
            Event::End(TagEnd::Emphasis) => {
                if let Some(Inline::Text(ref mut t)) = current_paragraph.last_mut() {
                    let content = t.clone();
                    *t = String::new();
                    current_paragraph.push(Inline::Italic(vec![Inline::Text(content)]));
                }
            }
            Event::Start(Tag::Strong) => {
                current_paragraph.push(Inline::Text(String::new()));
            }
            Event::End(TagEnd::Strong) => {
                if let Some(Inline::Text(ref mut t)) = current_paragraph.last_mut() {
                    let content = t.clone();
                    *t = String::new();
                    current_paragraph.push(Inline::Bold(vec![Inline::Text(content)]));
                }
            }
            Event::Start(Tag::Strikethrough) => {
                current_paragraph.push(Inline::Text(String::new()));
            }
            Event::End(TagEnd::Strikethrough) => {
                if let Some(Inline::Text(ref mut t)) = current_paragraph.last_mut() {
                    let content = t.clone();
                    *t = String::new();
                    current_paragraph.push(Inline::Strikethrough(vec![Inline::Text(content)]));
                }
            }
            Event::Code(code) => {
                current_paragraph.push(Inline::Code(code.to_string()));
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                current_paragraph.push(Inline::Text(String::new()));
                current_paragraph.push(Inline::Link {
                    text: vec![Inline::Text(dest_url.to_string())],
                    url: dest_url.to_string(),
                });
            }
            Event::End(TagEnd::Link) => {}
            Event::Start(Tag::Image {
                dest_url, title, ..
            }) => {
                doc.push_block(Block::Image {
                    src: dest_url.to_string(),
                    alt: title.to_string(),
                    width: None,
                    height: None,
                });
            }
            Event::End(TagEnd::Image) => {}
            _ => {}
        }
    }

    if !current_paragraph.is_empty() {
        doc.push_block(Block::Paragraph(current_paragraph));
    }
    if in_list && !list_items.is_empty() {
        doc.push_block(Block::List {
            ordered: list_ordered,
            items: list_items,
        });
    }

    Ok(doc)
}
