use std::path::Path;

use anyhow::Result;
use flip_ir::{Block, Document};
use serde::Serialize;

pub fn render(doc: &Document, path: &Path) -> Result<()> {
    let output = doc_to_yaml(doc)?;
    std::fs::write(path, output)?;
    Ok(())
}

pub fn render_bytes(doc: &Document) -> Result<Vec<u8>> {
    Ok(doc_to_yaml(doc)?.into_bytes())
}

fn doc_to_yaml(doc: &Document) -> Result<String> {
    #[derive(Serialize)]
    struct YamlDoc {
        metadata: YamlMetadata,
        blocks: Vec<YamlBlock>,
    }

    #[derive(Serialize)]
    struct YamlMetadata {
        title: Option<String>,
        author: Option<String>,
        source_format: Option<String>,
    }

    #[derive(Serialize)]
    struct YamlBlock {
        #[serde(rename = "type")]
        block_type: String,
        content: Option<String>,
        level: Option<u8>,
        #[serde(skip_serializing_if = "Option::is_none")]
        language: Option<String>,
    }

    let mut blocks = Vec::new();
    for block in &doc.blocks {
        match block {
            Block::Paragraph(inlines) => {
                let text: String = inlines.iter().map(|i| i.plain_text()).collect();
                blocks.push(YamlBlock {
                    block_type: "paragraph".to_string(),
                    content: Some(text),
                    level: None,
                    language: None,
                });
            }
            Block::Heading { level, content } => {
                let text: String = content.iter().map(|i| i.plain_text()).collect();
                blocks.push(YamlBlock {
                    block_type: "heading".to_string(),
                    content: Some(text),
                    level: Some(*level),
                    language: None,
                });
            }
            Block::Code { language, content } => {
                blocks.push(YamlBlock {
                    block_type: "code".to_string(),
                    content: Some(content.clone()),
                    level: None,
                    language: language.clone(),
                });
            }
            Block::Blockquote(content) => {
                let text: String = content.iter().map(|i| i.plain_text()).collect();
                blocks.push(YamlBlock {
                    block_type: "blockquote".to_string(),
                    content: Some(text),
                    level: None,
                    language: None,
                });
            }
            _ => {}
        }
    }

    let output = YamlDoc {
        metadata: YamlMetadata {
            title: doc.metadata.title.clone(),
            author: doc.metadata.author.clone(),
            source_format: doc.metadata.source_format.clone(),
        },
        blocks,
    };

    Ok(serde_yaml::to_string(&output)?)
}
