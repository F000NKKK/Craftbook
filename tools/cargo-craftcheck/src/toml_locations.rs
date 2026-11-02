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

/// Locate a policy value or an individual array element in the parsed TOML document.
pub fn policy_span(source: &str, field: &str, index: Option<usize>) -> Option<SourceSpan> {
    let doc = document(source)?;
    let item = doc.as_table().get(field)?;
    match index {
        None => span(source, item),
        Some(index) if field == "profiles" => {
            let value = item.as_value()?.as_array()?.get(index)?;
            let range = value.span()?;
            SourceSpan::new(range.start, range.end, source)
        }
        Some(index) if field == "dependency_rule" => {
            let range =
                if let Some(table) = item.as_array_of_tables().and_then(|array| array.get(index)) {
                    table.span()?
                } else {
                    item.as_array()?.get(index)?.span()?
                };
            SourceSpan::new(range.start, range.end, source)
        }
        Some(_) => None,
    }
}

/// Locate an effective policy field, including nested tables and individual rules.
pub fn policy_path_span(source: &str, path: &[&str], index: Option<usize>) -> Option<SourceSpan> {
    let doc = document(source)?;
    let (first, rest) = path.split_first()?;
    let mut current = doc.as_table().get(first)?;
    for part in rest {
        current = current.as_table_like()?.get(part)?;
    }
    match index {
        None => span(source, current),
        Some(index) if path == ["profiles"] => {
            let value = current.as_array()?.get(index)?;
            let range = value.span()?;
            SourceSpan::new(range.start, range.end, source)
        }
        Some(index) if path == ["dependency_rule"] => {
            let range = if let Some(table) = current
                .as_array_of_tables()
                .and_then(|array| array.get(index))
            {
                table.span()?
            } else {
                current.as_array()?.get(index)?.span()?
            };
            SourceSpan::new(range.start, range.end, source)
        }
        Some(_) => None,
    }
}

/// Locate a field in either array-of-tables or inline-array rules.
pub fn rule_field_span(source: &str, index: usize, field: &str) -> Option<SourceSpan> {
    let doc = document(source)?;
    let item = doc.as_table().get("dependency_rule")?;
    if let Some(table) = item.as_array_of_tables().and_then(|array| array.get(index)) {
        return span(source, table.get(field)?);
    }
    let rule = item.as_array()?.get(index)?;
    let value = rule.as_inline_table()?.get(field)?;
    let range = value.span()?;
    SourceSpan::new(range.start, range.end, source)
}

/// Locate the first invalid (empty) dependency-rule field, if present.
pub fn invalid_rule_field_span(source: &str, index: usize) -> Option<SourceSpan> {
    let doc = document(source)?;
    let item = doc.as_table().get("dependency_rule")?;
    for field in ["from", "disallow"] {
        let value =
            if let Some(table) = item.as_array_of_tables().and_then(|array| array.get(index)) {
                table.get(field)?.as_value()?
            } else {
                item.as_array()?.get(index)?.as_inline_table()?.get(field)?
            };
        if value.as_str().is_some_and(|text| text.trim().is_empty()) {
            let range = value.span()?;
            return SourceSpan::new(range.start, range.end, source);
        }
    }
    None
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
            let Some(target) = target.as_table() else {
                continue;
            };
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
            .or_else(|| {
                value
                    .as_table()
                    .and_then(|entry| entry.get("package"))
                    .and_then(Item::as_str)
            })
            .unwrap_or(key);
        if actual == package {
            found.push(span(source, value));
        }
    }
}

/// Returns the exact parsed rule entry for either supported TOML form.
pub fn rule_span(source: &str, index: usize) -> Option<SourceSpan> {
    policy_span(source, "dependency_rule", Some(index))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_rules_have_precise_ranges() {
        let source = "dependency_rule = [{ from = '', disallow = 'api' }, { from = 'web' }]\\r\\n";
        let entry = rule_span(source, 0).unwrap();
        assert_eq!(&source[entry.bytes], "{ from = '', disallow = 'api' }");
        let missing = rule_span(source, 1).unwrap();
        assert_eq!(&source[missing.bytes], "{ from = 'web' }");
        let field = rule_field_span(source, 0, "from").unwrap();
        assert_eq!(&source[field.bytes], "''");
        let invalid = invalid_rule_field_span(source, 0).unwrap();
        assert_eq!(&source[invalid.bytes], "''");
    }

    #[test]
    fn locates_exact_semantic_field_and_element_ranges() {
        let source = "version = 1\r\nproject = '  '\r\nprofiles = ['simple', 'unknown', 'unknown']\r\n[[dependency_rule]]\r\nfrom = ''\r\ndisallow = 'api'\r\n";
        let version = policy_span(source, "version", None).unwrap();
        assert_eq!(&source[version.bytes], "1");
        let project = policy_span(source, "project", None).unwrap();
        assert_eq!(&source[project.bytes], "'  '");
        let second = policy_span(source, "profiles", Some(2)).unwrap();
        assert_eq!(&source[second.bytes], "'unknown'");
        let rule = policy_span(source, "dependency_rule", Some(0)).unwrap();
        assert!(source[rule.bytes].contains("dependency_rule"));
        let invalid = invalid_rule_field_span(source, 0).unwrap();
        assert_eq!(&source[invalid.bytes], "''");
    }

    #[test]
    fn finds_aliased_and_target_specific_dependencies() {
        let text =
            "[target.'cfg(unix)'.dependencies]\nrenamed = { package = 'core', path = '../core' }\n";
        let result = dependency_span_in(text, "core").unwrap();
        assert!(result.bytes.start < result.bytes.end);
        assert!(text[result.bytes].contains("package = 'core'"));
    }

    #[test]
    fn reports_exact_value_column_with_unicode_and_crlf() {
        let text = "# русский комментарий\r\n[dependencies]\r\ncore = { path = '../core' }\r\n";
        let span = dependency_span_in(text, "core").unwrap();
        let (start, end) = span.positions(text).unwrap();
        assert_eq!(start.line, 3);
        assert_eq!(start.column, 8);
        assert_eq!(end.line, 3);
        assert!(end.column > start.column);
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
