mod block;
mod inline;
mod meta;

pub use block::*;
pub use inline::*;
pub use meta::*;

use std::fmt;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Format {
    Pdf,
    Docx,
    Xlsx,
    Pptx,
    Html,
    Markdown,
    Csv,
    Text,
    Epub,
    Odt,
    Ods,
    Odp,
    Rtf,
    Svg,
    Json,
    Yaml,
    Latex,
    Png,
    Jpeg,
    Webp,
    Gif,
    Bmp,
    Tiff,
}

impl Format {
    pub fn all() -> &'static [Format] {
        &[
            Format::Pdf,
            Format::Docx,
            Format::Xlsx,
            Format::Pptx,
            Format::Html,
            Format::Markdown,
            Format::Csv,
            Format::Text,
            Format::Epub,
            Format::Odt,
            Format::Ods,
            Format::Odp,
            Format::Rtf,
            Format::Svg,
            Format::Json,
            Format::Yaml,
            Format::Latex,
            Format::Png,
            Format::Jpeg,
            Format::Webp,
            Format::Gif,
            Format::Bmp,
            Format::Tiff,
        ]
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Format::Pdf => "pdf",
            Format::Docx => "docx",
            Format::Xlsx => "xlsx",
            Format::Pptx => "pptx",
            Format::Html => "html",
            Format::Markdown => "md",
            Format::Csv => "csv",
            Format::Text => "txt",
            Format::Epub => "epub",
            Format::Odt => "odt",
            Format::Ods => "ods",
            Format::Odp => "odp",
            Format::Rtf => "rtf",
            Format::Svg => "svg",
            Format::Json => "json",
            Format::Yaml => "yaml",
            Format::Latex => "tex",
            Format::Png => "png",
            Format::Jpeg => "jpg",
            Format::Webp => "webp",
            Format::Gif => "gif",
            Format::Bmp => "bmp",
            Format::Tiff => "tiff",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Format::Pdf => "PDF",
            Format::Docx => "Word (DOCX)",
            Format::Xlsx => "Excel (XLSX)",
            Format::Pptx => "PowerPoint (PPTX)",
            Format::Html => "HTML",
            Format::Markdown => "Markdown",
            Format::Csv => "CSV",
            Format::Text => "Plain Text",
            Format::Epub => "EPUB",
            Format::Odt => "OpenDocument Text (ODT)",
            Format::Ods => "OpenDocument Spreadsheet (ODS)",
            Format::Odp => "OpenDocument Presentation (ODP)",
            Format::Rtf => "RTF",
            Format::Svg => "SVG",
            Format::Json => "JSON",
            Format::Yaml => "YAML",
            Format::Latex => "LaTeX",
            Format::Png => "PNG Image",
            Format::Jpeg => "JPEG Image",
            Format::Webp => "WebP Image",
            Format::Gif => "GIF Image",
            Format::Bmp => "BMP Image",
            Format::Tiff => "TIFF Image",
        }
    }

    pub fn is_image(&self) -> bool {
        matches!(
            self,
            Format::Png | Format::Jpeg | Format::Webp | Format::Gif | Format::Bmp | Format::Tiff
        )
    }

    pub fn is_office(&self) -> bool {
        matches!(
            self,
            Format::Docx | Format::Xlsx | Format::Pptx | Format::Odt | Format::Ods | Format::Odp
        )
    }
}

impl FromStr for Format {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pdf" => Ok(Format::Pdf),
            "docx" | "doc" => Ok(Format::Docx),
            "xlsx" | "xls" => Ok(Format::Xlsx),
            "pptx" | "ppt" => Ok(Format::Pptx),
            "html" | "htm" => Ok(Format::Html),
            "md" | "markdown" => Ok(Format::Markdown),
            "csv" | "tsv" => Ok(Format::Csv),
            "txt" | "text" => Ok(Format::Text),
            "epub" => Ok(Format::Epub),
            "odt" => Ok(Format::Odt),
            "ods" => Ok(Format::Ods),
            "odp" => Ok(Format::Odp),
            "rtf" => Ok(Format::Rtf),
            "svg" => Ok(Format::Svg),
            "json" | "jsonl" => Ok(Format::Json),
            "yaml" | "yml" => Ok(Format::Yaml),
            "tex" | "latex" => Ok(Format::Latex),
            "png" => Ok(Format::Png),
            "jpg" | "jpeg" => Ok(Format::Jpeg),
            "webp" => Ok(Format::Webp),
            "gif" => Ok(Format::Gif),
            "bmp" => Ok(Format::Bmp),
            "tiff" | "tif" => Ok(Format::Tiff),
            _ => Err(format!("Unknown format: {}", s)),
        }
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Document {
    pub metadata: Metadata,
    pub blocks: Vec<Block>,
}

impl Document {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_block(&mut self, block: Block) {
        self.blocks.push(block);
    }

    pub fn push_text(&mut self, text: impl Into<String>) {
        self.blocks
            .push(Block::Paragraph(vec![Inline::Text(text.into())]));
    }

    pub fn plain_text(&self) -> String {
        let mut out = String::new();
        for block in &self.blocks {
            match block {
                Block::Paragraph(inlines) => {
                    for inline in inlines {
                        out.push_str(&inline.plain_text());
                    }
                    out.push('\n');
                }
                Block::Heading { level: _, content } => {
                    for inline in content {
                        out.push_str(&inline.plain_text());
                    }
                    out.push('\n');
                }
                Block::Table { headers, rows } => {
                    for hrow in headers {
                        for hcell in hrow {
                            let t: String = hcell.iter().map(|i| i.plain_text()).collect();
                            out.push_str(&t);
                            out.push('\t');
                        }
                        out.push('\n');
                    }
                    for row in rows {
                        for cell in row {
                            let t: String = cell.iter().map(|i| i.plain_text()).collect();
                            out.push_str(&t);
                            out.push('\t');
                        }
                        out.push('\n');
                    }
                    out.push('\n');
                }
                Block::List { ordered: _, items } => {
                    for (i, item) in items.iter().enumerate() {
                        out.push_str(&format!("{}. ", i + 1));
                        for inline in item {
                            out.push_str(&inline.plain_text());
                        }
                        out.push('\n');
                    }
                    out.push('\n');
                }
                Block::Code {
                    language: _,
                    content,
                } => {
                    out.push_str(content);
                    out.push('\n');
                }
                Block::Blockquote(content) => {
                    for inline in content {
                        out.push_str(&inline.plain_text());
                    }
                    out.push('\n');
                }
                Block::Image { src: _, alt, .. } => {
                    out.push_str(alt);
                    out.push('\n');
                }
                Block::HorizontalRule => {
                    out.push_str("---\n");
                }
                Block::TableFromCsv(text) => {
                    out.push_str(text);
                    out.push('\n');
                }
            }
        }
        out
    }
}

pub fn detect_format(path: &Path) -> Option<Format> {
    let ext = path.extension()?.to_str()?.to_lowercase();
    match ext.as_str() {
        "pdf" => Some(Format::Pdf),
        "docx" => Some(Format::Docx),
        "xlsx" | "xls" => Some(Format::Xlsx),
        "pptx" | "ppt" => Some(Format::Pptx),
        "html" | "htm" => Some(Format::Html),
        "md" | "markdown" | "mdown" | "mkd" => Some(Format::Markdown),
        "csv" | "tsv" => Some(Format::Csv),
        "txt" | "text" | "log" => Some(Format::Text),
        "epub" => Some(Format::Epub),
        "odt" => Some(Format::Odt),
        "ods" => Some(Format::Ods),
        "odp" => Some(Format::Odp),
        "rtf" => Some(Format::Rtf),
        "svg" => Some(Format::Svg),
        "json" | "jsonl" => Some(Format::Json),
        "yaml" | "yml" => Some(Format::Yaml),
        "tex" | "latex" => Some(Format::Latex),
        "png" => Some(Format::Png),
        "jpg" | "jpeg" => Some(Format::Jpeg),
        "webp" => Some(Format::Webp),
        "gif" => Some(Format::Gif),
        "bmp" => Some(Format::Bmp),
        "tiff" | "tif" => Some(Format::Tiff),
        _ => None,
    }
}
