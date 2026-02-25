# cclaml

ClaML XML to JSON parser for ICD-10-GM classification data.

## Build & Run

```bash
cargo build --release
cargo run --release -- <input.xml> -o output.json
```

Test file: `/Users/gerberur/Desktop/medcode-claude/icd10gm2025syst-claml/Klassifikationsdateien/icd10gm2025syst_claml_20240913.xml`

## Architecture

- `src/model.rs` — XML deserialization structs (serde + quick-xml). Mixed content (Label, Para, Fragment) uses `$value` with enum-based approach (`LabelContent`, `SimpleMixed`).
- `src/output.rs` — JSON output structs. Empty vecs are skipped via `skip_serializing_if`.
- `src/parser.rs` — Reads XML file, strips DOCTYPE, deserializes into model structs.
- `src/transform.rs` — Builds lookup maps, resolves breadcrumbs (with kind), resolves modifiers (with `excludeOnPrecedingModifier` exclusions and per-value rubrics), produces output structs.
- `src/main.rs` — CLI entry point (clap). Reads XML, transforms, writes JSON to file or stdout.

## Key Design Decisions

- **Mixed XML content**: quick-xml serde can't handle interleaved text/elements with `$text`. Uses `$value` + enum (`LabelContent`, `SimpleMixed`) to capture ordered mixed content.
- **References**: Formatted as `{{code†}}` / `{{code*}}` / `{{code!}}` in label text (dagger/aster/optional usage marks).
- **Modifier exclusions**: `excludeOnPrecedingModifier` meta on ModifierClass encodes which modifier value combinations are invalid. Exposed as `excludes` array on each modifier value.
- **Modifier rubrics**: Each ModifierClass can have its own inclusions, exclusions, coding hints, definitions, notes — independent from the parent category.
- **Breadcrumbs**: Include `kind` (chapter/block/category) alongside `code` for unambiguous resolution.

## ICD-10-GM ClaML Concepts

- **Chapters** (I–XXII): Top-level grouping by roman numeral.
- **Blocks** (e.g., A00-A09): Range-based grouping within chapters.
- **Categories** (e.g., A00, A00.0): Individual codes. Terminal if no sub_classes.
- **Modifiers**: Reusable digit-extension templates (4th/5th position). Applied via `ModifiedBy` on categories. `all="false"` + `ValidModifierClass` restricts which values apply.
- **UsageKinds**: dagger (†) = etiology, aster (*) = manifestation, optional (!) = supplementary.
