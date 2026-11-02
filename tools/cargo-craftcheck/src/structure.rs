//! Structural validation of policy source files with syntax-tree coordinates.
use std::path::Path;

use crate::report::Diagnostic;
use crate::toml_locations::policy_path_span;

fn diagnostic(path: &Path, source: &str, keys: &[&str], message: String) -> Diagnostic {
    let mut diagnostic = Diagnostic::new("CB001", message, path);
    if let Some(span) = policy_path_span(source, keys, None) {
        diagnostic = diagnostic.with_span(source, &span);
    }
    diagnostic
}

fn wrong_type(value: &toml::Value, kind: &str) -> bool {
    match kind {
        "string" => !value.is_str(),
        "integer" => !value.is_integer(),
        "boolean" => !value.is_bool(),
        "array" => !value.is_array(),
        "table" => !value.is_table(),
        _ => false,
    }
}

/// Validate each declared field independently; inheritance does not affect source locations.
pub fn inspect(path: &Path, source: &str) -> Vec<Diagnostic> {
    let Ok(table) = toml::from_str::<toml::Table>(source) else {
        return Vec::new();
    };
    let mut diagnostics = Vec::new();
    let fields = [
        ("version", "integer"),
        ("project", "string"),
        ("profiles", "array"),
        ("assurance", "table"),
        ("constraints", "table"),
        ("dependency_rule", "array"),
    ];
    for (key, value) in &table {
        if let Some((_, kind)) = fields.iter().find(|(name, _)| name == key) {
            if wrong_type(value, kind) {
                diagnostics.push(diagnostic(
                    path,
                    source,
                    &[key],
                    format!("{key} must be {kind}"),
                ));
                continue;
            }
        } else {
            diagnostics.push(diagnostic(
                path,
                source,
                &[key],
                format!("unknown policy field: {key}"),
            ));
            continue;
        }
        if key == "profiles"
            && let Some(items) = value.as_array()
        {
            for (index, entry) in items.iter().enumerate() {
                if !entry.is_str() {
                    let mut finding =
                        Diagnostic::new("CB001", format!("profiles[{index}] must be string"), path);
                    if let Some(span) = crate::toml_locations::policy_span(source, key, Some(index))
                    {
                        finding = finding.with_span(source, &span);
                    }
                    diagnostics.push(finding);
                }
            }
        }
        if key == "assurance" || key == "constraints" {
            let allowed: &[&str] = if key == "assurance" {
                &["format", "clippy", "test", "docs"]
            } else {
                &["no_std", "public_api_stable", "unsafe_review_required"]
            };
            if let Some(entries) = value.as_table() {
                for (field, entry) in entries {
                    if !allowed.contains(&field.as_str()) {
                        diagnostics.push(diagnostic(
                            path,
                            source,
                            &[key, field],
                            format!("unknown {key} field: {field}"),
                        ));
                    } else if !entry.is_bool() {
                        diagnostics.push(diagnostic(
                            path,
                            source,
                            &[key, field],
                            format!("{key}.{field} must be boolean"),
                        ));
                    }
                }
            }
        }
        if key == "dependency_rule"
            && let Some(rules) = value.as_array()
        {
            for (index, rule) in rules.iter().enumerate() {
                if let Some(fields) = rule.as_table() {
                    for (field, entry) in fields {
                        let message = if field != "from" && field != "disallow" {
                            Some(format!("unknown dependency_rule field: {field}"))
                        } else if !entry.is_str() {
                            Some(format!("dependency_rule[{index}].{field} must be string"))
                        } else {
                            None
                        };
                        if let Some(message) = message {
                            let mut finding = Diagnostic::new("CB001", message, path);
                            if let Some(span) =
                                crate::toml_locations::rule_field_span(source, index, field)
                            {
                                finding = finding.with_span(source, &span);
                            }
                            diagnostics.push(finding);
                        }
                    }
                    for required in ["from", "disallow"] {
                        if !fields.contains_key(required) {
                            let mut finding = Diagnostic::new(
                                "CB001",
                                format!("dependency_rule[{index}] missing {required}"),
                                path,
                            );
                            if let Some(span) = crate::toml_locations::rule_span(source, index) {
                                finding = finding.with_span(source, &span);
                            }
                            diagnostics.push(finding);
                        }
                    }
                } else {
                    let mut finding = Diagnostic::new(
                        "CB001",
                        format!("dependency_rule[{index}] must be table"),
                        path,
                    );
                    if let Some(span) = crate::toml_locations::policy_span(source, key, Some(index))
                    {
                        finding = finding.with_span(source, &span);
                    }
                    diagnostics.push(finding);
                }
            }
        }
    }
    diagnostics
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_declared_scalar_and_nested_types_report_source_ranges() {
        let cases = [
            ("version = '2'", "version must be integer"),
            ("project = 12", "project must be string"),
            ("profiles = 'simple'", "profiles must be array"),
            ("profiles = [true]", "profiles[0] must be string"),
            ("assurance = false", "assurance must be table"),
            ("constraints = 12", "constraints must be table"),
            ("dependency_rule = 'no'", "dependency_rule must be array"),
            ("[assurance]\nformat = 3", "assurance.format must be boolean"),
            ("[assurance]\ndocs = 3", "assurance.docs must be boolean"),
            ("[constraints]\nno_std = 3", "constraints.no_std must be boolean"),
            ("[constraints]\npublic_api_stable = 3", "constraints.public_api_stable must be boolean"),
            ("[constraints]\nunsafe_review_required = 3", "constraints.unsafe_review_required must be boolean"),
            ("dependency_rule = [{ from = 7, disallow = 'api' }]", "dependency_rule[0].from must be string"),
            ("dependency_rule = [{ from = 'api', disallow = false }]", "dependency_rule[0].disallow must be string"),
        ];
        for (source, expected) in cases {
            let findings = inspect(Path::new("CraftBook.toml"), source);
            let finding = findings.iter().find(|d| d.message == expected).unwrap_or_else(|| panic!("missing {expected} for {source}"));
            assert!(finding.line.is_some(), "{expected}");
            assert!(finding.column.is_some(), "{expected}");
            assert!(finding.end_line.is_some(), "{expected}");
            assert!(finding.end_column.is_some(), "{expected}");
        }
    }

    #[test]
    fn reports_nested_shape_and_type_locations() {
        let source = "version = 'wrong'\r\nproject = 7\r\nprofiles = ['simple', 4]\r\n[assurance]\r\nclippy = 'yes'\r\nmagic = true\r\n";
        let diagnostics = inspect(Path::new("CraftBook.toml"), source);
        for expected in [
            "version must be integer",
            "project must be string",
            "profiles[1] must be string",
            "assurance.clippy must be boolean",
            "unknown assurance field: magic",
        ] {
            let found = diagnostics.iter().find(|d| d.message == expected).unwrap();
            assert!(found.line.is_some(), "{expected}");
            assert!(found.column.is_some(), "{expected}");
            assert!(found.end_line.is_some(), "{expected}");
            assert!(found.end_column.is_some(), "{expected}");
        }
    }

    #[test]
    fn reports_unknown_fields_and_incomplete_rules() {
        let source = "version = 2\nproject = 'x'\nunknown = true\n[[dependency_rule]]\nfrom = 42\nextra = 'x'\n";
        let diagnostics = inspect(Path::new("CraftBook.toml"), source);
        assert!(
            diagnostics
                .iter()
                .any(|d| d.message == "unknown policy field: unknown" && d.line == Some(3))
        );
        assert!(diagnostics.iter().any(|d| d.message == "dependency_rule[0].from must be string" && d.line == Some(5)));
        assert!(
            diagnostics
                .iter()
                .any(|d| d.message == "unknown dependency_rule field: extra" && d.line == Some(6))
        );
        assert!(
            diagnostics
                .iter()
                .any(|d| d.message == "dependency_rule[0] missing disallow" && d.line == Some(4))
        );
    }
}
