use anyhow::{Context, Result};
use quick_xml::de::from_str;
use std::borrow::Cow;
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

fn strip_doctype(xml: &str) -> Cow<'_, str> {
    // Find the DOCTYPE declaration and splice it out without copying the entire file line-by-line
    let Some(start) = xml.find("<!DOCTYPE") else {
        return Cow::Borrowed(xml);
    };

    // Find the end of the DOCTYPE line
    let end = xml[start..].find('\n').map_or(xml.len(), |pos| start + pos + 1);

    let mut result = String::with_capacity(xml.len() - (end - start));
    result.push_str(&xml[..start]);
    result.push_str(&xml[end..]);
    Cow::Owned(result)
}
