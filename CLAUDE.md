# cclaml

ClaML XML to JSON parser for ICD-10-GM and OPS classification data.

## Build & Run

```bash
cargo build --release
cargo run --release -- <input.xml> -o output.json       # single file
cargo run --release -- <input.xml> -o out/               # split into directory
cargo run --release -- <input.xml> -o out/ --prefix icd10gm2025_ --compact
cargo run --release -- <input.xml> -o out/ --emit-paths | xargs gzip
```

### CLI Options

- `-o, --output <PATH>` — File path for single JSON, directory path (trailing `/`) to split into chapters/blocks/categories/modifiers. Omit for stdout.
- `--compact` — Compact JSON (no pretty-printing).
- `--prefix <PREFIX>` — Filename prefix in directory mode (e.g. `icd10gm2025_`). Must not contain path separators.
- `--emit-paths` — Print written file paths to stdout (one per line), for piping to `xargs gzip` etc.


## Filtering with jq

Stdout mode (no `-o`) pipes through jq to extract subsets:

```bash
cclaml <input.xml> | jq '.chapters'                                    # single key
cclaml <input.xml> | jq '.categories[] | select(.code == "A00.1")'     # find by code
cclaml <input.xml> | jq '.blocks[] | select(.super_class == "I")'      # filter by field
cclaml <input.xml> | jq '[.categories[] | select(.is_terminal)] | length'  # count
cclaml <input.xml> | jq '.categories[] | {code, label}'                # project fields
cclaml <input.xml> | jq '.modifiers["S_A00"]'                          # modifier lookup
cclaml <input.xml> --compact | jq '.chapters[]'                        # faster piping
```

## Architecture

- `src/model.rs` — XML deserialization structs (serde + quick-xml). Mixed content (Label, Para, Fragment) uses `$value` with enum-based approach (`LabelContent`, `SimpleMixed`).
- `src/output.rs` — JSON output structs. Empty vecs are skipped via `skip_serializing_if`.
- `src/parser.rs` — Reads XML file, strips DOCTYPE, deserializes into model structs.
- `src/transform.rs` — Builds lookup maps, resolves breadcrumbs (with kind), resolves modifiers (with `excludeOnPrecedingModifier` exclusions and per-value rubrics), produces output structs.
- `src/cli.rs` — CLI argument definitions (clap derive).
- `src/main.rs` — CLI entry point. Reads XML, transforms, writes JSON to file(s) or stdout.

## Key Design Decisions

- **Mixed XML content**: quick-xml serde can't handle interleaved text/elements with `$text`. Uses `$value` + enum (`LabelContent`, `SimpleMixed`) to capture ordered mixed content.
- **References**: Formatted as `{{code†}}` / `{{code*}}` / `{{code!}}` in label text (dagger/aster/optional usage marks). Unknown usage kinds (e.g., OPS "seite") produce no mark suffix.
- **Block range parsing**: Tries `...` separator first (OPS, e.g., `1-20...1-33`), then falls back to `-` (ICD-10-GM, e.g., `A00-A09`). Needed because OPS codes contain `-`.
- **Modifier exclusions**: `excludeOnPrecedingModifier` meta on ModifierClass encodes which modifier value combinations are invalid. Exposed as `excludes` array on each modifier value.
- **Modifier rubrics**: Each ModifierClass can have its own inclusions, exclusions, coding hints, definitions, notes — independent from the parent category.
- **Breadcrumbs**: Include `kind` (chapter/block/category) alongside `code` for unambiguous resolution.

## ClaML Concepts

Both ICD-10-GM and OPS use the ClaML 2.0.0 schema. Same XML structure, different data conventions.

### Shared Structure

- **Chapters**: Top-level grouping. ICD uses roman numerals (I–XXII), OPS uses digits (1, 3, 5, 6, 8, 9).
- **Blocks**: Range-based grouping within chapters. ICD: `A00-A09`, OPS: `1-20...1-33`.
- **Categories**: Individual codes. Terminal if no sub_classes.
- **Modifiers**: Reusable digit-extension templates. Applied via `ModifiedBy` on categories. `all="false"` + `ValidModifierClass` restricts which values apply.

### ICD-10-GM Specifics

- **Code format**: Letter + digits (e.g., `A00`, `A00.0`).
- **UsageKinds**: dagger (†) = etiology, aster (*) = manifestation, optional (!) = supplementary.
- **RubricKinds**: preferred, inclusion, exclusion, note, coding_hint, definition, text.

### OPS Specifics

- **Code format**: Digit + hyphen + digits (e.g., `1-20`, `1-202.01`). Can have letter suffixes (`1-20a`, `6-00p`). Special codes: `x` (sonstige/other), `y` (n.n.bez./not otherwise specified).
- **Block separator**: `...` instead of `-` (because codes themselves contain `-`).
- **UsageKinds**: seite (S) = side/location marking.
- **RubricKinds**: preferred, preferredLong, inclusion, exclusion, note.
