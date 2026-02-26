use std::collections::HashMap;

use crate::model::{self, ClaML};
use crate::output::{
    Block, BreadcrumbEntry, Category, Chapter, ModifierExclusion, ModifierGroup, ModifierRef,
    ModifierValue, Output,
};

pub fn transform(claml: &ClaML) -> Output {
    let modifier_map = build_modifier_map(claml);
    let modifier_class_map = build_modifier_class_map(claml);
    let class_map = build_class_map(claml);

    // Build top-level modifier definitions once
    let modifier_definitions = build_modifier_definitions(&modifier_map, &modifier_class_map);

    let mut chapters = Vec::new();
    let mut blocks = Vec::new();
    let mut categories = Vec::new();

    for class in &claml.classes {
        match class.kind.as_str() {
            "chapter" => {
                chapters.push(build_chapter(class));
            }
            "block" => {
                blocks.push(build_block(class));
            }
            "category" => {
                let breadcrumb = build_breadcrumb(&class.code, &class_map);
                let modifiers = resolve_modifier_refs(&class.modified_by);
                categories.push(build_category(class, breadcrumb, modifiers));
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

fn build_block(class: &model::Class) -> Block {
    let (range_start, range_end) = parse_block_range(&class.code);
    Block {
        code: class.code.clone(),
        label: get_preferred_label(&class.rubrics),
        range_start,
        range_end,
        super_class: class.super_classes.first().map(|s| s.code.clone()),
        sub_classes: class.sub_classes.iter().map(|s| s.code.clone()).collect(),
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

fn build_breadcrumb(code: &str, class_map: &HashMap<String, &model::Class>) -> Vec<BreadcrumbEntry> {
    let mut crumbs = Vec::new();

    if let Some(class) = class_map.get(code) {
        crumbs.push(BreadcrumbEntry {
            code: code.to_string(),
            kind: class.kind.clone(),
        });
    }

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
