use std::path::Path;

use anyhow::{Context, Result};
use flip_ir::{Block, Document, Format};

pub fn render(doc: &Document, path: &Path, format: Format) -> Result<()> {
    for block in &doc.blocks {
        if let Block::Image { src, .. } = block {
            let src_path = Path::new(src);
            if src_path.exists() {
                let img = image::open(src_path)
                    .with_context(|| format!("Failed to open image: {}", src))?;

                match format {
                    Format::Png => img.save(path)?,
                    Format::Jpeg => img.save(path)?,
                    Format::Webp => img.save(path)?,
                    Format::Gif => img.save(path)?,
                    Format::Bmp => img.save(path)?,
                    Format::Tiff => img.save(path)?,
                    _ => anyhow::bail!("Unsupported image output format"),
                }

                return Ok(());
            }
        }
    }

    anyhow::bail!("No image block found in document — cannot convert")
}
