pub mod pdf;
pub mod docx;
pub mod xlsx;
pub mod pptx;
pub mod html;
pub mod markdown;
pub mod csv_render;
pub mod text;
pub mod image_render;
pub mod epub;
pub mod json;
pub mod yaml;
pub mod latex;
pub mod odt;
pub mod ods;
pub mod odp;
pub mod svg_render;

use std::path::Path;

use anyhow::{Context, Result};
use flip_ir::{Document, Format};

pub fn render_file(doc: &Document, path: &Path, format: Format) -> Result<()> {
    match format {
        Format::Pdf => pdf::render(doc, path),
        Format::Docx => docx::render(doc, path),
        Format::Xlsx => xlsx::render(doc, path),
        Format::Pptx => pptx::render(doc, path),
        Format::Html => html::render(doc, path),
        Format::Markdown => markdown::render(doc, path),
        Format::Csv => csv_render::render(doc, path),
        Format::Text => text::render(doc, path),
        Format::Epub => epub::render(doc, path),
        Format::Json => json::render(doc, path),
        Format::Yaml => yaml::render(doc, path),
        Format::Latex => latex::render(doc, path),
        Format::Odt => odt::render(doc, path),
        Format::Ods => ods::render(doc, path),
        Format::Odp => odp::render(doc, path),
        Format::Svg => svg_render::render(doc, path),
        Format::Rtf => {
            anyhow::bail!("RTF output is not yet supported. Try --to txt or --to html.");
        }
        Format::Png | Format::Jpeg | Format::Webp | Format::Gif | Format::Bmp | Format::Tiff => {
            image_render::render(doc, path, format)
        }
    }
    .with_context(|| format!("Failed to render as {} to {}", format, path.display()))
}

pub fn render_bytes(doc: &Document, format: Format) -> Result<Vec<u8>> {
    match format {
        Format::Pdf => pdf::render_bytes(doc),
        Format::Docx => docx::render_bytes(doc),
        Format::Html => html::render_bytes(doc),
        Format::Markdown => markdown::render_bytes(doc),
        Format::Text => text::render_bytes(doc),
        Format::Json => json::render_bytes(doc),
        Format::Yaml => yaml::render_bytes(doc),
        Format::Csv => csv_render::render_bytes(doc),
        Format::Svg => svg_render::render_bytes(doc),
        _ => anyhow::bail!("Bytes output not supported for this format"),
    }
}
