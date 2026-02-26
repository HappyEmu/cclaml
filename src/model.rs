#![allow(dead_code)]

use serde::Deserialize;

/// Root ClaML element
#[derive(Debug, Deserialize)]
#[serde(rename = "ClaML")]
pub struct ClaML {
    #[serde(rename = "Modifier", default)]
    pub modifiers: Vec<Modifier>,
    #[serde(rename = "ModifierClass", default)]
    pub modifier_classes: Vec<ModifierClass>,
    #[serde(rename = "Class", default)]
    pub classes: Vec<Class>,
}

/// A Modifier definition (e.g., S04E10_4 for 4th digit extensions)
#[derive(Debug, Deserialize)]
pub struct Modifier {
    #[serde(rename = "@code")]
    pub code: String,
    #[serde(rename = "SubClass", default)]
    pub sub_classes: Vec<SubClass>,
    #[serde(rename = "Rubric", default)]
    pub rubrics: Vec<Rubric>,
}

/// A single value option within a Modifier group
#[derive(Debug, Deserialize)]
pub struct ModifierClass {
    #[serde(rename = "@code")]
    pub code: String,
    #[serde(rename = "@modifier")]
    pub modifier: String,
    #[serde(rename = "@usage")]
    pub usage: Option<String>,
    #[serde(rename = "Meta", default)]
    pub metas: Vec<Meta>,
    #[serde(rename = "SuperClass")]
    pub super_class: Option<SuperClass>,
    #[serde(rename = "Rubric", default)]
    pub rubrics: Vec<Rubric>,
}

/// A Class element (chapter, block, or category)
#[derive(Debug, Deserialize)]
pub struct Class {
    #[serde(rename = "@code")]
    pub code: String,
    #[serde(rename = "@kind")]
    pub kind: String,
    #[serde(rename = "@usage")]
    pub usage: Option<String>,
    #[serde(rename = "Meta", default)]
    pub metas: Vec<Meta>,
    #[serde(rename = "SuperClass", default)]
    pub super_classes: Vec<SuperClass>,
    #[serde(rename = "SubClass", default)]
    pub sub_classes: Vec<SubClass>,
    #[serde(rename = "ModifiedBy", default)]
    pub modified_by: Vec<ModifiedBy>,
    #[serde(rename = "ExcludeModifier", default)]
    pub exclude_modifiers: Vec<ExcludeModifier>,
    #[serde(rename = "Rubric", default)]
    pub rubrics: Vec<Rubric>,
}

/// Links a Class to a Modifier group
#[derive(Debug, Deserialize)]
pub struct ModifiedBy {
    #[serde(rename = "@code")]
    pub code: String,
    #[serde(rename = "@all", default = "default_true")]
    pub all: bool,
    #[serde(rename = "@position")]
    pub position: Option<String>,
    #[serde(rename = "ValidModifierClass", default)]
    pub valid_modifier_classes: Vec<ValidModifierClass>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct ValidModifierClass {
    #[serde(rename = "@code")]
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct ExcludeModifier {
    #[serde(rename = "@code")]
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct SuperClass {
    #[serde(rename = "@code")]
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct SubClass {
    #[serde(rename = "@code")]
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct Meta {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@value")]
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct Rubric {
    #[serde(rename = "@kind")]
    pub kind: String,
    #[serde(rename = "@id")]
    pub id: Option<String>,
    #[serde(rename = "@usage")]
    pub usage: Option<String>,
    #[serde(rename = "Label", default)]
    pub labels: Vec<Label>,
}

/// Mixed content enum for Label children: text, Para, Fragment, Reference, Term, etc.
#[derive(Debug, Deserialize)]
pub enum LabelContent {
    #[serde(rename = "$text")]
    Text(String),
    Para(Para),
    Fragment(Fragment),
    Reference(Reference),
    Term(Term),
    Include(IgnoredElement),
    IncludeDescendants(IgnoredElement),
    List(IgnoredElement),
    Table(IgnoredElement),
}

/// Mixed content enum for Para/Fragment children: text, Reference, Term
#[derive(Debug, Deserialize)]
pub enum SimpleMixed {
    #[serde(rename = "$text")]
    Text(String),
    Reference(Reference),
    Term(Term),
}

/// Catch-all for elements we don't need to deeply parse
#[derive(Debug, Deserialize)]
pub struct IgnoredElement {}

/// Label element — contains mixed content
#[derive(Debug, Deserialize)]
pub struct Label {
    #[serde(rename = "@xml:lang")]
    pub lang: Option<String>,
    #[serde(rename = "@xml:space")]
    pub space: Option<String>,
    #[serde(rename = "$value", default)]
    pub content: Vec<LabelContent>,
}

#[derive(Debug, Deserialize)]
pub struct Para {
    #[serde(rename = "@class")]
    pub class: Option<String>,
    #[serde(rename = "$value", default)]
    pub content: Vec<SimpleMixed>,
}

#[derive(Debug, Deserialize)]
pub struct Fragment {
    #[serde(rename = "@type")]
    pub fragment_type: Option<String>,
    #[serde(rename = "@class")]
    pub class: Option<String>,
    #[serde(rename = "@usage")]
    pub usage: Option<String>,
    #[serde(rename = "$value", default)]
    pub content: Vec<SimpleMixed>,
}

#[derive(Debug, Deserialize)]
pub struct Reference {
    #[serde(rename = "@code")]
    pub code: Option<String>,
    #[serde(rename = "@class")]
    pub class: Option<String>,
    #[serde(rename = "@authority")]
    pub authority: Option<String>,
    #[serde(rename = "@uid")]
    pub uid: Option<String>,
    #[serde(rename = "@usage")]
    pub usage: Option<String>,
    #[serde(rename = "$value", default)]
    pub content: Vec<SimpleMixed>,
}

#[derive(Debug, Deserialize)]
pub struct Term {
    #[serde(rename = "@class")]
    pub class: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

// Helper to flatten mixed content into a string
fn flatten_simple_mixed(content: &[SimpleMixed]) -> String {
    let mut out = String::new();
    for item in content {
        match item {
            SimpleMixed::Text(t) => out.push_str(t),
            SimpleMixed::Reference(r) => out.push_str(&r.flat_text()),
            SimpleMixed::Term(t) => {
                if let Some(text) = &t.text {
                    out.push_str(text);
                }
            }
        }
    }
    out
}

impl Label {
    pub fn flat_text(&self) -> String {
        let mut out = String::new();
        for item in &self.content {
            match item {
                LabelContent::Text(t) => {
                    let trimmed = t.trim();
                    if !trimmed.is_empty() {
                        if !out.is_empty() && !out.ends_with(' ') {
                            out.push(' ');
                        }
                        out.push_str(trimmed);
                    }
                }
                LabelContent::Para(p) => {
                    let text = flatten_simple_mixed(&p.content);
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        if !out.is_empty() && !out.ends_with(' ') {
                            out.push(' ');
                        }
                        out.push_str(trimmed);
                    }
                }
                LabelContent::Fragment(f) => {
                    let text = flatten_simple_mixed(&f.content);
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        if !out.is_empty() && !out.ends_with(' ') {
                            out.push(' ');
                        }
                        out.push_str(trimmed);
                    }
                }
                LabelContent::Reference(r) => {
                    let text = r.flat_text();
                    if !text.is_empty() {
                        if !out.is_empty() && !out.ends_with(' ') {
                            out.push(' ');
                        }
                        out.push_str(&text);
                    }
                }
                LabelContent::Term(t) => {
                    if let Some(text) = &t.text {
                        out.push_str(text);
                    }
                }
                _ => {}
            }
        }
        out.trim().to_string()
    }
}

impl Reference {
    pub fn flat_text(&self) -> String {
        let inner = flatten_simple_mixed(&self.content);
        if inner.is_empty() {
            return String::new();
        }
        let mark = match self.usage.as_deref() {
            Some("dagger") => "\u{2020}",
            Some("aster") => "*",
            Some("optional") => "!",
            _ => "",
        };
        format!("{{{{{inner}{mark}}}}}")
    }
}
