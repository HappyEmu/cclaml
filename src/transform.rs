use std::collections::HashMap;

use crate::model::{self, ClaML};
use crate::output::{Block, BreadcrumbEntry, Category, Chapter, ModifierExclusion, ModifierGroup, ModifierValue, Output};

pub fn transform(claml: &ClaML) -> Output {
    // Phase 2: Build lookup maps
    let modifier_map = build_modifier_map(claml);
    let modifier_class_map = build_modifier_class_map(claml);
    let class_map = build_class_map(claml);

    // Phase 3 & 4 & 5: Build output structs
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
                let modifiers =
                    resolve_modifiers(&class.modified_by, &modifier_map, &modifier_class_map);
                categories.push(build_category(class, breadcrumb, modifiers));
            }
            _ => {}
        }
    }

    Output {
        chapters,
        blocks,
        categories,
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
    if let Some((start, end)) = code.split_once('-') {
        (start.to_string(), end.to_string())
    } else {
        // Single-code blocks like "B99-B99" — shouldn't happen but handle gracefully
        (code.to_string(), code.to_string())
    }
}

fn build_category(
    class: &model::Class,
    breadcrumb: Vec<BreadcrumbEntry>,
    modifiers: Vec<ModifierGroup>,
) -> Category {
    Category {
        code: class.code.clone(),
        label: get_preferred_label(&class.rubrics),
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
    loop {
        if let Some(class) = class_map.get(current) {
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
        } else {
            break;
        }
    }

    crumbs.reverse();
    crumbs
}

fn resolve_modifiers(
    modified_by: &[model::ModifiedBy],
    modifier_map: &HashMap<String, &model::Modifier>,
    modifier_class_map: &HashMap<String, Vec<&model::ModifierClass>>,
) -> Vec<ModifierGroup> {
    let mut groups = Vec::new();

    for mb in modified_by {
        let description = modifier_map
            .get(&mb.code)
            .map(|m| get_text_label(&m.rubrics))
            .unwrap_or_default();

        let values = if let Some(mcs) = modifier_class_map.get(&mb.code) {
            let valid_codes: Option<std::collections::HashSet<&str>> = if !mb.all {
                Some(
                    mb.valid_modifier_classes
                        .iter()
                        .map(|v| v.code.as_str())
                        .collect(),
                )
            } else {
                None
            };

            mcs.iter()
                .filter(|mc| {
                    if let Some(ref valid) = valid_codes {
                        valid.contains(mc.code.as_str())
                    } else {
                        true
                    }
                })
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

        groups.push(ModifierGroup {
            code: mb.code.clone(),
            description,
            values,
        });
    }

    groups
}
