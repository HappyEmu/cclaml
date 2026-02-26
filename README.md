# cclaml

ClaML XML to JSON parser for ICD-10-GM and OPS classification data.

## Install

```bash
cargo build --release
```

The binary is at `target/release/cclaml`.

## Usage

```
cclaml [OPTIONS] <INPUT>
```

| Argument / Option | Description |
|---|---|
| `<INPUT>` | Input ClaML XML file (ICD-10-GM or OPS) |
| `-o`, `--output <PATH>` | Output path. File path writes a single JSON file; directory path (trailing `/`) splits into separate files. Omit for stdout |
| `--compact` | Output compact JSON instead of pretty-printed |
| `--prefix <PREFIX>` | Prefix for output filenames in directory mode (e.g. `icd10gm2025_`) |
| `--emit-paths` | Print written file paths to stdout, one per line. Useful for piping to `xargs gzip` |

## Examples

### Single JSON file

```bash
cclaml icd10gm2025.xml -o icd10gm2025.json
```

### Split into separate files (directory mode)

Use a trailing `/` to write chapters, blocks, categories, and modifiers as separate files:

```bash
cclaml icd10gm2025.xml -o out/
```

Produces:
```
out/chapters.json
out/blocks.json
out/categories.json
out/modifiers.json
```

### File prefix

Add a prefix to output filenames in directory mode:

```bash
cclaml icd10gm2025.xml -o out/ --prefix icd10gm2025_
```

Produces:
```
out/icd10gm2025_chapters.json
out/icd10gm2025_blocks.json
out/icd10gm2025_categories.json
out/icd10gm2025_modifiers.json
```

### Compact JSON

Skip pretty-printing for smaller files:

```bash
cclaml icd10gm2025.xml -o out/ --compact
```

### Pipe to gzip

Use `--emit-paths` to print written file paths to stdout, then pipe to `xargs gzip`:

```bash
cclaml icd10gm2025.xml -o out/ --prefix icd10gm2025_ --emit-paths | xargs gzip
```

### Stdout

Omit `-o` to write JSON to stdout:

```bash
cclaml icd10gm2025.xml > icd10gm2025.json
```

## Supported classifications

- **ICD-10-GM** — International Classification of Diseases, German Modification
- **OPS** — Operationen- und Prozedurenschlüssel (German procedure classification)

Both use the ClaML 2.0.0 XML schema.
