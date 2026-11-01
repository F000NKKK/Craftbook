//! Span-preserving TOML lookups. Parsed source offsets are never reconstructed by text search.
use std::path::Path;

use toml_edit::{Document, Item};

use crate::source::SourceSpan;

fn document(source: &str) -> Option<Document<String>> {
    source.parse().ok()
}

fn span(source: &str, item: &Item) -> Option<SourceSpan> {
    let range = item.span()?;
    SourceSpan::new(range.start, range.end, source)
}

/// Returns the value span for one unambiguous direct Cargo dependency declaration.
pub fn dependency_span(manifest: &Path, package: &str) -> Option<SourceSpan> {
    let source = std::fs::read_to_string(manifest).ok()?;
    dependency_span_in(&source, package)
}

pub fn dependency_span_in(source: &str, package: &str) -> Option<SourceSpan> {
    let doc = document(source)?;
    let root = doc.as_table();
    let mut found = Vec::new();
    for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(table) = root.get(section).and_then(Item::as_table) {
            collect(source, table, package, &mut found);
        }
    }
    if let Some(targets) = root.get("target").and_then(Item::as_table) {
        for (_, target) in targets.iter() {
            let Some(target) = target.as_table() else { continue };
            for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
                if let Some(table) = target.get(section).and_then(Item::as_table) {
                    collect(source, table, package, &mut found);
                }
            }
        }
    }
    (found.len() == 1).then(|| found.remove(0)).flatten()
}

fn collect(
    source: &str,
    table: &toml_edit::Table,
    package: &str,
    found: &mut Vec<Option<SourceSpan>>,
) {
    for (key, value) in table.iter() {
        let actual = value
            .as_value()
            .and_then(toml_edit::Value::as_inline_table)
            .and_then(|entry| entry.get("package"))
            .and_then(toml_edit::Value::as_str)
            .or_else(|| value.as_table().and_then(|entry| entry.get("package"))
                .and_then(Item::as_str))
            .unwrap_or(key);
        if actual == package {
            found.push(span(source, value));
        }
    }
}

/// Returns the exact parsed array-of-tables entry, not a guessed header position.
pub fn rule_span(source: &str, index: usize) -> Option<SourceSpan> {
    let doc = document(source)?;
    let array = doc.as_table().get("dependency_rule")?.as_array_of_tables()?;
    let table = array.get(index)?;
    let range = table.span()?;
    SourceSpan::new(range.start, range.end, source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_aliased_and_target_specific_dependencies() {
        let text = "[target.'cfg(unix)'.dependencies]\nrenamed = { package = 'core', path = '../core' }\n";
        let result = dependency_span_in(text, "core").unwrap();
        assert!(result.bytes.start < result.bytes.end);
        assert!(text[result.bytes].contains("package = 'core'"));
    }

    #[test]
    fn does_not_guess_ambiguous_dependency_position() {
        let text = "[dependencies]\ncore = '1'\n[dev-dependencies]\ncore = '1'\n";
        assert!(dependency_span_in(text, "core").is_none());
    }

    #[test]
    fn finds_rule_in_unicode_crlf_source() {
        let text = "project = 'пример'\r\n[[dependency_rule]]\r\nfrom = 'a'\r\ndisallow = 'b'\r\n";
        let span = rule_span(text, 0).unwrap();
        let (start, _) = span.positions(text).unwrap();
        assert_eq!(start.line, 2);
    }
}
