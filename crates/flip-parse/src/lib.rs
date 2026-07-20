pub mod csv_parser;
pub mod docx;
pub mod epub;
pub mod html;
pub mod image;
pub mod json;
pub mod markdown;
pub mod odp;
pub mod ods;
pub mod odt;
pub mod pdf;
pub mod pptx;
pub mod rtf;
pub mod svg;
pub mod text;
pub mod xlsx;
pub mod yaml;

use std::path::Path;

use anyhow::{Context, Result};
use flip_ir::{Document, Format};

pub fn parse_file(path: &Path, format: Format) -> Result<Document> {
    match format {
        Format::Pdf => pdf::parse(path),
        Format::Docx => docx::parse(path),
        Format::Xlsx => xlsx::parse(path),
        Format::Pptx => pptx::parse(path),
        Format::Html => html::parse(path),
        Format::Odp => odp::parse(path),
        Format::Markdown => markdown::parse(path),
        Format::Csv => csv_parser::parse(path),
        Format::Text => text::parse(path),
        Format::Epub => epub::parse(path),
        Format::Json => json::parse(path),
        Format::Yaml => yaml::parse(path),
        Format::Odt => odt::parse(path),
        Format::Ods => ods::parse(path),
        Format::Svg => svg::parse(path),
        Format::Rtf => rtf::parse(path),
        Format::Png | Format::Jpeg | Format::Webp | Format::Gif | Format::Bmp | Format::Tiff => {
            image::parse(path)
        }
        Format::Latex => {
            anyhow::bail!(
                "LaTeX parsing is not yet supported. Use --from to specify input format."
            );
        }
    }
    .with_context(|| format!("Failed to parse {} as {}", path.display(), format))
}
