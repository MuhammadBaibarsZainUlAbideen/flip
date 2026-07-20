use std::io::Write;
use std::path::Path;

use anyhow::Result;
use flip_ir::{Block, Document};
use zip::write::SimpleFileOptions;

pub fn render(doc: &Document, path: &Path) -> Result<()> {
    let mut content = String::new();
    content.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    content.push_str("<office:document-content xmlns:office=\"urn:oasis:names:tc:opendocument:xmlns:office:1.0\" xmlns:text=\"urn:oasis:names:tc:opendocument:xmlns:text:1.0\">\n");
    content.push_str("<office:body>\n<office:text>\n");

    for block in &doc.blocks {
        match block {
            Block::Paragraph(inlines) => {
                let text: String = inlines.iter().map(|i| i.plain_text()).collect();
                content.push_str(&format!("<text:p>{}</text:p>\n", xml_escape(&text)));
            }
            Block::Heading {
                level,
                content: heading_content,
            } => {
                let text: String = heading_content.iter().map(|i| i.plain_text()).collect();
                content.push_str(&format!(
                    "<text:heading text:outline-level=\"{}\">{}</text:heading>\n",
                    level,
                    xml_escape(&text)
                ));
            }
            _ => {}
        }
    }

    content.push_str("</office:text>\n</office:body>\n</office:document-content>");

    let file = std::fs::File::create(path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("mimetype", options)?;
    zip.write_all(b"application/vnd.oasis.opendocument.text")?;

    zip.start_file("content.xml", options)?;
    zip.write_all(content.as_bytes())?;
    zip.finish()?;

    Ok(())
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
