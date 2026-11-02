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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_column: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_column: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_end_line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_end_column: Option<usize>,
}

impl Diagnostic {
    pub fn new(rule_id: &str, message: impl Into<String>, path: &Path) -> Self {
        Self {
            rule_id: rule_id.to_owned(),
            message: message.into(),
            path: path.display().to_string(),
            config_path: None,
            rule_index: None,
            line: None,
            column: None,
            end_line: None,
            end_column: None,
            config_line: None,
            config_column: None,
            config_end_line: None,
            config_end_column: None,
        }
    }

    pub fn at(mut self, line: usize, column: usize) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }

    pub fn with_span(mut self, source: &str, span: &crate::source::SourceSpan) -> Self {
        if let Some((start, end)) = span.positions(source) {
            self.line = Some(start.line);
            self.column = Some(start.column);
            self.end_line = Some(end.line);
            self.end_column = Some(end.column);
        }
        self
    }

    pub fn with_end(mut self, line: usize, column: usize) -> Self {
        self.end_line = Some(line);
        self.end_column = Some(column);
        self
    }

    pub fn with_rule_end(mut self, position: Option<(usize, usize)>) -> Self {
        if let Some((line, column)) = position {
            self.config_end_line = Some(line);
            self.config_end_column = Some(column);
        }
        self
    }

    pub fn with_rule_column(mut self, column: Option<usize>) -> Self {
        self.config_column = column;
        self
    }

    pub fn with_rule_line(mut self, line: Option<usize>) -> Self {
        self.config_line = line;
        self
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

fn physical_location(
    path: &str,
    root: &Path,
    line: Option<usize>,
    column: usize,
    end: Option<(usize, usize)>,
) -> serde_json::Value {
    let mut location = serde_json::json!({
        "artifactLocation": {"uri": sarif_uri(path, root)}
    });
    if let Some(line) = line {
        let mut region = serde_json::json!({
            "startLine": line,
            "startColumn": column
        });
        if let Some((end_line, end_column)) = end {
            region["endLine"] = serde_json::json!(end_line);
            region["endColumn"] = serde_json::json!(end_column);
        }
        location["region"] = region;
    }
    location
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
            .map(|item| {
                let location = match (item.line, item.column) {
                    (Some(line), Some(column)) => format!("{}:{line}:{column}", item.path),
                    (Some(line), None) => format!("{}:{line}", item.path),
                    _ => item.path.clone(),
                };
                format!("{location}: [{}] {}", item.rule_id, item.message)
            })
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
                            "ruleIndex": item.rule_index,
                            "configLine": item.config_line,
                            "configColumn": item.config_column,
                            "configEndLine": item.config_end_line,
                            "configEndColumn": item.config_end_column
                        },
                        "relatedLocations": item.config_path.as_ref().map(|config| {
                            vec![serde_json::json!({
                                "id": 1,
                                "physicalLocation": physical_location(
                                    config,
                                    root,
                                    item.config_line,
                                    item.config_column.unwrap_or(1),
                                    item.config_end_line.zip(item.config_end_column)
                                )
                            })]
                        }).unwrap_or_default(),
                        "locations": [{"physicalLocation": physical_location(
                            &item.path,
                            root,
                            item.line,
                            item.column.unwrap_or(1),
                            item.end_line.zip(item.end_column)
                        )}]
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
    fn text_diagnostics_include_exact_source_position() {
        let finding =
            Diagnostic::new("CB001", "invalid value", Path::new("CraftBook.toml")).at(3, 12);
        let output = render(Format::Text, &[finding]).unwrap();
        assert_eq!(output, "CraftBook.toml:3:12: [CB001] invalid value");
    }

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
        let result = render_with_root(Format::Sarif, &[finding], Path::new("/project")).unwrap();
        let value: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(
            value["runs"][0]["results"][0]["locations"][0]["physicalLocation"]["artifactLocation"]
                ["uri"],
            "crates/alpha/Cargo.toml"
        );
        assert_eq!(value["runs"][0]["results"][0]["properties"]["ruleIndex"], 0);
    }

    #[test]
    fn source_span_renders_start_and_end_in_sarif() {
        let source = "αb\r\nxyz";
        let span = crate::source::SourceSpan::new(5, 8, source).unwrap();
        let finding = Diagnostic::new("CB001", "invalid", Path::new("CraftBook.toml"))
            .with_span(source, &span);
        let output = render(Format::Sarif, &[finding]).unwrap();
        let value: serde_json::Value = serde_json::from_str(&output).unwrap();
        let region = &value["runs"][0]["results"][0]["locations"][0]["physicalLocation"]["region"];
        assert_eq!(region["startLine"], 2);
        assert_eq!(region["startColumn"], 1);
        assert_eq!(region["endLine"], 2);
        assert_eq!(region["endColumn"], 4);
    }

    #[test]
    fn json_and_sarif_share_exact_source_coordinates() {
        let source = "версия = 'bad'\r\n";
        let offset = source.find("'bad'").unwrap();
        let span = crate::source::SourceSpan::new(offset, offset + 5, source).unwrap();
        let finding = Diagnostic::new("CB001", "invalid value", Path::new("CraftBook.toml"))
            .with_span(source, &span);
        let json: serde_json::Value =
            serde_json::from_str(&render(Format::Json, std::slice::from_ref(&finding)).unwrap())
                .unwrap();
        let sarif: serde_json::Value =
            serde_json::from_str(&render(Format::Sarif, &[finding]).unwrap()).unwrap();
        let item = &json["diagnostics"][0];
        let region = &sarif["runs"][0]["results"][0]["locations"][0]["physicalLocation"]["region"];
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(item["line"], region["startLine"]);
        assert_eq!(item["column"], region["startColumn"]);
        assert_eq!(item["end_line"], region["endLine"]);
        assert_eq!(item["end_column"], region["endColumn"]);
    }

    #[test]
    fn unknown_format_is_rejected() {
        assert!(Format::parse("xml").is_err());
    }
    #[test]
    fn unknown_locations_have_no_sarif_region() {
        let finding = Diagnostic::new(
            "CB003",
            "workspace inspection failed",
            Path::new("Cargo.toml"),
        );
        let output = render(Format::Sarif, &[finding]).unwrap();
        let value: serde_json::Value = serde_json::from_str(&output).unwrap();
        let location = &value["runs"][0]["results"][0]["locations"][0]["physicalLocation"];
        assert!(location.get("region").is_none());
    }
}
