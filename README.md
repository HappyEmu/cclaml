# cclaml

ClaML XML to JSON parser for ICD-10-GM and OPS classification data.

## Install

From the repository:

```bash
cargo install --git https://github.com/HappyEmu/cclaml.git
```

Or build locally:

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

## Sample JSON output

### Chapter

```json
{
  "code": "I",
  "label": "Bestimmte infektiöse und parasitäre Krankheiten",
  "sub_classes": ["A00-A09", "A15-A19", "A20-A28"],
  "exclusions": ["Grippe und sonstige akute Infektionen der Atemwege (J00-J22)"]
}
```

Fields:
- `code` — Chapter identifier (roman numeral for ICD-10-GM, digit for OPS).
- `label` — Preferred label text.
- `sub_classes` — Block codes belonging to this chapter.
- `inclusions`, `exclusions`, `notes` — Rubric texts. Omitted when empty.

### Block

```json
{
  "code": "A00-A09",
  "label": "Infektiöse Darmkrankheiten",
  "range_start": "A00",
  "range_end": "A09",
  "super_class": "I",
  "sub_classes": ["A00", "A01", "A02", "A03", "A04", "A05", "A06", "A07", "A08", "A09"],
  "exclusions": ["Lebensmittelvergiftung durch Bakterien (A05.-)"]
}
```

Fields:
- `code` — Block range code. ICD-10-GM uses `-` separator (`A00-A09`), OPS uses `...` (`1-20...1-33`).
- `range_start`, `range_end` — Parsed start/end codes of the range.
- `super_class` — Parent chapter code.
- `sub_classes` — Category or sub-block codes within this block.
- `inclusions`, `exclusions`, `notes` — Rubric texts. Omitted when empty.

### Category

```json
{
  "code": "A00.1",
  "label": "Cholera durch Vibrio cholerae O139",
  "is_terminal": true,
  "super_class": "A00",
  "breadcrumb": [
    { "code": "I", "kind": "chapter" },
    { "code": "A00-A09", "kind": "block" },
    { "code": "A00", "kind": "category" },
    { "code": "A00.1", "kind": "category" }
  ],
  "inclusions": ["Cholera: El Tor"],
  "coding_hints": ["Soll der Erreger angegeben werden, ist eine zusätzliche Schlüsselnummer (U80-U85) zu benutzen."],
  "modifiers": [
    {
      "code": "S_A00",
      "valid_values": [".0", ".1"]
    }
  ]
}
```

Fields:
- `code` — Category code. ICD-10-GM: letter + digits (`A00.1`). OPS: digit + hyphen + digits (`1-202.01`).
- `label` — Preferred label. References to other codes appear as `{{A00.0†}}` (dagger), `{{G63.0*}}` (aster), `{{U80!}}` (optional).
- `label_long` — Extended label (OPS `preferredLong` rubric). Omitted when absent.
- `is_terminal` — `true` if the category has no sub-categories.
- `super_class` — Parent category or block code.
- `sub_classes` — Child category codes. Omitted when empty.
- `breadcrumb` — Full path from chapter to this category, each entry with `code` and `kind` (`chapter`, `block`, or `category`).
- `inclusions`, `exclusions`, `coding_hints`, `definitions`, `notes` — Rubric texts. Omitted when empty.
- `modifiers` — Modifier references. Each has a `code` pointing into the top-level modifiers map. `valid_values` lists allowed modifier codes when not all values apply; omitted when all values are valid.

### Modifier

Modifiers are keyed by their code in a top-level map:

```json
{
  "S_A00": {
    "description": "Cholera",
    "values": [
      {
        "code": ".0",
        "label": "Cholera durch Vibrio cholerae O:1, Biovar cholerae"
      },
      {
        "code": ".1",
        "label": "Cholera durch Vibrio cholerae O139",
        "usage": "dagger",
        "inclusions": ["Cholera: El Tor"],
        "exclusions": ["Cholera, nicht näher bezeichnet (A00.9)"],
        "excludes": [
          {
            "modifier": "S_B95_B98",
            "code": ".0"
          }
        ]
      }
    ]
  }
}
```

Fields:
- `description` — Label for the modifier group (from the `text` rubric).
- `values` — Available modifier values, each with:
  - `code` — Modifier value code (e.g., `.0`, `.1`).
  - `label` — Preferred label text.
  - `usage` — Usage kind if present (`dagger`, `aster`, `optional`). Omitted when absent.
  - `inclusions`, `exclusions`, `coding_hints`, `definitions`, `notes` — Per-value rubric texts. Omitted when empty.
  - `excludes` — Modifier value combinations that are invalid when this value is used. Each entry names the other `modifier` code and the excluded `code` within it. Omitted when empty.

## Filtering with jq

When writing to stdout (no `-o`), you can pipe through [jq](https://jqlang.github.io/jq/) to extract subsets of the output.

### Extract a single top-level key

```bash
# Only chapters
cclaml icd10gm2025.xml | jq '.chapters'

# Only modifiers
cclaml icd10gm2025.xml | jq '.modifiers'
```

### Write a subset to a file

```bash
cclaml icd10gm2025.xml | jq '.categories' > categories.json
```

### Find a specific code

```bash
# Look up a single category by code
cclaml icd10gm2025.xml | jq '.categories[] | select(.code == "A00.1")'

# Find all blocks in chapter I
cclaml icd10gm2025.xml | jq '.blocks[] | select(.super_class == "I")'
```

### List all terminal categories

```bash
cclaml icd10gm2025.xml | jq '[.categories[] | select(.is_terminal)] | length'
```

### Extract codes and labels only

```bash
cclaml icd10gm2025.xml | jq '.categories[] | {code, label}'
```

### Look up a modifier by key

```bash
cclaml icd10gm2025.xml | jq '.modifiers["S_A00"]'
```

### Combine with --compact for faster piping

`--compact` skips pretty-printing on the cclaml side, which is faster for large files when jq will reformat anyway:

```bash
cclaml icd10gm2025.xml --compact | jq '.chapters[]'
```

## Supported classifications

- **ICD-10-GM** — International Classification of Diseases, German Modification
- **OPS** — Operationen- und Prozedurenschlüssel (German procedure classification)

Both use the ClaML 2.0.0 XML schema.
