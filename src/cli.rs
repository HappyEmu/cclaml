use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "cclaml",
    about = "Parse ClaML XML classification files (ICD-10-GM, OPS) to structured JSON",
    long_about = "Parse ClaML XML classification files (ICD-10-GM, OPS) to structured JSON.\n\n\
        Output modes:\n  \
        - Single file:  cclaml input.xml -o output.json\n  \
        - Directory:    cclaml input.xml -o out/        (splits into chapters, blocks, categories, modifiers)\n  \
        - Stdout:       cclaml input.xml                (single JSON to stdout)\n\n\
        Examples:\n  \
        cclaml icd10gm.xml -o icd10gm.json\n  \
        cclaml icd10gm.xml -o out/ --prefix icd10gm2025_ --compact\n  \
        cclaml ops.xml -o out/ --emit-paths | xargs gzip",
    after_help = "Supported classifications: ICD-10-GM and OPS (ClaML 2.0.0 schema)."
)]
pub struct Cli {
    /// Input ClaML XML file (ICD-10-GM or OPS)
    pub input: PathBuf,

    /// Output path. File path writes single JSON; directory path (trailing '/') splits into separate files. Omit for stdout
    #[arg(short, long, value_name = "PATH")]
    pub output: Option<PathBuf>,

    /// Output compact JSON instead of pretty-printed
    #[arg(long)]
    pub compact: bool,

    /// Prefix for output filenames in directory mode (e.g. "icd10gm2025_" produces "icd10gm2025_chapters.json")
    #[arg(long, value_name = "PREFIX")]
    pub prefix: Option<String>,

    /// Print written file paths to stdout, one per line. Useful for piping: --emit-paths | xargs gzip
    #[arg(long)]
    pub emit_paths: bool,
}
