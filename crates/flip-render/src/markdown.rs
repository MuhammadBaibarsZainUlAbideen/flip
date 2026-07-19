use std::path::Path;

use anyhow::Result;
use flip_ir::{Block, Document, Inline};

pub fn render(doc: &Document, path: &Path) -> Result<()> {
    let bytes = render_bytes(doc)?;
    std::fs::write(path, bytes)?;
    Ok(())
}

pub fn render_bytes(doc: &Document) -> Result<Vec<u8>> {
    let mut md = String::new();

    if let Some(ref title) = doc.metadata.title {
        md.push_str(&format!("# {}\n\n", title));
    }

    for block in &doc.blocks {
        render_block(&mut md, block);
        md.push('\n');
    }

    Ok(md.into_bytes())
}

fn render_block(md: &mut String, block: &Block) {
    match block {
        Block::Paragraph(inlines) => {
            render_inlines(md, inlines);
            md.push('\n');
        }
        Block::Heading { level, content } => {
            for _ in 0..*level {
                md.push('#');
            }
            md.push(' ');
            render_inlines(md, content);
            md.push('\n');
        }
        Block::Code { language, content } => {
            if let Some(lang) = language {
                md.push_str(&format!("```{}\n", lang));
            } else {
                md.push_str("```\n");
            }
            md.push_str(content);
            md.push_str("\n```\n");
        }
        Block::List { ordered, items } => {
            for (i, item) in items.iter().enumerate() {
                if *ordered {
                    md.push_str(&format!("{}. ", i + 1));
                } else {
                    md.push_str("- ");
                }
                render_inlines(md, item);
                md.push('\n');
            }
        }
        Block::Blockquote(content) => {
            md.push_str("> ");
            render_inlines(md, content);
            md.push('\n');
        }
        Block::Table { headers, rows } => {
            if !headers.is_empty() {
                for row in headers {
                    for (i, cell) in row.iter().enumerate() {
                        md.push('|');
                        let text: String = cell.iter().map(|c| c.plain_text()).collect();
                        md.push_str(&text);
                        if i == row.len() - 1 {
                            md.push_str("|\n");
                        }
                    }
                }
                let num_cols = headers[0].len();
                md.push_str(&format!("|{}|\n", "---|".repeat(num_cols)));
            }
            for row in rows {
                for (i, cell) in row.iter().enumerate() {
                    md.push('|');
                    let text: String = cell.iter().map(|c| c.plain_text()).collect();
                    md.push_str(&text);
                    if i == row.len() - 1 {
                        md.push_str("|\n");
                    }
                }
            }
        }
        Block::Image { src, alt, .. } => {
            md.push_str(&format!("![{}]({})\n", alt, src));
        }
        Block::HorizontalRule => {
            md.push_str("---\n");
        }
        Block::TableFromCsv(text) => {
            md.push_str(text);
            md.push('\n');
        }
    }
}

fn render_inlines(md: &mut String, inlines: &[Inline]) {
    for inline in inlines {
        match inline {
            Inline::Text(text) => {
                md.push_str(text);
            }
            Inline::Bold(content) => {
                md.push_str("**");
                render_inlines(md, content);
                md.push_str("**");
            }
            Inline::Italic(content) => {
                md.push('*');
                render_inlines(md, content);
                md.push('*');
            }
            Inline::Strikethrough(content) => {
                md.push_str("~~");
                render_inlines(md, content);
                md.push_str("~~");
            }
            Inline::Code(text) => {
                md.push('`');
                md.push_str(text);
                md.push('`');
            }
            Inline::Link { text, url } => {
                md.push('[');
                render_inlines(md, text);
                md.push_str("](");
                md.push_str(url);
                md.push(')');
            }
            Inline::Image { alt, src } => {
                md.push_str(&format!("![{}]({})", alt, src));
            }
            Inline::Superscript(content) => {
                render_inlines(md, content);
            }
            Inline::Subscript(content) => {
                render_inlines(md, content);
            }
        }
    }
}
