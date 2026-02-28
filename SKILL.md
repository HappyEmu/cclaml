---
name: cclaml
description: Guide for using the cclaml binary to convert ClaML XML files (ICD-10-GM, OPS) to JSON. Use when the user asks how to parse, convert, or extract data from ICD-10-GM or OPS ClaML XML files.
disable-model-invocation: true
argument-hint: [question]
---

# cclaml — ClaML XML to JSON parser

Converts ICD-10-GM and OPS ClaML XML files into structured JSON.

## Usage

```bash
cclaml <INPUT> [OPTIONS]
```

## Options

| Option | Description |
|---|---|
| `-o, --output <PATH>` | File path for single JSON, or directory path (trailing `/`) to split into chapters/blocks/categories/modifiers |
| `--compact` | Compact JSON (no pretty-printing) |
| `--prefix <PREFIX>` | Filename prefix in directory mode |
| `--emit-paths` | Print written file paths to stdout (for piping to `xargs gzip`) |
| `--flat` | Resolve modifiers into individual category codes |

## Examples

```bash
# Single JSON file
cclaml input.xml -o output.json

# Split into separate files
cclaml input.xml -o out/

# Split with prefix and compress
cclaml input.xml -o out/ --prefix icd10gm2025_ --emit-paths | xargs gzip

# Stdout + jq
cclaml input.xml | jq '[.categories[] | select(.is_terminal)]'
cclaml input.xml | jq '.categories[] | select(.code == "A00.1")'
cclaml input.xml | jq '.modifiers["S04E10_4"]'

# Flat mode (expand modifiers into concrete codes)
cclaml input.xml --flat -o flat.json
```

## Output structure

Four top-level keys: `chapters`, `blocks`, `categories`, `modifiers`.

- **Categories** have `code`, `label`, `is_terminal`, `breadcrumb`, and optional rubrics (`inclusions`, `exclusions`, `coding_hints`, `notes`).
- **Modifiers** are a map of modifier code to values array, each with `code`, `label`, and optional rubrics.
- `--flat` expands modifiers into new terminal categories and adds `mod_codes` to parents.
