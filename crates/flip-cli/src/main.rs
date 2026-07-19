use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueHint};
use colored::Colorize;
use flip_ir::{detect_format, Document, Format};
use flip_parse::parse_file;
use flip_render::{render_bytes, render_file};
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Parser)]
#[command(
    name = "flip",
    about = "Universal offline file converter — flip between 20+ formats",
    version,
    after_help = "Examples:\n  flip report.pdf report.docx\n  flip *.csv --to html\n  flip --from md --to pdf < input.md"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Input file(s) to convert
    #[arg(value_hint = ValueHint::FilePath, num_args = 0..)]
    inputs: Vec<PathBuf>,

    /// Output format (e.g., pdf, docx, html, md, csv, txt, json)
    #[arg(short, long, value_name = "FORMAT")]
    to: Option<String>,

    /// Input format override (auto-detected if not specified)
    #[arg(short, long, value_name = "FORMAT")]
    from: Option<String>,

    /// Output file or directory
    #[arg(short, long, value_name = "PATH")]
    out: Option<PathBuf>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Dry run — show what would be done without converting
    #[arg(long)]
    dry_run: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// List all supported formats
    Formats,

    /// Show the conversion matrix (what can convert to what)
    Matrix,

    /// Show version information
    Version,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Formats) => {
            print_formats();
            return Ok(());
        }
        Some(Commands::Matrix) => {
            print_matrix();
            return Ok(());
        }
        Some(Commands::Version) => {
            println!("flip {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        None => {}
    }

    if cli.inputs.is_empty() {
        // Try reading from stdin
        if atty::is(atty::Stream::Stdin) {
            eprintln!(
                "{}",
                "No input files specified. Use 'flip --help' for usage.".yellow()
            );
            std::process::exit(1);
        }

        // Read from stdin, write to stdout
        let _from_format = cli
            .from
            .as_deref()
            .map(Format::from_str)
            .transpose()
            .map_err(|e| anyhow::anyhow!(e))?
            .unwrap_or(Format::Text);

        let to_format = cli
            .to
            .as_deref()
            .map(Format::from_str)
            .transpose()
            .map_err(|e| anyhow::anyhow!(e))?
            .unwrap_or(Format::Text);

        let mut doc = Document::default();
        let mut buf = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf)?;
        doc.push_block(flip_ir::Block::paragraph(buf));

        let output = render_bytes(&doc, to_format)?;
        std::io::Write::write_all(&mut std::io::stdout(), &output)?;
        return Ok(());
    }

    let out_format = cli
        .to
        .as_deref()
        .map(Format::from_str)
        .transpose()
        .map_err(|e| anyhow::anyhow!(e))?;

    let from_override = cli
        .from
        .as_deref()
        .map(Format::from_str)
        .transpose()
        .map_err(|e| anyhow::anyhow!(e))?;

    let pb = if cli.inputs.len() > 1 && !cli.dry_run {
        let pb = ProgressBar::new(cli.inputs.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                .unwrap()
                .progress_chars("#>-"),
        );
        Some(pb)
    } else {
        None
    };

    let mut success_count = 0u32;
    let mut error_count = 0u32;

    for input in &cli.inputs {
        if !input.exists() {
            eprintln!("{}: {}", "Error".red().bold(), format!("File not found: {}", input.display()));
            error_count += 1;
            continue;
        }

        let input_format = from_override.or_else(|| detect_format(input));

        let input_format = match input_format {
            Some(f) => f,
            None => {
                eprintln!(
                    "{}: {}",
                    "Error".red().bold(),
                    format!("Cannot detect format for: {}", input.display())
                );
                error_count += 1;
                continue;
            }
        };

        let output_path = determine_output_path(input, &cli.out, out_format, input_format)?;

        let output_format = out_format.unwrap_or_else(|| {
            detect_format(&output_path).unwrap_or(Format::Text)
        });

        if cli.dry_run {
            println!(
                "{} {} -> {} {}",
                "Would convert:".cyan(),
                input.display().to_string().white().bold(),
                output_path.display().to_string().green().bold(),
                format!("({})", output_format.display_name()).dimmed()
            );
            continue;
        }

        if cli.verbose {
            println!(
                "{} {} -> {} ({})",
                "Converting:".cyan(),
                input.display().to_string().white().bold(),
                output_path.display().to_string().green().bold(),
                format!("{} -> {}", input_format, output_format).dimmed()
            );
        }

        match convert_file(input, input_format, &output_path, output_format) {
            Ok(bytes_written) => {
                success_count += 1;
                if cli.verbose {
                    println!(
                        "  {} {} ({} bytes)",
                        "Done!".green().bold(),
                        output_path.display(),
                        bytes_written
                    );
                }
            }
            Err(e) => {
                error_count += 1;
                eprintln!(
                    "{}: {} -> {}: {}",
                    "Error".red().bold(),
                    input.display(),
                    output_path.display(),
                    e
                );
            }
        }

        if let Some(ref pb) = pb {
            pb.inc(1);
        }
    }

    if let Some(pb) = pb {
        pb.finish_with_message("done");
    }

    if cli.inputs.len() > 1 {
        println!(
            "\n{} {} converted, {} failed",
            "Summary:".bold(),
            success_count.to_string().green(),
            if error_count > 0 {
                error_count.to_string().red()
            } else {
                error_count.to_string().green()
            }
        );
    }

    if error_count > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn convert_file(
    input: &Path,
    input_format: Format,
    output: &Path,
    output_format: Format,
) -> Result<u64> {
    let doc = parse_file(input, input_format)
        .with_context(|| format!("Failed to parse {}", input.display()))?;

    if let Some(parent) = output.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    render_file(&doc, output, output_format)
        .with_context(|| format!("Failed to render to {}", output.display()))?;

    let size = std::fs::metadata(output).map(|m| m.len()).unwrap_or(0);
    Ok(size)
}

fn determine_output_path(
    input: &Path,
    out_dir: &Option<PathBuf>,
    to_format: Option<Format>,
    from_format: Format,
) -> Result<PathBuf> {
    if let Some(dir) = out_dir {
        if dir.is_dir() {
            let stem = input
                .file_stem()
                .context("Invalid filename")?
                .to_string_lossy();
            let ext = to_format
                .map(|f| f.extension().to_string())
                .unwrap_or_else(|| {
                    // Keep original extension if no --to specified
                    input
                        .extension()
                        .map(|e| e.to_string_lossy().to_string())
                        .unwrap_or_default()
                });
            return Ok(dir.join(format!("{}.{}", stem, ext)));
        }
        return Ok(dir.clone());
    }

    // Default: same directory, same name, new extension
    let stem = input
        .file_stem()
        .context("Invalid filename")?
        .to_string_lossy();

    let ext = to_format
        .map(|f| f.extension().to_string())
        .unwrap_or_else(|| {
            // If no --to, try to use the format's own extension
            from_format.extension().to_string()
        });

    let mut output = input.to_path_buf();
    output.set_file_name(format!("{}.{}", stem, ext));

    // Avoid overwriting the input file
    if output == input {
        output.set_file_name(format!("{}.converted.{}", stem, ext));
    }

    Ok(output)
}

fn print_formats() {
    println!("{}", "Supported formats:".bold().underline());
    println!();
    for format in Format::all() {
        println!(
            "  {:<6} {:<35} .{}",
            format!("{:?}", format).cyan(),
            format.display_name().white(),
            format.extension().dimmed()
        );
    }
    println!();
    println!(
        "{}",
        "Use --from and --to flags to specify input/output formats.".dimmed()
    );
}

fn print_matrix() {
    let formats: Vec<Format> = Format::all().iter().cloned().collect();

    println!("{}", "Conversion matrix:".bold().underline());
    println!();
    println!("  Rows = input, Columns = output");
    println!("  ✓ = supported, · = not yet supported");
    println!();

    // Print header
    print!("{:>8}", "");
    for f in &formats {
        print!(" {:>4}", format!("{:?}", f).chars().take(4).collect::<String>());
    }
    println!();

    for row_format in &formats {
        print!("{:>8}", format!("{:?}", row_format).cyan());
        for col_format in &formats {
            if row_format == col_format {
                print!("    {}", "·".dimmed());
            } else if row_format.is_image() && col_format.is_image() {
                print!("    {}", "✓".green());
            } else {
                print!("    {}", "✓".green());
            }
        }
        println!();
    }

    println!();
    println!(
        "{}",
        "All formats support bidirectional conversion via the Document IR hub.".dimmed()
    );
}
