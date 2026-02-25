mod model;
mod output;
mod parser;
mod transform;

use anyhow::Result;
use clap::Parser;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "cclaml", about = "Parse ClaML XML files to structured JSON")]
struct Cli {
    /// Input ClaML XML file
    input: PathBuf,

    /// Output JSON file (defaults to stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Output compact JSON (no pretty-printing)
    #[arg(long)]
    compact: bool,
}

fn to_json(data: &impl serde::Serialize, compact: bool) -> serde_json::Result<String> {
    if compact {
        serde_json::to_string(data)
    } else {
        serde_json::to_string_pretty(data)
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    eprintln!("Parsing {}...", cli.input.display());
    let claml = parser::parse_claml(&cli.input)?;

    eprintln!(
        "Parsed {} classes, {} modifiers, {} modifier classes",
        claml.classes.len(),
        claml.modifiers.len(),
        claml.modifier_classes.len()
    );

    eprintln!("Transforming...");
    let output = transform::transform(&claml);

    eprintln!(
        "Output: {} chapters, {} blocks, {} categories, {} modifiers",
        output.chapters.len(),
        output.blocks.len(),
        output.categories.len(),
        output.modifiers.len()
    );

    let compact = cli.compact;

    match cli.output {
        Some(path) if path.to_string_lossy().ends_with('/') || path.is_dir() => {
            fs::create_dir_all(&path)?;

            fn write_json(dir: &std::path::Path, name: &str, data: &impl serde::Serialize, compact: bool) -> Result<()> {
                let file_path = dir.join(name);
                let json = to_json(data, compact)?;
                fs::write(&file_path, &json)?;
                eprintln!("Written {}", file_path.display());
                Ok(())
            }

            write_json(&path, "chapters.json", &output.chapters, compact)?;
            write_json(&path, "blocks.json", &output.blocks, compact)?;
            write_json(&path, "categories.json", &output.categories, compact)?;
            write_json(&path, "modifiers.json", &output.modifiers, compact)?;
        }
        Some(path) => {
            let json = to_json(&output, compact)?;
            fs::write(&path, &json)?;
            eprintln!("Written to {}", path.display());
        }
        None => {
            let json = to_json(&output, compact)?;
            io::stdout().write_all(json.as_bytes())?;
            io::stdout().write_all(b"\n")?;
        }
    }

    Ok(())
}
