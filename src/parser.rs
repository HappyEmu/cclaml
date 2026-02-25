use anyhow::{Context, Result};
use quick_xml::de::from_str;
use std::fs;
use std::path::Path;

use crate::model::ClaML;

pub fn parse_claml(path: &Path) -> Result<ClaML> {
    let xml = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    // Strip the DOCTYPE declaration since quick-xml's serde doesn't handle it
    let xml = strip_doctype(&xml);

    let claml: ClaML =
        from_str(&xml).with_context(|| "Failed to deserialize ClaML XML")?;

    Ok(claml)
}

fn strip_doctype(xml: &str) -> String {
    // Remove <!DOCTYPE ...> line
    let mut result = String::with_capacity(xml.len());
    for line in xml.lines() {
        if line.trim_start().starts_with("<!DOCTYPE") {
            continue;
        }
        result.push_str(line);
        result.push('\n');
    }
    result
}
