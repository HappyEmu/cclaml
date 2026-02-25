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
        "Output: {} chapters, {} blocks, {} categories",
        output.chapters.len(),
        output.blocks.len(),
        output.categories.len()
    );

    let json = serde_json::to_string_pretty(&output)?;

    match cli.output {
        Some(path) => {
            fs::write(&path, &json)?;
            eprintln!("Written to {}", path.display());
        }
        None => {
            io::stdout().write_all(json.as_bytes())?;
            io::stdout().write_all(b"\n")?;
        }
    }

    Ok(())
}
