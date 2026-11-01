//! Workspace dependency rules backed by Cargo's own metadata output.
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Metadata {
    packages: Vec<Package>,
    workspace_members: Vec<String>,
    resolve: Option<Resolve>,
}

#[derive(Debug, Deserialize)]
struct Package {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct Resolve {
    nodes: Vec<Node>,
}

#[derive(Debug, Deserialize)]
struct Node {
    id: String,
    #[serde(default)]
    deps: Vec<NodeDependency>,
}

#[derive(Debug, Deserialize)]
struct NodeDependency {
    pkg: String,
}

#[derive(Debug, Deserialize)]
pub struct DependencyRule {
    pub from: String,
    pub disallow: String,
}

/// Obtain the resolved dependency graph from Cargo.
pub fn check_workspace(path: &Path, rules: &[DependencyRule]) -> Result<Vec<String>, String> {
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1"])
        .arg("--manifest-path")
        .arg(path)
        .output()
        .map_err(|err| format!("unable to run cargo metadata: {err}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let text = String::from_utf8(output.stdout)
        .map_err(|err| format!("cargo metadata returned non-UTF8 output: {err}"))?;
    check_metadata(&text, rules)
}

/// Analyze only direct edges between members of the same workspace.
pub fn check_metadata(input: &str, rules: &[DependencyRule]) -> Result<Vec<String>, String> {
    let metadata: Metadata = serde_json::from_str(input)
        .map_err(|err| format!("invalid cargo metadata: {err}"))?;
    let workspace: HashSet<&str> = metadata.workspace_members.iter().map(String::as_str).collect();
    let names: HashMap<&str, &str> = metadata
        .packages
        .iter()
        .filter(|package| workspace.contains(package.id.as_str()))
        .map(|package| (package.id.as_str(), package.name.as_str()))
        .collect();
    let mut findings = Vec::new();
    let nodes = metadata
        .resolve
        .ok_or("cargo metadata lacks a resolved dependency graph")?
        .nodes;
    for node in nodes {
        let Some(&source) = names.get(node.id.as_str()) else {
            continue;
        };
        for dependency in node.deps {
            let Some(&target) = names.get(dependency.pkg.as_str()) else {
                continue;
            };
            for rule in rules {
                if rule.from == source && rule.disallow == target {
                    findings.push(format!("{source} must not depend on {target}"));
                }
            }
        }
    }
    findings.sort();
    findings.dedup();
    Ok(findings)
}

#[cfg(test)]
mod tests {
    use super::*;

    const METADATA: &str = r#"{
      "packages": [{"id":"local:a","name":"a"},{"id":"local:b","name":"b"},{"id":"registry:c","name":"c"}],
      "workspace_members":["local:a","local:b"],
      "resolve":{"nodes":[
        {"id":"local:a","deps":[{"pkg":"local:b"},{"pkg":"registry:c"}]},
        {"id":"local:b","deps":[]},
        {"id":"registry:c","deps":[]}
      ]}
    }"#;

    #[test]
    fn catches_forbidden_direct_workspace_edge() {
        let rules = vec![DependencyRule {
            from: "a".into(),
            disallow: "b".into(),
        }];
        assert_eq!(
            check_metadata(METADATA, &rules).unwrap(),
            vec!["a must not depend on b"]
        );
    }

    #[test]
    fn ignores_external_dependencies() {
        let rules = vec![DependencyRule {
            from: "a".into(),
            disallow: "c".into(),
        }];
        assert!(check_metadata(METADATA, &rules).unwrap().is_empty());
    }

    #[test]
    fn requires_resolved_graph() {
        assert!(check_metadata(
            r#"{"packages":[],"workspace_members":[],"resolve":null}"#,
            &[]
        )
        .is_err());
    }
}
