use std::io::Cursor;
use std::path::Path;

use anyhow::{Context, Result};
use epub_builder::{EpubBuilder, EpubContent, ZipLibrary};

use flip_ir::{Block, Document};

pub fn render(doc: &Document, path: &Path) -> Result<()> {
    let zip = ZipLibrary::new().context("ZipLibrary::new failed")?;
    let mut builder = EpubBuilder::new(zip).context("EpubBuilder::new failed")?;

    if let Some(ref title) = doc.metadata.title {
        builder.metadata("title", title)?;
    } else {
        builder.metadata("title", "flip output")?;
    }
    if let Some(ref author) = doc.metadata.author {
        builder.metadata("author", author)?;
    }

    let mut html_content = String::new();
    html_content.push_str("<html><body>\n");

    for block in &doc.blocks {
        match block {
            Block::Paragraph(inlines) => {
                let text: String = inlines.iter().map(|i| i.plain_text()).collect();
                html_content.push_str(&format!("<p>{}</p>\n", text));
            }
            Block::Heading { level, content } => {
                let text: String = content.iter().map(|i| i.plain_text()).collect();
                let tag = format!("h{}", level);
                html_content.push_str(&format!("<{}>{}</{}>\n", tag, text, tag));
            }
            Block::Code { content, .. } => {
                html_content.push_str(&format!(
                    "<pre><code>{}</code></pre>\n",
                    content
                        .replace('&', "&amp;")
                        .replace('<', "&lt;")
                        .replace('>', "&gt;")
                ));
            }
            Block::List { items, .. } => {
                html_content.push_str("<ul>\n");
                for item in items {
                    let text: String = item.iter().map(|i| i.plain_text()).collect();
                    html_content.push_str(&format!("<li>{}</li>\n", text));
                }
                html_content.push_str("</ul>\n");
            }
            Block::Blockquote(content) => {
                let text: String = content.iter().map(|i| i.plain_text()).collect();
                html_content
                    .push_str(&format!("<blockquote>{}</blockquote>\n", text));
            }
            _ => {}
        }
    }

    html_content.push_str("</body></html>");

    builder
        .add_content(
            EpubContent::new("chapter_1.xhtml", Cursor::new(html_content.into_bytes()))
                .title("Content"),
        )
        .context("EPUB add_content failed")?;

    let mut output = Vec::new();
    builder.generate(&mut output).context("EPUB generate failed")?;
    std::fs::write(path, output)?;

    Ok(())
}
