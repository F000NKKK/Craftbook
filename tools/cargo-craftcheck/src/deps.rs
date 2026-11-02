//! Workspace dependency rules backed by Cargo's own metadata output.
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
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
    #[serde(default)]
    manifest_path: String,
    #[serde(default)]
    metadata: serde_json::Value,
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
    #[serde(default)]
    pub from: String,
    #[serde(default)]
    pub disallow: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub when: Option<String>,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
}

fn classified(package: &Package, policy: &crate::Policy) -> serde_json::Value {
    let metadata = package.metadata.get("craftbook");
    let mut layer = metadata
        .and_then(|m| m.get("layer"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_owned();
    let mut context = metadata
        .and_then(|m| m.get("context"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_owned();
    let mut tags: Vec<String> = metadata
        .and_then(|m| m.get("tags"))
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str().map(str::to_owned))
        .collect();
    for selector in &policy.package {
        if selector.matches(&package.name) {
            if layer.is_empty() {
                if let Some(value) = &selector.layer {
                    layer = value.clone();
                }
            }
            if context.is_empty() {
                if let Some(value) = &selector.context {
                    context = value.clone();
                }
            }
            for tag in &selector.tags {
                if !tags.contains(tag) {
                    tags.push(tag.clone());
                }
            }
        }
    }
    serde_json::json!({"name": package.name, "layer": layer, "context": context, "tags": tags})
}

impl DependencyRule {
    fn matches(
        &self,
        source: &Package,
        target: &Package,
        policy: &crate::Policy,
    ) -> Result<bool, String> {
        if let Some(expression) = &self.when {
            let program = cel::Program::compile(expression)
                .map_err(|error| format!("invalid CEL expression: {error}"))?;
            let mut context = cel::Context::default();
            for (name, package) in [("from", source), ("to", target)] {
                let value = classified(package, policy);
                context
                    .add_variable(
                        name,
                        cel::to_value(&value)
                            .map_err(|error| format!("invalid package metadata: {error}"))?,
                    )
                    .map_err(|error| format!("CEL context: {error}"))?;
            }
            let result = program
                .execute(&context)
                .map_err(|error| format!("CEL evaluation failed: {error}"))?;
            return match result {
                cel::Value::Bool(value) => Ok(value),
                _ => Err("CEL condition must return a boolean".into()),
            };
        }
        Ok(self.from == source.name && self.disallow == target.name)
    }
}

/// Get the Cargo workspace root for a manifest, without resolving dependencies.
pub fn workspace_root(manifest: &Path) -> Result<String, String> {
    let output = Command::new("cargo")
        .args([
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
            "--manifest-path",
        ])
        .arg(manifest)
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).map_err(|error| error.to_string())?;
    value["workspace_root"]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| "missing Cargo workspace root".to_owned())
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

#[derive(Debug, Clone)]
pub struct DependencyViolation {
    pub rule_id: Option<String>,
    pub description: Option<String>,
    pub from: String,
    pub disallow: String,
    pub manifest: PathBuf,
    pub config: PathBuf,
    pub rule_index: usize,
    pub config_line: Option<usize>,
    pub manifest_line: Option<usize>,
    pub manifest_column: Option<usize>,
    pub config_column: Option<usize>,
    pub manifest_end: Option<(usize, usize)>,
    pub config_end: Option<(usize, usize)>,
}

type SourceCoordinates = ((usize, usize), (usize, usize));

fn dependency_position(manifest: &Path, package: &str) -> Option<SourceCoordinates> {
    let source = std::fs::read_to_string(manifest).ok()?;
    let span = crate::toml_locations::dependency_span_in(&source, package)?;
    span.positions(&source)
        .map(|(start, end)| ((start.line, start.column), (end.line, end.column)))
}

#[cfg(test)]
fn dependency_line(manifest: &Path, package: &str) -> Option<usize> {
    dependency_position(manifest, package).map(|(start, _)| start.0)
}

fn rule_source(
    directory: &Path,
    from: &str,
    disallow: &str,
    when: Option<&str>,
) -> Result<(PathBuf, usize, Option<SourceCoordinates>), String> {
    for path in crate::scopes::sources(directory).into_iter().rev() {
        let content = std::fs::read_to_string(&path)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        let table: toml::Table =
            toml::from_str(&content).map_err(|error| format!("{}: {error}", path.display()))?;
        if let Some(rules) = table.get("dependency_rule") {
            let list = rules
                .as_array()
                .ok_or_else(|| format!("{}: dependency_rule must be an array", path.display()))?;
            for (index, entry) in list.iter().enumerate() {
                if (when.is_some() && entry.get("when").and_then(toml::Value::as_str) == when)
                    || (when.is_none()
                        && entry.get("from").and_then(toml::Value::as_str) == Some(from)
                        && entry.get("disallow").and_then(toml::Value::as_str) == Some(disallow))
                {
                    let position = crate::toml_locations::rule_span(&content, index)
                        .and_then(|span| span.positions(&content))
                        .map(|(start, end)| ((start.line, start.column), (end.line, end.column)));
                    return Ok((path, index, position));
                }
            }
            break;
        }
    }
    Err(format!("unable to locate rule {from} -> {disallow}"))
}

/// Diagnose direct workspace violations with their actual source policy.
pub fn inspect_workspace_scoped(path: &Path) -> Result<Vec<DependencyViolation>, String> {
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1"])
        .arg("--manifest-path")
        .arg(path)
        .output()
        .map_err(|error| format!("cargo metadata: {error}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    let metadata: Metadata = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("invalid cargo metadata: {error}"))?;
    let members: HashSet<&str> = metadata
        .workspace_members
        .iter()
        .map(String::as_str)
        .collect();
    let packages: HashMap<&str, &Package> = metadata
        .packages
        .iter()
        .filter(|package| members.contains(package.id.as_str()))
        .map(|package| (package.id.as_str(), package))
        .collect();
    let mut violations = Vec::new();
    for node in metadata
        .resolve
        .ok_or("missing resolved dependencies")?
        .nodes
    {
        let Some(package) = packages.get(node.id.as_str()) else {
            continue;
        };
        let manifest = PathBuf::from(&package.manifest_path);
        let directory = manifest.parent().ok_or("invalid package manifest path")?;
        let Some(raw) = crate::scopes::effective(directory)? else {
            continue;
        };
        let policy: crate::Policy = raw
            .try_into()
            .map_err(|error| format!("{}: {error}", directory.display()))?;
        let failures = crate::validate(&policy);
        if !failures.is_empty() {
            return Err(format!("{}: {}", directory.display(), failures.join("; ")));
        }
        if policy.architecture.require_classification {
            let metadata = classified(package, &policy);
            let is_classified = {
                metadata
                    .get("layer")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|layer| !layer.is_empty())
                    && metadata
                        .get("context")
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|context| !context.is_empty())
            };
            if !is_classified {
                return Err(format!(
                    "workspace package {} lacks [package.metadata.craftbook] layer/context classification",
                    package.name
                ));
            }
        }
        for edge in node.deps {
            let Some(target) = packages.get(edge.pkg.as_str()) else {
                continue;
            };
            for rule in &policy.dependency_rule {
                if !rule.matches(package, target, &policy)? {
                    continue;
                }
                let (config, rule_index, config_position) =
                    rule_source(directory, &rule.from, &rule.disallow, rule.when.as_deref())?;
                let manifest_position = dependency_position(&manifest, &target.name);
                violations.push(DependencyViolation {
                    rule_id: rule.id.clone(),
                    description: rule.message.clone(),
                    from: package.name.clone(),
                    disallow: target.name.clone(),
                    manifest: manifest.clone(),
                    config,
                    rule_index,
                    config_line: config_position.map(|(start, _)| start.0),
                    config_column: config_position.map(|(start, _)| start.1),
                    config_end: config_position.map(|(_, end)| end),
                    manifest_line: manifest_position.map(|(start, _)| start.0),
                    manifest_column: manifest_position.map(|(start, _)| start.1),
                    manifest_end: manifest_position.map(|(_, end)| end),
                });
            }
        }
    }
    violations.sort_by(|a, b| {
        a.manifest
            .cmp(&b.manifest)
            .then(a.disallow.cmp(&b.disallow))
    });
    violations.dedup_by(|a, b| a.manifest == b.manifest && a.disallow == b.disallow);
    Ok(violations)
}

/// Validate each package with the effective policy of its own directory.
/// A nested CraftBook.toml can replace inherited dependency rules.
pub fn check_workspace_scoped(path: &Path) -> Result<Vec<String>, String> {
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1"])
        .arg("--manifest-path")
        .arg(path)
        .output()
        .map_err(|e| format!("cargo metadata: {e}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    let metadata: Metadata = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("invalid cargo metadata: {e}"))?;
    let members: HashSet<&str> = metadata
        .workspace_members
        .iter()
        .map(String::as_str)
        .collect();
    let packages: HashMap<&str, &Package> = metadata
        .packages
        .iter()
        .filter(|p| members.contains(p.id.as_str()))
        .map(|p| (p.id.as_str(), p))
        .collect();
    let mut findings = Vec::new();
    let nodes = metadata
        .resolve
        .ok_or("missing resolved dependencies")?
        .nodes;
    for node in nodes {
        let Some(package) = packages.get(node.id.as_str()) else {
            continue;
        };
        let Some(dir) = Path::new(&package.manifest_path).parent() else {
            return Err(format!("invalid manifest path for {}", package.name));
        };
        let Some(raw) = crate::scopes::effective(dir)? else {
            continue;
        };
        let policy: crate::Policy = raw
            .try_into()
            .map_err(|e| format!("{}: {e}", dir.display()))?;
        let errors = crate::validate(&policy);
        if !errors.is_empty() {
            return Err(format!("{}: {}", dir.display(), errors.join("; ")));
        }
        if policy.architecture.require_classification {
            let metadata = classified(package, &policy);
            let is_classified = {
                metadata
                    .get("layer")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|layer| !layer.is_empty())
                    && metadata
                        .get("context")
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|context| !context.is_empty())
            };
            if !is_classified {
                return Err(format!(
                    "workspace package {} lacks [package.metadata.craftbook] layer/context classification",
                    package.name
                ));
            }
        }
        for edge in node.deps {
            let Some(target) = packages.get(edge.pkg.as_str()) else {
                continue;
            };
            for rule in &policy.dependency_rule {
                if !rule.matches(package, target, &policy)? {
                    continue;
                }
                findings.push(format!(
                    "{}: {} must not depend on {}",
                    dir.display(),
                    package.name,
                    target.name
                ));
            }
        }
    }
    findings.sort();
    findings.dedup();
    Ok(findings)
}

/// Analyze only direct edges between members of the same workspace.
pub fn check_metadata(input: &str, rules: &[DependencyRule]) -> Result<Vec<String>, String> {
    let metadata: Metadata =
        serde_json::from_str(input).map_err(|err| format!("invalid cargo metadata: {err}"))?;
    let workspace: HashSet<&str> = metadata
        .workspace_members
        .iter()
        .map(String::as_str)
        .collect();
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
                if rule.when.is_none() && rule.from == source && rule.disallow == target {
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
    fn cel_rule_reads_cargo_package_metadata() {
        let policy: crate::Policy = crate::parse(
            r#"
version = 2
project = "test"
[[dependency_rule]]
id = "ARCH001"
when = 'from.layer == "domain" && to.layer == "infra"'
action = "deny"
message = "forbidden"
"#,
        )
        .unwrap();
        let packages: Vec<Package> = serde_json::from_value(serde_json::json!([
            {"id":"a", "name":"example-domain", "metadata":{"craftbook":{"layer":"domain","context":"example"}}},
            {"id":"b", "name":"example-infra", "metadata":{"craftbook":{"layer":"infra","context":"example"}}}
        ])).unwrap();
        assert!(
            policy.dependency_rule[0]
                .matches(&packages[0], &packages[1], &policy)
                .unwrap()
        );
        assert!(
            !policy.dependency_rule[0]
                .matches(&packages[1], &packages[0], &policy)
                .unwrap()
        );
    }

    #[test]
    fn cel_rules_use_package_glob_fallbacks() {
        let policy = crate::parse(
            r#"
version=2
project="fixture"
[[package]]
match="*-domain"
layer="domain"
context="example"
[[package]]
match="*-infra"
layer="infra"
context="example"
[[dependency_rule]]
when='from.layer == "domain" && to.layer == "infra"'
action="deny"
"#,
        )
        .unwrap();
        let packages: Vec<Package> = serde_json::from_value(serde_json::json!([
            {"id":"a","name":"example-domain"},
            {"id":"b","name":"example-infra"}
        ]))
        .unwrap();
        assert!(
            policy.dependency_rule[0]
                .matches(&packages[0], &packages[1], &policy)
                .unwrap()
        );
    }

    #[test]
    fn catches_forbidden_direct_workspace_edge() {
        let rules = vec![DependencyRule {
            from: "a".into(),
            disallow: "b".into(),
            id: None,
            when: None,
            action: None,
            message: None,
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
            id: None,
            when: None,
            action: None,
            message: None,
        }];
        assert!(check_metadata(METADATA, &rules).unwrap().is_empty());
    }

    #[test]
    fn requires_resolved_graph() {
        assert!(
            check_metadata(
                r#"{"packages":[],"workspace_members":[],"resolve":null}"#,
                &[]
            )
            .is_err()
        );
    }
    #[test]
    fn source_line_detection_handles_aliases_and_ambiguity() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "craftbook-dependency-lines-{}-{}.toml",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(
            &path,
            "[package]\nname='fixture'\nversion='0.1.0'\n[dependencies]\nrenamed = { package='actual', version='1' }\n",
        )
        .unwrap();
        assert_eq!(dependency_line(&path, "actual"), Some(5));
        assert_eq!(dependency_line(&path, "renamed"), None);
        std::fs::write(
            &path,
            "[dependencies]\nactual='1'\n[dev-dependencies]\nactual='1'\n",
        )
        .unwrap();
        assert_eq!(dependency_line(&path, "actual"), None);
        std::fs::write(
            &path,
            "[target.'cfg(unix)'.dependencies]\nrenamed = { package='actual', version='1' }\n",
        )
        .unwrap();
        assert_eq!(dependency_line(&path, "actual"), Some(2));
        std::fs::write(
            &path,
            "[dependencies]\nactual='1'\n[target.'cfg(unix)'.dependencies]\nactual='1'\n",
        )
        .unwrap();
        assert_eq!(dependency_line(&path, "actual"), None);
        std::fs::remove_file(path).unwrap();
    }
}
