//! Structured diagnostics for CI and code scanning.
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
    pub rule_id: String,
    pub message: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_index: Option<usize>,
}

impl Diagnostic {
    pub fn new(rule_id: &str, message: impl Into<String>, path: &Path) -> Self {
        Self {
            rule_id: rule_id.to_owned(),
            message: message.into(),
            path: path.display().to_string(),
            config_path: None,
            rule_index: None,
        }
    }

    pub fn with_rule(mut self, config: &Path, index: usize) -> Self {
        self.config_path = Some(config.display().to_string());
        self.rule_index = Some(index);
        self
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

fn sarif_uri(path: &str, root: &Path) -> String {
    let input = Path::new(path);
    let resolved = std::fs::canonicalize(input).unwrap_or_else(|_| input.to_path_buf());
    let relative = resolved.strip_prefix(root).unwrap_or(&resolved);
    relative.to_string_lossy().replace('\\', "/")
}

pub fn render(format: Format, findings: &[Diagnostic]) -> Result<String, String> {
    render_with_root(format, findings, Path::new("."))
}

/// Render SARIF paths relative to the selected project root when possible.
pub fn render_with_root(
    format: Format,
    findings: &[Diagnostic],
    root: &Path,
) -> Result<String, String> {
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
                        "properties": {
                            "configPath": item.config_path,
                            "ruleIndex": item.rule_index
                        },
                        "locations": [{"physicalLocation": {
                            "artifactLocation": {"uri": sarif_uri(&item.path, root)}
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
    fn sarif_paths_are_relative_to_project_root() {
        let finding = Diagnostic::new(
            "CB002",
            "forbidden edge",
            Path::new("/project/crates/alpha/Cargo.toml"),
        )
        .with_rule(Path::new("/project/CraftBook.toml"), 0);
        let result = render_with_root(
            Format::Sarif,
            &[finding],
            Path::new("/project"),
        )
        .unwrap();
        let value: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(
            value["runs"][0]["results"][0]["locations"][0]["physicalLocation"]
                ["artifactLocation"]["uri"],
            "crates/alpha/Cargo.toml"
        );
        assert_eq!(
            value["runs"][0]["results"][0]["properties"]["ruleIndex"],
            0
        );
    }

    #[test]
    fn unknown_format_is_rejected() {
        assert!(Format::parse("xml").is_err());
    }
}
