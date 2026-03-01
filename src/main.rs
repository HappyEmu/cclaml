mod cli;
mod model;
mod output;
mod parser;
mod transform;

use anyhow::Result;
use clap::Parser;
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};

fn write_json(writer: impl Write, data: &impl serde::Serialize, compact: bool) -> serde_json::Result<()> {
    if compact {
        serde_json::to_writer(writer, data)
    } else {
        serde_json::to_writer_pretty(writer, data)
    }
}

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    eprintln!("Parsing {}...", cli.input.display());
    let claml = parser::parse_claml(&cli.input)?;

    eprintln!(
        "Parsed {} classes, {} modifiers, {} modifier classes",
        claml.classes.len(),
        claml.modifiers.len(),
        claml.modifier_classes.len()
    );

    eprintln!("Transforming...");
    let output = transform::transform(&claml, cli.flat);

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

            let prefix = cli.prefix.as_deref().unwrap_or("");
            anyhow::ensure!(
                !prefix.contains(std::path::is_separator),
                "prefix must not contain path separators"
            );

            let emit_paths = cli.emit_paths;

            fn write_json_file(dir: &std::path::Path, prefix: &str, name: &str, data: &impl serde::Serialize, compact: bool, emit_paths: bool) -> Result<()> {
                let file_path = dir.join(format!("{prefix}{name}"));
                let file = File::create(&file_path)?;
                let writer = BufWriter::new(file);
                if compact {
                    serde_json::to_writer(writer, data)?;
                } else {
                    serde_json::to_writer_pretty(writer, data)?;
                }
                eprintln!("Written {}", file_path.display());
                if emit_paths {
                    println!("{}", file_path.display());
                }
                Ok(())
            }

            write_json_file(&path, prefix, "chapters.json", &output.chapters, compact, emit_paths)?;
            write_json_file(&path, prefix, "blocks.json", &output.blocks, compact, emit_paths)?;
            write_json_file(&path, prefix, "categories.json", &output.categories, compact, emit_paths)?;
            write_json_file(&path, prefix, "modifiers.json", &output.modifiers, compact, emit_paths)?;
        }
        Some(path) => {
            let file = File::create(&path)?;
            let writer = BufWriter::new(file);
            write_json(writer, &output, compact)?;
            eprintln!("Written to {}", path.display());
        }
        None => {
            let stdout = io::stdout().lock();
            let mut writer = BufWriter::new(stdout);
            write_json(&mut writer, &output, compact)?;
            writer.write_all(b"\n")?;
            writer.flush()?;
        }
    }

    Ok(())
}
