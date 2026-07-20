use std::io::Write;
use std::path::Path;

use anyhow::Result;
use flip_ir::{Block, Document};
use zip::write::SimpleFileOptions;

pub fn render(doc: &Document, path: &Path) -> Result<()> {
    let mut content = String::new();
    content.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    content.push_str("<office:document-content xmlns:office=\"urn:oasis:names:tc:opendocument:xmlns:office:1.0\" xmlns:text=\"urn:oasis:names:tc:opendocument:xmlns:text:1.0\" xmlns:table=\"urn:oasis:names:tc:opendocument:xmlns:table:1.0\">\n");
    content.push_str("<office:body>\n<office:spreadsheet>\n");

    for block in &doc.blocks {
        match block {
            Block::Table { headers, rows } => {
                content.push_str("<table:table table:name=\"Sheet1\">\n");
                for header_row in headers {
                    content.push_str("<table:table-row>\n");
                    for cell in header_row {
                        let text: String = cell.iter().map(|i| i.plain_text()).collect();
                        content.push_str(&format!(
                            "<table:table-cell><text:p>{}</text:p></table:table-cell>\n",
                            xml_escape(&text)
                        ));
                    }
                    content.push_str("</table:table-row>\n");
                }
                for row in rows {
                    content.push_str("<table:table-row>\n");
                    for cell in row {
                        let text: String = cell.iter().map(|i| i.plain_text()).collect();
                        content.push_str(&format!(
                            "<table:table-cell><text:p>{}</text:p></table:table-cell>\n",
                            xml_escape(&text)
                        ));
                    }
                    content.push_str("</table:table-row>\n");
                }
                content.push_str("</table:table>\n");
            }
            _ => {
                let text = block_plain_text(block);
                if !text.trim().is_empty() {
                    content.push_str("<table:table table:name=\"Sheet1\"><table:table-row>\n");
                    content.push_str(&format!(
                        "<table:table-cell><text:p>{}</text:p></table:table-cell>\n",
                        xml_escape(&text)
                    ));
                    content.push_str("</table:table-row></table:table>\n");
                }
            }
        }
    }

    content.push_str("</office:spreadsheet>\n</office:body>\n</office:document-content>");

    let file = std::fs::File::create(path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("mimetype", options)?;
    zip.write_all(b"application/vnd.oasis.opendocument.spreadsheet")?;

    zip.start_file("content.xml", options)?;
    zip.write_all(content.as_bytes())?;
    zip.finish()?;

    Ok(())
}

fn block_plain_text(block: &Block) -> String {
    match block {
        Block::Paragraph(inlines) => inlines.iter().map(|i| i.plain_text()).collect(),
        Block::Heading { content, .. } => content.iter().map(|i| i.plain_text()).collect(),
        Block::Code { content, .. } => content.clone(),
        _ => String::new(),
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
