//! Structured diagnostics for CI and code scanning.
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
    pub rule_id: String,
    pub message: String,
    pub path: String,
}

impl Diagnostic {
    pub fn new(rule_id: &str, message: impl Into<String>, path: &Path) -> Self {
        Self {
            rule_id: rule_id.to_owned(),
            message: message.into(),
            path: path.display().to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Text,
    Json,
    Sarif,
}

impl Format {
    pub fn parse(raw: &str) -> Result<Self, String> {
        match raw {
            "text" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            "sarif" => Ok(Self::Sarif),
            _ => Err(format!("unsupported output format: {raw}")),
        }
    }
}

pub fn render(format: Format, findings: &[Diagnostic]) -> Result<String, String> {
    match format {
        Format::Text => Ok(findings
            .iter()
            .map(|item| format!("{}: [{}] {}", item.path, item.rule_id, item.message))
            .collect::<Vec<_>>()
            .join("\n")),
        Format::Json => serde_json::to_string_pretty(&serde_json::json!({
            "schemaVersion": 1,
            "tool": "cargo-craftcheck",
            "success": findings.is_empty(),
            "diagnostics": findings
        }))
        .map_err(|error| error.to_string()),
        Format::Sarif => {
            let results: Vec<_> = findings
                .iter()
                .map(|item| {
                    serde_json::json!({
                        "ruleId": item.rule_id,
                        "level": "error",
                        "message": {"text": item.message},
                        "locations": [{"physicalLocation": {
                            "artifactLocation": {"uri": item.path}
                        }}]
                    })
                })
                .collect();
            serde_json::to_string_pretty(&serde_json::json!({
                "version": "2.1.0",
                "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
                "runs": [{
                    "tool": {"driver": {
                        "name": "cargo-craftcheck",
                        "rules": [
                            {"id": "CB001", "shortDescription": {"text": "Invalid CraftBook policy"}},
                            {"id": "CB002", "shortDescription": {"text": "Forbidden workspace dependency"}},
                            {"id": "CB003", "shortDescription": {"text": "Cargo workspace inspection failed"}}
                        ]
                    }},
                    "results": results
                }]
            }))
            .map_err(|error| error.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_json_is_success() {
        let result = render(Format::Json, &[]).unwrap();
        let value: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(value["success"], true);
    }

    #[test]
    fn sarif_includes_rule_and_location() {
        let findings = [Diagnostic::new(
            "CB002",
            "forbidden edge",
            Path::new("Cargo.toml"),
        )];
        let result = render(Format::Sarif, &findings).unwrap();
        let value: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(value["version"], "2.1.0");
        assert_eq!(value["runs"][0]["results"][0]["ruleId"], "CB002");
    }

    #[test]
    fn unknown_format_is_rejected() {
        assert!(Format::parse("xml").is_err());
    }
}
