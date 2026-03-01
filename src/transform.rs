use std::collections::{HashMap, HashSet};

use crate::model::{self, ClaML};
use crate::output::{
    Block, BreadcrumbEntry, Category, Chapter, ModifierExclusion, ModifierGroup, ModifierRef,
    ModifierValue, Output,
};

pub fn transform(claml: &ClaML, flat: bool) -> Output {
    let modifier_map = build_modifier_map(claml);
    let modifier_class_map = build_modifier_class_map(claml);
    let class_map = build_class_map(claml);

    // Build top-level modifier definitions once
    let modifier_definitions = build_modifier_definitions(&modifier_map, &modifier_class_map);

    // Collect excluded modifier codes per category
    let exclude_modifier_set: HashMap<&str, HashSet<&str>> = claml
        .classes
        .iter()
        .filter(|c| !c.exclude_modifiers.is_empty())
        .map(|c| {
            let set: HashSet<&str> = c.exclude_modifiers.iter().map(|e| e.code.as_str()).collect();
            (c.code.as_str(), set)
        })
        .collect();

    let mut chapters = Vec::new();
    let mut blocks = Vec::new();
    let mut categories = Vec::new();

    for class in &claml.classes {
        match class.kind.as_str() {
            "chapter" => {
                chapters.push(build_chapter(class));
            }
            "block" => {
                let breadcrumb = build_breadcrumb(&class.code, &class_map);
                blocks.push(build_block(class, breadcrumb));
            }
            "category" => {
                let breadcrumb = build_breadcrumb(&class.code, &class_map);
                if flat && !class.modified_by.is_empty() {
                    // Emit expanded categories for each resolved modifier combination
                    let expanded = expand_modifiers(
                        class,
                        &breadcrumb,
                        &modifier_definitions,
                        &exclude_modifier_set,
                    );
                    // Emit parent category without modifier refs, with mod_codes
                    let mut parent = build_category(class, breadcrumb.clone(), Vec::new());
                    parent.mod_codes = expanded.iter().map(|c| c.code.clone()).collect();
                    categories.push(parent);
                    categories.extend(expanded);
                } else {
                    let modifiers = resolve_modifier_refs(&class.modified_by);
                    categories.push(build_category(class, breadcrumb, modifiers));
                }
            }
            _ => {}
        }
    }

    Output {
        chapters,
        blocks,
        categories,
        modifiers: modifier_definitions,
    }
}

fn build_modifier_map<'a>(claml: &'a ClaML) -> HashMap<&'a str, &'a model::Modifier> {
    claml.modifiers.iter().map(|m| (m.code.as_str(), m)).collect()
}

fn build_modifier_class_map<'a>(claml: &'a ClaML) -> HashMap<&'a str, Vec<&'a model::ModifierClass>> {
    let mut map: HashMap<&str, Vec<&model::ModifierClass>> = HashMap::new();
    for mc in &claml.modifier_classes {
        map.entry(mc.modifier.as_str()).or_default().push(mc);
    }
    map
}

fn build_class_map<'a>(claml: &'a ClaML) -> HashMap<&'a str, &'a model::Class> {
    claml.classes.iter().map(|c| (c.code.as_str(), c)).collect()
}

fn build_modifier_definitions(
    modifier_map: &HashMap<&str, &model::Modifier>,
    modifier_class_map: &HashMap<&str, Vec<&model::ModifierClass>>,
) -> HashMap<String, ModifierGroup> {
    let mut definitions = HashMap::new();

    for (&code, modifier) in modifier_map {
        let description = get_text_label(&modifier.rubrics);

        let values = if let Some(mcs) = modifier_class_map.get(code) {
            mcs.iter()
                .map(|mc| {
                    let excludes = mc
                        .metas
                        .iter()
                        .filter(|m| m.name == "excludeOnPrecedingModifier")
                        .filter_map(|m| {
                            let (modifier, code) = m.value.split_once(' ')?;
                            Some(ModifierExclusion {
                                modifier: modifier.to_string(),
                                code: code.to_string(),
                            })
                        })
                        .collect();

                    let rubrics = extract_all_rubrics(&mc.rubrics);
                    ModifierValue {
                        code: mc.code.clone(),
                        label: rubrics.preferred,
                        usage: mc.usage.clone(),
                        inclusions: rubrics.inclusions,
                        exclusions: rubrics.exclusions,
                        coding_hints: rubrics.coding_hints,
                        definitions: rubrics.definitions,
                        notes: rubrics.notes,
                        excludes,
                    }
                })
                .collect()
        } else {
            Vec::new()
        };

        definitions.insert(
            code.to_string(),
            ModifierGroup {
                description,
                values,
            },
        );
    }

    definitions
}

/// Build modifier references for a category. Each reference has the modifier code
/// and optionally a `valid_values` list when `all="false"`.
fn resolve_modifier_refs(modified_by: &[model::ModifiedBy]) -> Vec<ModifierRef> {
    modified_by
        .iter()
        .map(|mb| {
            let valid_values = if !mb.all {
                Some(
                    mb.valid_modifier_classes
                        .iter()
                        .map(|v| v.code.clone())
                        .collect(),
                )
            } else {
                None
            };
            ModifierRef {
                code: mb.code.clone(),
                valid_values,
            }
        })
        .collect()
}

/// Extracted rubric data from a single pass over the rubric list.
struct ExtractedRubrics {
    preferred: String,
    preferred_long: String,
    inclusions: Vec<String>,
    exclusions: Vec<String>,
    coding_hints: Vec<String>,
    definitions: Vec<String>,
    notes: Vec<String>,
    introductions: Vec<String>,
    texts: Vec<String>,
}

/// Extract all rubric kinds in a single pass over the rubric list.
fn extract_all_rubrics(rubrics: &[model::Rubric]) -> ExtractedRubrics {
    let mut result = ExtractedRubrics {
        preferred: String::new(),
        preferred_long: String::new(),
        inclusions: Vec::new(),
        exclusions: Vec::new(),
        coding_hints: Vec::new(),
        definitions: Vec::new(),
        notes: Vec::new(),
        introductions: Vec::new(),
        texts: Vec::new(),
    };

    for rubric in rubrics {
        match rubric.kind.as_str() {
            "preferred" => {
                if result.preferred.is_empty() {
                    if let Some(label) = rubric.labels.first() {
                        result.preferred = label.flat_text();
                    }
                }
            }
            "preferredLong" => {
                if result.preferred_long.is_empty() {
                    if let Some(label) = rubric.labels.first() {
                        result.preferred_long = label.flat_text();
                    }
                }
            }
            "inclusion" => {
                for l in &rubric.labels {
                    let text = l.flat_text();
                    if !text.is_empty() {
                        result.inclusions.push(text);
                    }
                }
            }
            "exclusion" => {
                for l in &rubric.labels {
                    let text = l.flat_text();
                    if !text.is_empty() {
                        result.exclusions.push(text);
                    }
                }
            }
            "coding-hint" => {
                for l in &rubric.labels {
                    let text = l.flat_text();
                    if !text.is_empty() {
                        result.coding_hints.push(text);
                    }
                }
            }
            "definition" => {
                for l in &rubric.labels {
                    let text = l.flat_text();
                    if !text.is_empty() {
                        result.definitions.push(text);
                    }
                }
            }
            "note" => {
                for l in &rubric.labels {
                    let text = l.flat_text();
                    if !text.is_empty() {
                        result.notes.push(text);
                    }
                }
            }
            "introduction" => {
                for l in &rubric.labels {
                    let text = l.flat_text();
                    if !text.is_empty() {
                        result.introductions.push(text);
                    }
                }
            }
            "text" => {
                for l in &rubric.labels {
                    let text = l.flat_text();
                    if !text.is_empty() {
                        result.texts.push(text);
                    }
                }
            }
            _ => {}
        }
    }

    result
}

fn get_text_label(rubrics: &[model::Rubric]) -> String {
    for rubric in rubrics {
        if rubric.kind == "text" {
            if let Some(label) = rubric.labels.first() {
                return label.flat_text();
            }
        }
    }
    String::new()
}

fn build_chapter(class: &model::Class) -> Chapter {
    let rubrics = extract_all_rubrics(&class.rubrics);
    Chapter {
        code: class.code.clone(),
        label: rubrics.preferred,
        sub_classes: class.sub_classes.iter().map(|s| s.code.clone()).collect(),
        inclusions: rubrics.inclusions,
        exclusions: rubrics.exclusions,
        coding_hints: rubrics.coding_hints,
        notes: rubrics.notes,
        introductions: rubrics.introductions,
        texts: rubrics.texts,
    }
}

fn build_block(class: &model::Class, breadcrumb: Vec<BreadcrumbEntry>) -> Block {
    let (range_start, range_end) = parse_block_range(&class.code);
    let rubrics = extract_all_rubrics(&class.rubrics);
    Block {
        code: class.code.clone(),
        label: rubrics.preferred,
        range_start,
        range_end,
        super_class: class.super_classes.first().map(|s| s.code.clone()),
        sub_classes: class.sub_classes.iter().map(|s| s.code.clone()).collect(),
        breadcrumb,
        inclusions: rubrics.inclusions,
        exclusions: rubrics.exclusions,
        coding_hints: rubrics.coding_hints,
        notes: rubrics.notes,
        texts: rubrics.texts,
    }
}

fn parse_block_range(code: &str) -> (String, String) {
    // OPS uses "..." as separator (e.g., "1-20...1-33") since codes contain "-"
    // ICD-10-GM uses "-" (e.g., "A00-A09")
    if let Some((start, end)) = code.split_once("...") {
        (start.to_string(), end.to_string())
    } else if let Some((start, end)) = code.split_once('-') {
        (start.to_string(), end.to_string())
    } else {
        (code.to_string(), code.to_string())
    }
}

fn build_category(
    class: &model::Class,
    breadcrumb: Vec<BreadcrumbEntry>,
    modifiers: Vec<ModifierRef>,
) -> Category {
    let rubrics = extract_all_rubrics(&class.rubrics);
    Category {
        code: class.code.clone(),
        label: rubrics.preferred,
        label_long: if rubrics.preferred_long.is_empty() {
            None
        } else {
            Some(rubrics.preferred_long)
        },
        is_terminal: class.sub_classes.is_empty(),
        super_class: class.super_classes.first().map(|s| s.code.clone()),
        sub_classes: class.sub_classes.iter().map(|s| s.code.clone()).collect(),
        breadcrumb,
        inclusions: rubrics.inclusions,
        exclusions: rubrics.exclusions,
        coding_hints: rubrics.coding_hints,
        definitions: rubrics.definitions,
        notes: rubrics.notes,
        texts: rubrics.texts,
        mod_codes: Vec::new(),
        modifiers,
    }
}

/// Expand a category's modifiers into individual flat categories.
/// For categories with multiple modifiers, only fully-resolved combinations are emitted.
fn expand_modifiers(
    class: &model::Class,
    parent_breadcrumb: &[BreadcrumbEntry],
    modifier_definitions: &HashMap<String, ModifierGroup>,
    exclude_modifier_set: &HashMap<&str, HashSet<&str>>,
) -> Vec<Category> {
    let excluded = exclude_modifier_set.get(class.code.as_str());

    // Collect valid modifier values and codes in a single pass over modified_by,
    // respecting ExcludeModifier
    let mut modifier_value_sets: Vec<Vec<&ModifierValue>> = Vec::new();
    let mut modifier_codes: Vec<&str> = Vec::new();

    for mb in &class.modified_by {
        // Skip modifiers excluded by ExcludeModifier on this category
        if excluded.map_or(false, |set| set.contains(mb.code.as_str())) {
            continue;
        }

        let group = match modifier_definitions.get(&mb.code) {
            Some(g) => g,
            None => {
                modifier_value_sets.push(Vec::new());
                modifier_codes.push(&mb.code);
                continue;
            }
        };

        let values: Vec<&ModifierValue> = group
            .values
            .iter()
            .filter(|v| {
                if mb.all {
                    true
                } else {
                    mb.valid_modifier_classes
                        .iter()
                        .any(|vc| vc.code == v.code)
                }
            })
            .collect();

        modifier_value_sets.push(values);
        modifier_codes.push(&mb.code);
    }

    // If any modifier has no valid values, skip expansion entirely
    if modifier_value_sets.is_empty() || modifier_value_sets.iter().any(|s| s.is_empty()) {
        return Vec::new();
    }

    // Build cartesian product — for 2+ modifiers only fully-resolved combos
    let combinations = cartesian_product(&modifier_value_sets);

    // Collect parent metadata once
    let parent_rubrics = extract_all_rubrics(&class.rubrics);

    // Build parent breadcrumb entry once (shared across all combos)
    let parent_entry = BreadcrumbEntry {
        code: class.code.clone(),
        kind: "category".to_string(),
    };

    let mut results = Vec::new();

    for combo in &combinations {
        // Check excludeOnPrecedingModifier conflicts
        if has_exclusion_conflict(combo, &modifier_codes) {
            continue;
        }

        // Build expanded code: parent code + all modifier value codes
        let mut code = class.code.clone();
        for val in combo {
            code.push_str(&val.code);
        }

        // Label: parent label + ": " + each modifier value's label joined by ": "
        let modifier_labels: Vec<&str> = combo.iter().map(|v| v.label.as_str()).collect();
        let label = format!("{}: {}", parent_rubrics.preferred, modifier_labels.join(": "));

        // Breadcrumb: parent's ancestors + parent itself
        let mut breadcrumb = parent_breadcrumb.to_vec();
        breadcrumb.push(parent_entry.clone());

        // Merge metadata: parent + all modifier values in order
        let mut inclusions = parent_rubrics.inclusions.clone();
        let mut exclusions = parent_rubrics.exclusions.clone();
        let mut coding_hints = parent_rubrics.coding_hints.clone();
        let mut definitions = parent_rubrics.definitions.clone();
        let mut notes = parent_rubrics.notes.clone();
        let texts = parent_rubrics.texts.clone();

        for val in combo {
            inclusions.extend(val.inclusions.iter().cloned());
            exclusions.extend(val.exclusions.iter().cloned());
            coding_hints.extend(val.coding_hints.iter().cloned());
            definitions.extend(val.definitions.iter().cloned());
            notes.extend(val.notes.iter().cloned());
        }

        results.push(Category {
            code,
            label,
            label_long: None,
            is_terminal: true,
            super_class: Some(class.code.clone()),
            sub_classes: Vec::new(),
            breadcrumb,
            inclusions,
            exclusions,
            coding_hints,
            definitions,
            notes,
            texts,
            mod_codes: Vec::new(),
            modifiers: Vec::new(),
        });
    }

    results
}

/// Build the cartesian product of modifier value sets.
fn cartesian_product<'a>(sets: &[Vec<&'a ModifierValue>]) -> Vec<Vec<&'a ModifierValue>> {
    let mut result: Vec<Vec<&'a ModifierValue>> = vec![vec![]];
    for set in sets {
        let mut next = Vec::with_capacity(result.len() * set.len());
        for existing in &result {
            for &item in set {
                let mut combo = existing.clone();
                combo.push(item);
                next.push(combo);
            }
        }
        result = next;
    }
    result
}

/// Check if a modifier value combination has an excludeOnPrecedingModifier conflict.
fn has_exclusion_conflict(combo: &[&ModifierValue], modifier_codes: &[&str]) -> bool {
    for (i, val) in combo.iter().enumerate() {
        for excl in &val.excludes {
            for (j, preceding_val) in combo[..i].iter().enumerate() {
                if modifier_codes[j] == excl.modifier && preceding_val.code == excl.code {
                    return true;
                }
            }
        }
    }
    false
}

fn build_breadcrumb(code: &str, class_map: &HashMap<&str, &model::Class>) -> Vec<BreadcrumbEntry> {
    let mut crumbs = Vec::new();

    let mut current = code;
    while let Some(class) = class_map.get(current) {
        if let Some(parent) = class.super_classes.first() {
            let kind = class_map
                .get(parent.code.as_str())
                .map(|c| c.kind.clone())
                .unwrap_or_default();
            crumbs.push(BreadcrumbEntry {
                code: parent.code.clone(),
                kind,
            });
            current = &parent.code;
        } else {
            break;
        }
    }

    crumbs.reverse();
    crumbs
}
