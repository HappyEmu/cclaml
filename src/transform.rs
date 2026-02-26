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
    let exclude_modifier_set: HashMap<String, HashSet<String>> = claml
        .classes
        .iter()
        .filter(|c| !c.exclude_modifiers.is_empty())
        .map(|c| {
            let set: HashSet<String> = c.exclude_modifiers.iter().map(|e| e.code.clone()).collect();
            (c.code.clone(), set)
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
                    // Emit parent category without modifier refs
                    categories.push(build_category(class, breadcrumb.clone(), Vec::new()));
                    // Emit expanded categories for each resolved modifier combination
                    let expanded = expand_modifiers(
                        class,
                        &breadcrumb,
                        &modifier_definitions,
                        &exclude_modifier_set,
                    );
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

fn build_modifier_map(claml: &ClaML) -> HashMap<String, &model::Modifier> {
    claml.modifiers.iter().map(|m| (m.code.clone(), m)).collect()
}

fn build_modifier_class_map(claml: &ClaML) -> HashMap<String, Vec<&model::ModifierClass>> {
    let mut map: HashMap<String, Vec<&model::ModifierClass>> = HashMap::new();
    for mc in &claml.modifier_classes {
        map.entry(mc.modifier.clone()).or_default().push(mc);
    }
    map
}

fn build_class_map(claml: &ClaML) -> HashMap<String, &model::Class> {
    claml.classes.iter().map(|c| (c.code.clone(), c)).collect()
}

fn build_modifier_definitions(
    modifier_map: &HashMap<String, &model::Modifier>,
    modifier_class_map: &HashMap<String, Vec<&model::ModifierClass>>,
) -> HashMap<String, ModifierGroup> {
    let mut definitions = HashMap::new();

    for (code, modifier) in modifier_map {
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
                    ModifierValue {
                        code: mc.code.clone(),
                        label: get_preferred_label(&mc.rubrics),
                        usage: mc.usage.clone(),
                        inclusions: get_rubric_labels(&mc.rubrics, "inclusion"),
                        exclusions: get_rubric_labels(&mc.rubrics, "exclusion"),
                        coding_hints: get_rubric_labels(&mc.rubrics, "coding-hint"),
                        definitions: get_rubric_labels(&mc.rubrics, "definition"),
                        notes: get_rubric_labels(&mc.rubrics, "note"),
                        excludes,
                    }
                })
                .collect()
        } else {
            Vec::new()
        };

        definitions.insert(
            code.clone(),
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

fn get_preferred_label(rubrics: &[model::Rubric]) -> String {
    for rubric in rubrics {
        if rubric.kind == "preferred" {
            if let Some(label) = rubric.labels.first() {
                return label.flat_text();
            }
        }
    }
    String::new()
}

fn get_rubric_labels(rubrics: &[model::Rubric], kind: &str) -> Vec<String> {
    rubrics
        .iter()
        .filter(|r| r.kind == kind)
        .flat_map(|r| r.labels.iter().map(|l| l.flat_text()))
        .filter(|s| !s.is_empty())
        .collect()
}

fn get_preferred_long_label(rubrics: &[model::Rubric]) -> String {
    for rubric in rubrics {
        if rubric.kind == "preferredLong" {
            if let Some(label) = rubric.labels.first() {
                return label.flat_text();
            }
        }
    }
    String::new()
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
    Chapter {
        code: class.code.clone(),
        label: get_preferred_label(&class.rubrics),
        sub_classes: class.sub_classes.iter().map(|s| s.code.clone()).collect(),
        inclusions: get_rubric_labels(&class.rubrics, "inclusion"),
        exclusions: get_rubric_labels(&class.rubrics, "exclusion"),
        notes: get_rubric_labels(&class.rubrics, "note"),
    }
}

fn build_block(class: &model::Class, breadcrumb: Vec<BreadcrumbEntry>) -> Block {
    let (range_start, range_end) = parse_block_range(&class.code);
    Block {
        code: class.code.clone(),
        label: get_preferred_label(&class.rubrics),
        range_start,
        range_end,
        super_class: class.super_classes.first().map(|s| s.code.clone()),
        sub_classes: class.sub_classes.iter().map(|s| s.code.clone()).collect(),
        breadcrumb,
        inclusions: get_rubric_labels(&class.rubrics, "inclusion"),
        exclusions: get_rubric_labels(&class.rubrics, "exclusion"),
        notes: get_rubric_labels(&class.rubrics, "note"),
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
    let label_long = get_preferred_long_label(&class.rubrics);
    Category {
        code: class.code.clone(),
        label: get_preferred_label(&class.rubrics),
        label_long: if label_long.is_empty() {
            None
        } else {
            Some(label_long)
        },
        is_terminal: class.sub_classes.is_empty(),
        super_class: class.super_classes.first().map(|s| s.code.clone()),
        sub_classes: class.sub_classes.iter().map(|s| s.code.clone()).collect(),
        breadcrumb,
        inclusions: get_rubric_labels(&class.rubrics, "inclusion"),
        exclusions: get_rubric_labels(&class.rubrics, "exclusion"),
        coding_hints: get_rubric_labels(&class.rubrics, "coding-hint"),
        definitions: get_rubric_labels(&class.rubrics, "definition"),
        notes: get_rubric_labels(&class.rubrics, "note"),
        modifiers,
    }
}

/// Expand a category's modifiers into individual flat categories.
/// For categories with multiple modifiers, only fully-resolved combinations are emitted.
fn expand_modifiers(
    class: &model::Class,
    parent_breadcrumb: &[BreadcrumbEntry],
    modifier_definitions: &HashMap<String, ModifierGroup>,
    exclude_modifier_set: &HashMap<String, HashSet<String>>,
) -> Vec<Category> {
    let excluded = exclude_modifier_set.get(&class.code);

    // Collect valid modifier values for each ModifiedBy, respecting ExcludeModifier
    let modifier_value_sets: Vec<Vec<&ModifierValue>> = class
        .modified_by
        .iter()
        .filter(|mb| {
            // Skip modifiers excluded by ExcludeModifier on this category
            excluded.map_or(true, |set| !set.contains(&mb.code))
        })
        .map(|mb| {
            let group = match modifier_definitions.get(&mb.code) {
                Some(g) => g,
                None => return Vec::new(),
            };
            group
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
                .collect()
        })
        .collect();

    // If any modifier has no valid values, skip expansion entirely
    if modifier_value_sets.is_empty() || modifier_value_sets.iter().any(|s| s.is_empty()) {
        return Vec::new();
    }

    let modifier_codes: Vec<&str> = class
        .modified_by
        .iter()
        .filter(|mb| excluded.map_or(true, |set| !set.contains(&mb.code)))
        .map(|mb| mb.code.as_str())
        .collect();

    // Build cartesian product — for 2+ modifiers only fully-resolved combos
    let combinations = cartesian_product(&modifier_value_sets);

    // Collect parent metadata once
    let parent_inclusions = get_rubric_labels(&class.rubrics, "inclusion");
    let parent_exclusions = get_rubric_labels(&class.rubrics, "exclusion");
    let parent_coding_hints = get_rubric_labels(&class.rubrics, "coding-hint");
    let parent_definitions = get_rubric_labels(&class.rubrics, "definition");
    let parent_notes = get_rubric_labels(&class.rubrics, "note");

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

        // Label: last modifier value's label (most specific)
        let label = combo
            .last()
            .map(|v| v.label.clone())
            .unwrap_or_default();

        // Breadcrumb: parent's ancestors + parent itself
        let mut breadcrumb = parent_breadcrumb.to_vec();
        breadcrumb.push(BreadcrumbEntry {
            code: class.code.clone(),
            kind: "category".to_string(),
        });

        // Merge metadata: parent + all modifier values in order
        let mut inclusions = parent_inclusions.clone();
        let mut exclusions = parent_exclusions.clone();
        let mut coding_hints = parent_coding_hints.clone();
        let mut definitions = parent_definitions.clone();
        let mut notes = parent_notes.clone();

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

fn build_breadcrumb(code: &str, class_map: &HashMap<String, &model::Class>) -> Vec<BreadcrumbEntry> {
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
