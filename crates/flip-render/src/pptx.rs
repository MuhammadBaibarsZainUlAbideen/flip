use std::io::Write;
use std::path::Path;

use anyhow::Result;
use flip_ir::{Block, Document};
use zip::write::SimpleFileOptions;

pub fn render(doc: &Document, path: &Path) -> Result<()> {
    let file = std::fs::File::create(path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    write_content_types(&mut zip, &options)?;
    write_rels(&mut zip, &options)?;
    write_presentation_xml(&mut zip, &options)?;

    let mut slide_num = 1u32;
    for block in &doc.blocks {
        let text = match block {
            Block::Paragraph(inlines) => {
                inlines.iter().map(|i| i.plain_text()).collect::<String>()
            }
            Block::Heading { content, .. } => {
                content.iter().map(|i| i.plain_text()).collect::<String>()
            }
            Block::List { items, .. } => items
                .iter()
                .map(|item| {
                    let t: String = item.iter().map(|i| i.plain_text()).collect();
                    format!("• {}", t)
                })
                .collect::<Vec<_>>()
                .join("\n"),
            Block::Code { content, .. } => content.clone(),
            Block::Blockquote(content) => {
                let t: String = content.iter().map(|i| i.plain_text()).collect();
                format!("\"{}\"", t)
            }
            _ => continue,
        };

        if text.trim().is_empty() {
            continue;
        }

        write_slide(&mut zip, &options, slide_num, &text)?;
        slide_num += 1;
    }

    if slide_num == 1 {
        write_slide(&mut zip, &options, 1, "flip output")?;
    }

    write_presentation_rels(&mut zip, &options, slide_num - 1)?;

    zip.finish()?;
    Ok(())
}

fn write_content_types(
    zip: &mut zip::ZipWriter<std::fs::File>,
    options: &SimpleFileOptions,
) -> Result<()> {
    zip.start_file("[Content_Types].xml", options.clone())?;
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>
  <Override PartName="/ppt/slides/slide1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>
</Types>"#,
    )?;
    Ok(())
}

fn write_rels(
    zip: &mut zip::ZipWriter<std::fs::File>,
    options: &SimpleFileOptions,
) -> Result<()> {
    zip.start_file("_rels/.rels", options.clone())?;
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/>
</Relationships>"#,
    )?;
    Ok(())
}

fn write_presentation_xml(
    zip: &mut zip::ZipWriter<std::fs::File>,
    options: &SimpleFileOptions,
) -> Result<()> {
    zip.start_file("ppt/presentation.xml", options.clone())?;
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentation xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:sldMasterIdLst/>
  <p:sldIdLst/>
  <p:sldSz cx="9144000" cy="6858000"/>
  <p:notesSz cx="6858000" cy="9144000"/>
</p:presentation>"#,
    )?;
    Ok(())
}

fn write_presentation_rels(
    zip: &mut zip::ZipWriter<std::fs::File>,
    options: &SimpleFileOptions,
    slide_count: u32,
) -> Result<()> {
    zip.start_file("ppt/_rels/presentation.xml.rels", options.clone())?;
    let mut rels = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
    );
    for i in 1..=slide_count {
        rels.push_str(&format!(
            r#"
  <Relationship Id="rId{i}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide{i}.xml"/>"#
        ));
    }
    rels.push_str("\n</Relationships>");
    zip.write_all(rels.as_bytes())?;
    Ok(())
}

fn write_slide(
    zip: &mut zip::ZipWriter<std::fs::File>,
    options: &SimpleFileOptions,
    num: u32,
    text: &str,
) -> Result<()> {
    let path = format!("ppt/slides/slide{}.xml", num);
    zip.start_file(&path, options.clone())?;

    let escaped = text
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");

    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:cSld>
    <p:spTree>
      <p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
      <p:grpSpPr/>
      <p:sp>
        <p:nvSpPr><p:cNvPr id="2" name=""/><p:cNvSpPr><a:spLocks noGrp="1"/></p:cNvSpPr><p:nvPr/></p:nvSpPr>
        <p:spPr>
          <a:xfrm><a:off x="457200" y="274638"/><a:ext cx="8229600" cy="4114800"/></a:xfrm>
        </p:spPr>
        <p:txBody>
          <a:bodyPr/>
          <a:lstStyle/>
          <a:p><a:r><a:rPr lang="en-US" sz="2400"/><a:t>{}</a:t></a:r></a:p>
        </p:txBody>
      </p:sp>
    </p:spTree>
  </p:cSld>
</p:sld>"#,
        escaped
    );
    zip.write_all(xml.as_bytes())?;

    let rels_path = format!("ppt/slides/_rels/slide{}.xml.rels", num);
    zip.start_file(&rels_path, options.clone())?;
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
</Relationships>"#,
    )?;

    Ok(())
}
