<div align="center">

# `flip`

**Universal offline file converter -- flip between 20+ formats**

[![CI](https://github.com/flip-cli/flip/actions/workflows/ci.yml/badge.svg)](https://github.com/flip-cli/flip/actions)
[![Crates.io](https://img.shields.io/crates/v/flip.svg)](https://crates.io/crates/flip)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

```
flip report.pdf report.docx
flip *.csv --to html
flip presentation.pptx slides.md
```

</div>

---

## What is this?

`flip` is a fast, offline, single-binary file converter written in Rust. Convert between PDFs, Word docs, spreadsheets, presentations, HTML, Markdown, CSV, EPUB, images, and more -- no internet required.

**Your files never leave your machine.**

## Install

```bash
cargo install flip
```

That's it. `flip` is now available globally from any directory.

Or grab a [pre-built binary](https://github.com/flip-cli/flip/releases) for your platform (no Rust needed).

## Quick Start

```bash
# Convert a single file
flip document.pdf --to html

# Batch convert
flip *.csv --to json

# Specify where to save
flip report.md --to pdf -o ./output/my_report.pdf

# Preview what would happen
flip *.html --to pdf --dry-run

# See all formats
flip formats
```

## Supported Formats

| Format | Parse (read) | Render (write) |
|---|:---:|:---:|
| PDF (`.pdf`) | :white_check_mark: | :white_check_mark: |
| Word (`.docx`) | :white_check_mark: | :white_check_mark: |
| Excel (`.xlsx`) | :white_check_mark: | :white_check_mark: |
| PowerPoint (`.pptx`) | :white_check_mark: | :white_check_mark: |
| HTML (`.html`) | :white_check_mark: | :white_check_mark: |
| Markdown (`.md`) | :white_check_mark: | :white_check_mark: |
| CSV (`.csv`) | :white_check_mark: | :white_check_mark: |
| Plain Text (`.txt`) | :white_check_mark: | :white_check_mark: |
| EPUB (`.epub`) | :white_check_mark: | :white_check_mark: |
| ODT (`.odt`) | :white_check_mark: | :white_check_mark: |
| ODS (`.ods`) | :white_check_mark: | :white_check_mark: |
| ODP (`.odp`) | :white_check_mark: | :white_check_mark: |
| SVG (`.svg`) | :white_check_mark: | :white_check_mark: |
| JSON (`.json`) | :white_check_mark: | :white_check_mark: |
| YAML (`.yaml`) | :white_check_mark: | :white_check_mark: |
| LaTeX (`.tex`) | | :white_check_mark: |
| PNG (`.png`) | :white_check_mark: | :white_check_mark: |
| JPEG (`.jpg`) | :white_check_mark: | :white_check_mark: |
| WebP (`.webp`) | :white_check_mark: | :white_check_mark: |
| GIF (`.gif`) | :white_check_mark: | :white_check_mark: |
| BMP (`.bmp`) | :white_check_mark: | :white_check_mark: |
| TIFF (`.tiff`) | :white_check_mark: | :white_check_mark: |

## CLI Reference

```
Usage: flip [OPTIONS] [INPUTS]... [COMMAND]

Commands:
  formats  List all supported formats
  matrix   Show the conversion matrix
  version  Show version information

Options:
  -t, --to <FORMAT>      Output format (e.g., pdf, html, json, csv, txt)
  -f, --from <FORMAT>    Force input format (auto-detected by default)
  -o, --out <PATH>       Output file or directory
  -v, --verbose          Verbose output
      --dry-run          Preview without converting
  -h, --help             Print help
  -V, --version          Print version
```

## How It Works

```
                  +----------+
                  |  Input   |
                  |  Format  |
                  +----+-----+
                       |
                  +----v-----+
                  |  Parser  |
                  |   (N)    |
                  +----+-----+
                       |
              +--------v--------+
              |   Document IR   |
              |     (hub)       |
              +--------+--------+
                       |
                  +----v-----+
                  | Renderer |
                  |   (M)    |
                  +----+-----+
                       |
                  +----v-----+
                  |  Output  |
                  |  Format  |
                  +----------+
```

`flip` uses a **hub-and-spoke architecture** with a Document IR (Intermediate Representation). Adding a new format requires only 1 parser and 1 renderer -- not N x M converters.

**Crates:**
- `flip-ir` -- The Document IR (shared data types)
- `flip-parse` -- Input parsers (20+ formats -> IR)
- `flip-render` -- Output renderers (IR -> 20+ formats)
- `flip-cli` -- The CLI interface

## Why `flip`?

| | `flip` | `pandoc` |
|---|---|---|
| **Install** | `cargo install flip` | System package manager |
| **Binary size** | Small (single Rust binary) | ~100 MB |
| **Offline** | 100% -- no cloud, no API | Yes |
| **Privacy** | Zero network calls | Zero |
| **Speed** | Native binary, startup in ms | Fast |
| **Office formats** | PDF, DOCX, XLSX, PPTX, EPUB, ODT/ODS/ODP | 40+ formats |
| **Images** | PNG, JPG, WebP, GIF, BMP, TIFF | Limited |

`flip` focuses on being **fast, private, and easy to install**. If you need 40+ formats or complex document features, pandoc is great. If you want a quick, lightweight converter that just works, try `flip`.

## Contributing

Contributions are welcome! Here's how to get started:

```bash
git clone https://github.com/flip-cli/flip.git
cd flip
cargo build
cargo test
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Adding a New Format

1. Add parser in `crates/flip-parse/src/` (input format -> Document IR)
2. Add renderer in `crates/flip-render/src/` (Document IR -> output format)
3. Register in `crates/flip-parse/src/lib.rs` and `crates/flip-render/src/lib.rs`
4. Add format enum variants in `crates/flip-ir/src/`

## License

MIT -- see [LICENSE](LICENSE) for details.

---

<div align="center">

**Built with Rust**

[GitHub](https://github.com/flip-cli/flip) | [Issues](https://github.com/flip-cli/flip/issues) | [Releases](https://github.com/flip-cli/flip/releases)

</div>
