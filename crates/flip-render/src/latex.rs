use std::path::Path;

use anyhow::Result;
use flip_ir::{Block, Document, Inline};

pub fn render(doc: &Document, path: &Path) -> Result<()> {
    let mut tex = String::new();

    tex.push_str("\\documentclass[12pt]{article}\n");
    tex.push_str("\\usepackage[utf8]{inputenc}\n");
    tex.push_str("\\usepackage{geometry}\n");
    tex.push_str("\\geometry{a4paper, margin=1in}\n");
    tex.push_str("\\usepackage{hyperref}\n");
    tex.push_str("\\usepackage{listings}\n");
    tex.push_str("\\usepackage{graphicx}\n");
    tex.push('\n');

    if let Some(ref title) = doc.metadata.title {
        tex.push_str(&format!("\\title{{{}}}\n", escape_latex(title)));
    }
    if let Some(ref author) = doc.metadata.author {
        tex.push_str(&format!("\\author{{{}}}\n", escape_latex(author)));
    }
    tex.push_str("\\date{\\today}\n\n");
    tex.push_str("\\begin{document}\n");
    tex.push_str("\\maketitle\n\n");

    for block in &doc.blocks {
        render_block(&mut tex, block);
    }

    tex.push_str("\\end{document}\n");

    std::fs::write(path, tex)?;
    Ok(())
}

fn render_block(tex: &mut String, block: &Block) {
    match block {
        Block::Paragraph(inlines) => {
            let text = inlines_to_latex(inlines);
            tex.push_str(&text);
            tex.push_str("\n\n");
        }
        Block::Heading { level, content } => {
            let text: String = content.iter().map(|i| i.plain_text()).collect();
            let cmd = match level {
                1 => "\\section",
                2 => "\\subsection",
                3 => "\\subsubsection",
                4 => "\\paragraph",
                _ => "\\subparagraph",
            };
            tex.push_str(&format!("{}{{{}}}\n\n", cmd, escape_latex(&text)));
        }
        Block::Code {
            language: _,
            content,
        } => {
            tex.push_str("\\begin{lstlisting}\n");
            tex.push_str(content);
            tex.push_str("\n\\end{lstlisting}\n\n");
        }
        Block::List { ordered, items } => {
            let env = if *ordered { "enumerate" } else { "itemize" };
            tex.push_str(&format!("\\begin{{{}}}\n", env));
            for item in items {
                let text: String = item.iter().map(|i| i.plain_text()).collect();
                tex.push_str(&format!("\\item {}\n", escape_latex(&text)));
            }
            tex.push_str(&format!("\\end{{{}}}\n\n", env));
        }
        Block::Blockquote(content) => {
            let text: String = content.iter().map(|i| i.plain_text()).collect();
            tex.push_str(&format!(
                "\\begin{{quote}}\n{}\n\\end{{quote}}\n\n",
                escape_latex(&text)
            ));
        }
        Block::Image { src, alt, .. } => {
            tex.push_str(&format!(
                "\\begin{{figure}}[h]\n\\centering\n\\includegraphics[width=0.8\\textwidth]{{{}}}\n\\caption{{{}}}\n\\end{{figure}}\n\n",
                escape_latex(src),
                escape_latex(alt)
            ));
        }
        Block::HorizontalRule => {
            tex.push_str("\\hrule\n\n");
        }
        _ => {}
    }
}

fn inlines_to_latex(inlines: &[Inline]) -> String {
    let mut text = String::new();
    for inline in inlines {
        match inline {
            Inline::Text(t) => text.push_str(&escape_latex(t)),
            Inline::Bold(content) => {
                text.push_str(&format!("\\textbf{{{}}}", inlines_to_latex(content)));
            }
            Inline::Italic(content) => {
                text.push_str(&format!("\\textit{{{}}}", inlines_to_latex(content)));
            }
            Inline::Code(t) => {
                text.push_str(&format!("\\texttt{{{}}}", escape_latex(t)));
            }
            Inline::Link {
                text: link_text,
                url,
            } => {
                let display: String = link_text.iter().map(|i| i.plain_text()).collect();
                text.push_str(&format!(
                    "\\href{{{}}}{{{}}}",
                    escape_latex(url),
                    escape_latex(&display)
                ));
            }
            _ => {
                text.push_str(&inline.plain_text());
            }
        }
    }
    text
}

fn escape_latex(s: &str) -> String {
    s.replace('\\', "\\textbackslash{}")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('_', "\\_")
        .replace('#', "\\#")
        .replace('$', "\\$")
        .replace('%', "\\%")
        .replace('&', "\\&")
        .replace('^', "\\^{}")
        .replace('~', "\\~{}")
}
