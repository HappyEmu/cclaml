use std::collections::HashMap;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Output {
    pub chapters: Vec<Chapter>,
    pub blocks: Vec<Block>,
    pub categories: Vec<Category>,
    pub modifiers: HashMap<String, ModifierGroup>,
}

#[derive(Debug, Serialize)]
pub struct Chapter {
    pub code: String,
    pub label: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sub_classes: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub inclusions: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub exclusions: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub coding_hints: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub introductions: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub texts: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Block {
    pub code: String,
    pub label: String,
    pub range_start: String,
    pub range_end: String,
    pub super_class: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sub_classes: Vec<String>,
    pub breadcrumb: Vec<BreadcrumbEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub inclusions: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub exclusions: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub coding_hints: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub texts: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Category {
    pub code: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_long: Option<String>,
    pub is_terminal: bool,
    pub super_class: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sub_classes: Vec<String>,
    pub breadcrumb: Vec<BreadcrumbEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub inclusions: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub exclusions: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub coding_hints: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub definitions: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub texts: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub mod_codes: Vec<ModCode>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<ModifierRef>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BreadcrumbEntry {
    pub code: String,
    pub kind: String,
}

#[derive(Debug, Serialize)]
pub struct ModifierRef {
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_values: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct ModifierGroup {
    pub description: String,
    pub values: Vec<ModifierValue>,
}

#[derive(Debug, Serialize)]
pub struct ModifierValue {
    pub code: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub inclusions: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub exclusions: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub coding_hints: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub definitions: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub excludes: Vec<ModifierExclusion>,
}

#[derive(Debug)]
pub struct ModCode {
    pub code: String,
    pub digits: Vec<ModDigit>,
}

impl Serialize for ModCode {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(1 + self.digits.len()))?;
        map.serialize_entry("code", &self.code)?;
        for digit in &self.digits {
            map.serialize_entry(&digit.index.to_string(), &digit.modifier)?;
        }
        map.end()
    }
}

#[derive(Debug)]
pub struct ModDigit {
    pub index: usize,
    pub modifier: String,
}

#[derive(Debug, Serialize)]
pub struct ModifierExclusion {
    pub modifier: String,
    pub code: String,
}
