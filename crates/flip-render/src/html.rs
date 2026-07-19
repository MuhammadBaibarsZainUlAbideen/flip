use std::path::Path;

use anyhow::Result;
use flip_ir::{Block, Document, Inline};

pub fn render(doc: &Document, path: &Path) -> Result<()> {
    let bytes = render_bytes(doc)?;
    std::fs::write(path, bytes)?;
    Ok(())
}

pub fn render_bytes(doc: &Document) -> Result<Vec<u8>> {
    let mut html = String::new();

    html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
    html.push_str("<meta charset=\"UTF-8\">\n");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");

    if let Some(ref title) = doc.metadata.title {
        html.push_str(&format!("<title>{}</title>\n", escape_html(title)));
    } else {
        html.push_str("<title>flip output</title>\n");
    }

    html.push_str("<style>\n");
    html.push_str(include_str!("style.css"));
    html.push_str("</style>\n");
    html.push_str("</head>\n<body>\n<main>\n");

    for block in &doc.blocks {
        render_block(&mut html, block);
    }

    html.push_str("</main>\n</body>\n</html>");

    Ok(html.into_bytes())
}

fn render_block(html: &mut String, block: &Block) {
    match block {
        Block::Paragraph(inlines) => {
            html.push_str("<p>");
            render_inlines(html, inlines);
            html.push_str("</p>\n");
        }
        Block::Heading { level, content } => {
            let tag = format!("h{}", level);
            html.push_str(&format!("<{}>", tag));
            render_inlines(html, content);
            html.push_str(&format!("</{}>\n", tag));
        }
        Block::Code { language, content } => {
            if let Some(lang) = language {
                html.push_str(&format!(
                    "<pre><code class=\"language-{}\">{}</code></pre>\n",
                    escape_html(lang),
                    escape_html(content)
                ));
            } else {
                html.push_str(&format!(
                    "<pre><code>{}</code></pre>\n",
                    escape_html(content)
                ));
            }
        }
        Block::List { ordered, items } => {
            let tag = if *ordered { "ol" } else { "ul" };
            html.push_str(&format!("<{}>\n", tag));
            for item in items {
                html.push_str("<li>");
                render_inlines(html, item);
                html.push_str("</li>\n");
            }
            html.push_str(&format!("</{}>\n", tag));
        }
        Block::Blockquote(content) => {
            html.push_str("<blockquote>");
            render_inlines(html, content);
            html.push_str("</blockquote>\n");
        }
        Block::Table { headers, rows } => {
            html.push_str("<table>\n");
            if !headers.is_empty() {
                html.push_str("<thead>\n");
                for header_row in headers {
                    html.push_str("<tr>\n");
                    for cell in header_row {
                        html.push_str("<th>");
                        render_inlines(html, cell);
                        html.push_str("</th>\n");
                    }
                    html.push_str("</tr>\n");
                }
                html.push_str("</thead>\n");
            }
            if !rows.is_empty() {
                html.push_str("<tbody>\n");
                for row in rows {
                    html.push_str("<tr>\n");
                    for cell in row {
                        html.push_str("<td>");
                        render_inlines(html, cell);
                        html.push_str("</td>\n");
                    }
                    html.push_str("</tr>\n");
                }
                html.push_str("</tbody>\n");
            }
            html.push_str("</table>\n");
        }
        Block::Image { src, alt, .. } => {
            html.push_str(&format!(
                "<figure><img src=\"{}\" alt=\"{}\"><figcaption>{}</figcaption></figure>\n",
                escape_html(src),
                escape_html(alt),
                escape_html(alt)
            ));
        }
        Block::HorizontalRule => {
            html.push_str("<hr>\n");
        }
        Block::TableFromCsv(text) => {
            html.push_str("<pre>");
            html.push_str(&escape_html(text));
            html.push_str("</pre>\n");
        }
    }
}

fn render_inlines(html: &mut String, inlines: &[Inline]) {
    for inline in inlines {
        match inline {
            Inline::Text(text) => {
                html.push_str(&escape_html(text));
            }
            Inline::Bold(content) => {
                html.push_str("<strong>");
                render_inlines(html, content);
                html.push_str("</strong>");
            }
            Inline::Italic(content) => {
                html.push_str("<em>");
                render_inlines(html, content);
                html.push_str("</em>");
            }
            Inline::Strikethrough(content) => {
                html.push_str("<del>");
                render_inlines(html, content);
                html.push_str("</del>");
            }
            Inline::Code(text) => {
                html.push_str(&format!("<code>{}</code>", escape_html(text)));
            }
            Inline::Link { text, url } => {
                html.push_str(&format!("<a href=\"{}\">", escape_html(url)));
                render_inlines(html, text);
                html.push_str("</a>");
            }
            Inline::Image { alt, src } => {
                html.push_str(&format!(
                    "<img src=\"{}\" alt=\"{}\">",
                    escape_html(src),
                    escape_html(alt)
                ));
            }
            Inline::Superscript(content) => {
                html.push_str("<sup>");
                render_inlines(html, content);
                html.push_str("</sup>");
            }
            Inline::Subscript(content) => {
                html.push_str("<sub>");
                render_inlines(html, content);
                html.push_str("</sub>");
            }
        }
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
