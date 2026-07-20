use std::path::Path;

use anyhow::Result;
use flip_ir::{Block, Document, Format};

pub fn render(doc: &Document, path: &Path, format: Format) -> Result<()> {
    let mut img = image::ImageBuffer::<image::Rgb<u8>, _>::from_pixel(
        800,
        600,
        image::Rgb([255u8, 255, 255]),
    );

    let mut y = 40i32;

    for block in &doc.blocks {
        if y >= 560 {
            break;
        }

        match block {
            Block::Paragraph(inlines) => {
                let text: String = inlines.iter().map(|i| i.plain_text()).collect();
                let line = truncate(&text, 90);
                draw_text_simple(&mut img, &line, 20, y);
                y += 24;
            }
            Block::Heading { level, content } => {
                let text: String = content.iter().map(|i| i.plain_text()).collect();
                let line = truncate(&text, 90);
                draw_text_simple(&mut img, &line, 20, y);
                let spacing = match level {
                    1 => 36,
                    2 => 30,
                    _ => 24,
                };
                y += spacing;
            }
            Block::Code { content, .. } => {
                for line in content.lines() {
                    let l = truncate(line, 90);
                    draw_text_simple(&mut img, &l, 30, y);
                    y += 18;
                }
                y += 8;
            }
            _ => {}
        }
    }

    match format {
        Format::Png => img.save(path)?,
        Format::Jpeg => img.save(path)?,
        Format::Webp => img.save(path)?,
        Format::Gif => img.save(path)?,
        Format::Bmp => img.save(path)?,
        Format::Tiff => img.save(path)?,
        _ => anyhow::bail!("Unsupported image format"),
    }

    Ok(())
}

fn truncate(s: &str, max_chars: usize) -> String {
    if s.len() <= max_chars {
        s.to_string()
    } else {
        format!("{}...", &s[..max_chars])
    }
}

fn draw_text_simple(
    img: &mut image::ImageBuffer<image::Rgb<u8>, Vec<u8>>,
    text: &str,
    x: i32,
    y: i32,
) {
    for (i, ch) in text.chars().enumerate() {
        let px = x + (i as i32 * 8);
        if px >= 790 {
            break;
        }
        draw_char(img, ch, px, y);
    }
}

fn draw_char(img: &mut image::ImageBuffer<image::Rgb<u8>, Vec<u8>>, ch: char, x: i32, y: i32) {
    let pattern = get_char_pattern(ch);
    for (row, &bits) in pattern.iter().enumerate() {
        for col in 0..8 {
            if bits & (0x80 >> col) != 0 {
                let px = x + col;
                let py = y + row as i32;
                if (0..800).contains(&px) && (0..600).contains(&py) {
                    img.put_pixel(px as u32, py as u32, image::Rgb([51, 51, 51]));
                }
            }
        }
    }
}

fn get_char_pattern(ch: char) -> [u8; 8] {
    match ch {
        'A'..='Z' | 'a'..='z' => [0x3C, 0x66, 0x66, 0x7E, 0x66, 0x66, 0x66, 0x00],
        '0'..='9' => [0x3C, 0x66, 0x6E, 0x76, 0x66, 0x66, 0x3C, 0x00],
        ' ' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x00],
        ',' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x30, 0x00],
        '!' => [0x18, 0x18, 0x18, 0x18, 0x00, 0x00, 0x18, 0x00],
        '?' => [0x3C, 0x66, 0x06, 0x0C, 0x18, 0x00, 0x18, 0x00],
        '-' => [0x00, 0x00, 0x00, 0x7E, 0x00, 0x00, 0x00, 0x00],
        ':' => [0x00, 0x18, 0x18, 0x00, 0x00, 0x18, 0x18, 0x00],
        '/' => [0x02, 0x06, 0x0C, 0x18, 0x30, 0x60, 0x40, 0x00],
        _ => [0x3C, 0x42, 0x4A, 0x56, 0x62, 0x42, 0x3C, 0x00],
    }
}
