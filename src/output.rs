use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Output {
    pub chapters: Vec<Chapter>,
    pub blocks: Vec<Block>,
    pub categories: Vec<Category>,
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
    pub notes: Vec<String>,
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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub inclusions: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub exclusions: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Category {
    pub code: String,
    pub label: String,
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
    pub modifiers: Vec<ModifierGroup>,
}

#[derive(Debug, Serialize)]
pub struct BreadcrumbEntry {
    pub code: String,
    pub kind: String,
}

#[derive(Debug, Serialize)]
pub struct ModifierGroup {
    pub code: String,
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

#[derive(Debug, Serialize)]
pub struct ModifierExclusion {
    pub modifier: String,
    pub code: String,
}
