use std::path::Path;

use anyhow::Result;
use flip_ir::{Block, Document};

pub fn render(doc: &Document, path: &Path) -> Result<()> {
    let bytes = render_bytes(doc)?;
    std::fs::write(path, bytes)?;
    Ok(())
}

pub fn render_bytes(doc: &Document) -> Result<Vec<u8>> {
    let mut svg = String::new();
    let width = 800;
    let height = 600;

    svg.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\">\n",
        width, height, width, height
    ));
    svg.push_str("<rect width=\"100%\" height=\"100%\" fill=\"white\"/>\n");

    let mut y = 40i32;

    for block in &doc.blocks {
        if y >= 560 {
            break;
        }

        match block {
            Block::Paragraph(inlines) => {
                let text: String = inlines.iter().map(|i| i.plain_text()).collect();
                let escaped = xml_escape(&text);
                svg.push_str(&format!(
                    "<text x=\"20\" y=\"{}\" font-family=\"sans-serif\" font-size=\"14\" fill=\"#333\">{}</text>\n",
                    y, escaped
                ));
                y += 24;
            }
            Block::Heading { level, content } => {
                let text: String = content.iter().map(|i| i.plain_text()).collect();
                let escaped = xml_escape(&text);
                let size: i32 = match level {
                    1 => 24,
                    2 => 20,
                    3 => 16,
                    _ => 14,
                };
                svg.push_str(&format!(
                    "<text x=\"20\" y=\"{}\" font-family=\"sans-serif\" font-size=\"{}\" font-weight=\"bold\" fill=\"#000\">{}</text>\n",
                    y, size, escaped
                ));
                y += (size as f64 * 1.5) as i32;
            }
            Block::Code { content, .. } => {
                for line in content.lines() {
                    let escaped = xml_escape(line);
                    svg.push_str(&format!(
                        "<text x=\"30\" y=\"{}\" font-family=\"monospace\" font-size=\"12\" fill=\"#555\">{}</text>\n",
                        y, escaped
                    ));
                    y += 18;
                }
                y += 8;
            }
            _ => {}
        }
    }

    svg.push_str("</svg>");
    Ok(svg.into_bytes())
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
