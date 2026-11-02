//! Craftbook v2 policy model and validation.
//! Rust code semantics are checked by rustc and Clippy, not by textual heuristics.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use serde::Deserialize;

pub mod deps;
pub mod report;
pub mod scopes;
pub mod source;
pub mod structure;
pub mod toml_locations;

const PROFILES: &[&str] = &[
    "simple",
    "layered",
    "ddd",
    "hexagonal",
    "clean",
    "cqrs",
    "event-sourcing",
    "actor",
    "systems",
    "embedded",
    "performance",
    "functional-core",
    "plugin",
];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub version: u32,
    pub project: String,
    #[serde(default)]
    pub profiles: Vec<String>,
    #[serde(default)]
    pub assurance: Assurance,
    #[serde(default)]
    pub constraints: Constraints,
    #[serde(default)]
    pub dependency_rule: Vec<deps::DependencyRule>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Assurance {
    #[serde(default)]
    pub format: bool,
    #[serde(default)]
    pub clippy: bool,
    #[serde(default)]
    pub test: bool,
    #[serde(default)]
    pub docs: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Constraints {
    #[serde(default)]
    pub no_std: bool,
    #[serde(default)]
    pub public_api_stable: bool,
    #[serde(default)]
    pub unsafe_review_required: bool,
}

/// Whether a named architecture profile is supported.
pub fn supported_profile(profile: &str) -> bool {
    PROFILES.contains(&profile)
}

/// Parse configuration, reporting TOML syntax and shape errors.
pub fn parse(input: &str) -> Result<Policy, String> {
    toml::from_str(input).map_err(|error| format!("invalid policy TOML: {error}"))
}

/// Validate policy semantics without requiring a specific architecture.
pub fn validate(policy: &Policy) -> Vec<String> {
    let mut errors = Vec::new();
    if policy.version != 2 {
        errors.push("version must be 2".to_owned());
    }
    if policy.project.trim().is_empty() {
        errors.push("project must be a nonempty string".to_owned());
    }

    let mut seen = HashSet::new();
    for profile in &policy.profiles {
        if !supported_profile(profile) {
            errors.push(format!("unknown profile: {profile}"));
        }
        if !seen.insert(profile) {
            errors.push(format!("duplicate profile: {profile}"));
        }
    }
    for rule in &policy.dependency_rule {
        if let Some(condition) = &rule.when {
            if let Err(error) = cel::Program::compile(condition) {
                errors.push(format!("invalid CEL dependency condition: {error}"));
            }
            if rule.action.as_deref().unwrap_or("deny") != "deny" {
                errors.push("conditional dependency rules currently require action = deny".to_owned());
            }
        } else if rule.from.trim().is_empty() || rule.disallow.trim().is_empty() {
            errors.push("dependency rules require from/disallow or a CEL when condition".to_owned());
        }
    }
    errors
}

/// Read and validate a policy from disk.
pub fn check_file(path: &Path) -> Result<(), Vec<String>> {
    let text = fs::read_to_string(path)
        .map_err(|error| vec![format!("cannot read {}: {error}", path.display())])?;
    let policy = parse(&text).map_err(|error| vec![error])?;
    let findings = validate(&policy);
    if findings.is_empty() {
        Ok(())
    } else {
        Err(findings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = r#"
version = 2
project = "application"
profiles = ["systems", "performance"]
[assurance]
format = true
clippy = true
[constraints]
no_std = true
"#;

    #[test]
    fn accepts_valid_policy() {
        let policy = parse(VALID).unwrap();
        assert!(validate(&policy).is_empty());
    }

    #[test]
    fn rejects_unknown_profile() {
        let policy = parse("version=2\nproject='x'\nprofiles=['unexpected']").unwrap();
        assert!(
            validate(&policy)
                .iter()
                .any(|e| e.contains("unknown profile"))
        );
    }

    #[test]
    fn rejects_duplicates() {
        let policy = parse("version=2\nproject='x'\nprofiles=['ddd','ddd']").unwrap();
        assert!(validate(&policy).iter().any(|e| e.contains("duplicate")));
    }

    #[test]
    fn rejects_unknown_settings() {
        assert!(parse("version=2\nproject='x'\n[assurance]\nmagic=true").is_err());
    }

    #[test]
    fn rejects_wrong_types() {
        assert!(parse("version=2\nproject='x'\n[assurance]\nclippy='yes'").is_err());
    }

    #[test]
    fn rejects_invalid_version_and_project() {
        let policy = parse("version=1\nproject='  '").unwrap();
        assert_eq!(validate(&policy).len(), 2);
    }
}
