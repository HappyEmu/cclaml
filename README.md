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

### Cross-compile for Linux (musl)

To produce a statically-linked Linux binary from macOS:

1. Install the musl cross-compiler toolchain:

```bash
brew install filosottile/musl-cross/musl-cross
```

2. Add the Rust target:

```bash
rustup target add x86_64-unknown-linux-musl
```

3. Configure the linker in `.cargo/config.toml`:

```toml
[target.x86_64-unknown-linux-musl]
linker = "x86_64-linux-musl-gcc"
```

4. Build:

```bash
cargo build --release --target x86_64-unknown-linux-musl
```

The statically-linked binary is at `target/x86_64-unknown-linux-musl/release/cclaml`. It runs on any x86_64 Linux system with no runtime dependencies.

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
| `--flat` | Resolve modifiers into individual category codes. Each modifier combination produces a new terminal category that inherits parent metadata. For categories with multiple modifiers, only fully-resolved codes are emitted (no partial application). The top-level modifier definitions remain in the output |

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

### Flat mode (resolved modifiers)

Use `--flat` to resolve modifiers into individual category codes. Each modifier combination produces a new terminal category:

```bash
cclaml icd10gm2025.xml --flat -o flat.json
```

Parent categories gain a `mod_codes` field listing the resolved codes, while each resolved category inherits parent metadata (inclusions, exclusions, breadcrumbs, etc.).

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
  "inclusions": ["Krankheiten, die allgemein als ansteckend oder übertragbar anerkannt sind"],
  "exclusions": ["Grippe und sonstige akute Infektionen der Atemwege {{J00-J22}}"]
}
```

Fields:
- `code` — Chapter identifier (roman numeral for ICD-10-GM, digit for OPS).
- `label` — Preferred label text.
- `sub_classes` — Block codes belonging to this chapter.
- `inclusions`, `exclusions`, `notes`, `coding_hints`, `definitions` — Rubric texts. Omitted when empty.

### Block

```json
{
  "code": "A00-A09",
  "label": "Infektiöse Darmkrankheiten",
  "range_start": "A00",
  "range_end": "A09",
  "super_class": "I",
  "sub_classes": ["A00", "A01", "A02", "A03", "A04", "A05", "A06", "A07", "A08", "A09"],
  "breadcrumb": [
    { "code": "I", "kind": "chapter" }
  ]
}
```

Fields:
- `code` — Block range code. ICD-10-GM uses `-` separator (`A00-A09`), OPS uses `...` (`1-20...1-33`).
- `range_start`, `range_end` — Parsed start/end codes of the range.
- `super_class` — Parent chapter or parent block code.
- `sub_classes` — Category or sub-block codes within this block.
- `breadcrumb` — Ancestor path from chapter to this block's parent, each entry with `code` and `kind` (`chapter` or `block`). Blocks can be nested (e.g., ICD: chapter II → block C00-C97 → block C00-C75 → block C00-C14).
- `inclusions`, `exclusions`, `notes` — Rubric texts. Omitted when empty.

### Category

```json
{
  "code": "A00.1",
  "label": "Cholera durch Vibrio cholerae O:1, Biovar eltor",
  "is_terminal": true,
  "super_class": "A00",
  "breadcrumb": [
    { "code": "I", "kind": "chapter" },
    { "code": "A00-A09", "kind": "block" },
    { "code": "A00", "kind": "category" }
  ],
  "inclusions": ["El-Tor-Cholera"]
}
```

Fields:
- `code` — Category code. ICD-10-GM: letter + digits (`A00.1`). OPS: digit + hyphen + digits (`1-202.01`).
- `label` — Preferred label. References to other codes appear as `{{A00.0†}}` (dagger), `{{G63.0*}}` (aster), `{{U80!}}` (optional).
- `label_long` — Extended label (OPS `preferredLong` rubric). Omitted when absent.
- `is_terminal` — `true` if the category has no sub-categories.
- `super_class` — Parent category or block code.
- `sub_classes` — Child category codes. Omitted when empty.
- `breadcrumb` — Ancestor path from chapter to this category's parent, each entry with `code` and `kind` (`chapter`, `block`, or `category`). Does not include the category itself.
- `inclusions`, `exclusions`, `coding_hints`, `definitions`, `notes` — Rubric texts. Omitted when empty.
- `modifiers` — Modifier references. Each has a `code` pointing into the top-level modifiers map. `valid_values` lists allowed modifier codes when not all values apply; omitted when all values are valid.
- `mod_codes` — (flat mode only) List of resolved modifier codes derived from this category. Only present when `--flat` is used and the category has modifiers.

### Modifier

Modifiers are keyed by their code in a top-level map:

```json
{
  "S04E10_4": {
    "description": "Die folgenden vierten Stellen sind bei den Kategorien E10-E14 zu benutzen:",
    "values": [
      {
        "code": ".0",
        "label": "Mit Koma",
        "inclusions": ["Diabetisches Koma: hyperosmolar", "Diabetisches Koma: mit oder ohne Ketoazidose"],
        "exclusions": ["Hypoglykämisches Koma (.6)"]
      },
      {
        "code": ".2",
        "label": "Mit Nierenkomplikationen",
        "usage": "dagger",
        "inclusions": ["Diabetische Nephropathie {{N08.3*}}", "Kimmelstiel-Wilson-Syndrom {{N08.3*}}"]
      }
    ]
  }
}
```

Fields:
- `description` — Label for the modifier group (from the `text` rubric).
- `values` — Available modifier values, each with:
  - `code` — Modifier value code (e.g., `.0`, `.2`).
  - `label` — Preferred label text.
  - `usage` — Usage kind if present (`dagger`, `aster`, `optional`). Omitted when absent.
  - `inclusions`, `exclusions`, `coding_hints`, `definitions`, `notes` — Per-value rubric texts. Omitted when empty.
  - `excludes` — Modifier value combinations that are invalid when this value is used. Each entry names the other `modifier` code and the excluded `code` within it. Omitted when empty.

## Piping to jq

When writing to stdout (no `-o`), you can pipe directly to `jq` to extract or filter parts of the output.

### Extract only chapters

```bash
cclaml icd10gm2025.xml | jq '.chapters'
```

### Extract only blocks

```bash
cclaml icd10gm2025.xml | jq '.blocks'
```

### Extract only terminal categories

```bash
cclaml icd10gm2025.xml | jq '[.categories[] | select(.is_terminal)]'
```

### Look up a single category by code

```bash
cclaml icd10gm2025.xml | jq '.categories[] | select(.code == "A00.1")'
```

### List all chapter codes and labels

```bash
cclaml icd10gm2025.xml | jq '.chapters[] | {code, label}'
```

### Extract categories belonging to a specific block

```bash
cclaml icd10gm2025.xml | jq '[.categories[] | select(.breadcrumb[] | .code == "A00-A09" and .kind == "block")]'
```

### Get modifier details for a specific modifier code

```bash
cclaml icd10gm2025.xml | jq '.modifiers["S02C88_5"]'
```

### Combine with `--compact` for faster processing

For large files, `--compact` skips pretty-printing and produces smaller output, which `jq` can then re-format as needed:

```bash
cclaml icd10gm2025.xml --compact | jq '.chapters'
```

## Supported classifications

- **ICD-10-GM** — International Classification of Diseases, German Modification
- **OPS** — Operationen- und Prozedurenschlüssel (German procedure classification)

Both use the ClaML 2.0.0 XML schema.
