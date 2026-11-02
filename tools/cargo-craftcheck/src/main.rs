//! Cargo subcommand entry point: cargo craftcheck [check|deps].

use std::collections::HashSet;
use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use cargo_craftcheck::report::{Diagnostic, Format, render_with_root};

fn usage() {
    println!(
        "Usage: cargo craftcheck [check|deps|config|explain] [--config CraftBook.toml] [--manifest-path Cargo.toml] [--format text|json|sarif]"
    );
}

fn semantic_diagnostics(policy: &cargo_craftcheck::Policy, directory: &Path) -> Vec<Diagnostic> {
    let mut findings = Vec::new();
    let sources = cargo_craftcheck::scopes::sources(directory);
    let mut profile_seen = std::collections::HashSet::new();
    let mut issues: Vec<(&str, Option<usize>, String)> = Vec::new();
    if policy.version != 2 {
        issues.push(("version", None, "version must be 2".to_owned()));
    }
    if policy.project.trim().is_empty() {
        issues.push((
            "project",
            None,
            "project must be a nonempty string".to_owned(),
        ));
    }
    for (index, profile) in policy.profiles.iter().enumerate() {
        if !cargo_craftcheck::supported_profile(profile) {
            issues.push((
                "profiles",
                Some(index),
                format!("unknown profile: {profile}"),
            ));
        }
        if !profile_seen.insert(profile) {
            issues.push((
                "profiles",
                Some(index),
                format!("duplicate profile: {profile}"),
            ));
        }
    }
    for (index, rule) in policy.dependency_rule.iter().enumerate() {
        if rule.from.trim().is_empty() || rule.disallow.trim().is_empty() {
            issues.push((
                "dependency_rule",
                Some(index),
                "dependency rules require nonempty from and disallow".to_owned(),
            ));
        }
    }
    for (field, index, message) in issues {
        let mut diagnostic =
            Diagnostic::new("CB001", message, directory.join("CraftBook.toml").as_path());
        for path in sources.iter().rev() {
            let Ok(source) = std::fs::read_to_string(path) else {
                continue;
            };
            let Ok(table) = toml::from_str::<toml::Table>(&source) else {
                continue;
            };
            if !table.contains_key(field) {
                continue;
            }
            diagnostic.path = path.display().to_string();
            let location = if field == "dependency_rule" {
                index.and_then(|index| {
                    cargo_craftcheck::toml_locations::invalid_rule_field_span(&source, index)
                })
            } else {
                cargo_craftcheck::toml_locations::policy_span(&source, field, index)
            };
            if let Some(span) = location {
                diagnostic = diagnostic.with_span(&source, &span);
            }
            break;
        }
        findings.push(diagnostic);
    }
    findings
}

fn check_policy_tree(root: &Path, verbose: bool) -> Result<Vec<Diagnostic>, String> {
    let mut findings = Vec::new();
    let configs = cargo_craftcheck::scopes::discover(root, "CraftBook.toml")?;
    for path in configs {
        let dir = path.parent().ok_or("invalid configuration path")?;
        let merged = match cargo_craftcheck::scopes::effective(dir) {
            Ok(Some(merged)) => merged,
            Ok(None) => {
                findings.push(Diagnostic::new("CB001", "missing effective policy", &path));
                continue;
            }
            Err(error) => {
                let mut diagnostic = Diagnostic::new("CB001", error, &path);
                if let Ok(source) = std::fs::read_to_string(&path)
                    && let Err(parse_error) = toml::from_str::<toml::Table>(&source)
                    && let Some(range) = parse_error.span()
                    && let Some(span) =
                        cargo_craftcheck::source::SourceSpan::new(range.start, range.end, &source)
                {
                    diagnostic = diagnostic.with_span(&source, &span);
                }
                findings.push(diagnostic);
                continue;
            }
        };
        // Validate each source document once, with its own exact coordinates.
        if let Ok(source) = std::fs::read_to_string(&path) {
            findings.extend(cargo_craftcheck::structure::inspect(&path, &source));
        }
        match merged.try_into() {
            Ok(policy) => {
                let policy: cargo_craftcheck::Policy = policy;
                findings.extend(semantic_diagnostics(&policy, dir));
            }
            Err(error) => {
                // Structural validation already points to invalid declarations in
                // the original file; keep the fallback for missing inherited keys.
                if !findings
                    .iter()
                    .any(|item| item.rule_id == "CB001" && item.path == path.display().to_string())
                {
                    findings.push(Diagnostic::new("CB001", format!("{error}"), &path));
                }
            }
        }
        if verbose {
            println!("Policy checked: {}", path.display());
        }
    }
    Ok(findings)
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1).peekable();
    // Cargo invokes cargo-craftcheck with an initial "craftcheck" argument.
    if args.peek().is_some_and(|arg| arg == "craftcheck") {
        args.next();
    }
    let command = match args.peek().map(String::as_str) {
        Some("check") => {
            args.next();
            "check"
        }
        Some("deps") => {
            args.next();
            "deps"
        }
        Some("config") => {
            args.next();
            "config"
        }
        Some("explain") => {
            args.next();
            "explain"
        }
        _ => "check",
    };

    let mut config: Option<PathBuf> = None;
    let mut manifest: Option<PathBuf> = None;
    let mut format = Format::Text;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--config" => {
                config = Some(PathBuf::from(
                    args.next().ok_or("--config requires a path")?,
                ));
            }
            "--format" => {
                let value = args.next().ok_or("--format requires a value")?;
                format = Format::parse(&value)?;
            }
            "--manifest-path" => {
                manifest = Some(PathBuf::from(
                    args.next().ok_or("--manifest-path requires a path")?,
                ));
            }
            "-h" | "--help" => {
                usage();
                return Ok(());
            }
            other => return Err(format!("unexpected argument: {other}")),
        }
    }

    let cwd = env::current_dir().map_err(|e| e.to_string())?;
    let selected = match config {
        Some(path) => path,
        None => cargo_craftcheck::scopes::nearest(&cwd)
            .ok_or("CraftBook.toml not found in the current directory or parents")?,
    };
    let root_config = std::fs::canonicalize(&selected)
        .map_err(|error| format!("{}: {error}", selected.display()))?;
    if root_config
        .file_name()
        .is_none_or(|name| name != "CraftBook.toml")
    {
        return Err("--config must point to a CraftBook.toml file".into());
    }
    let root = root_config.parent().ok_or("invalid --config path")?;
    if (command == "config" || command == "explain") && format != Format::Text {
        return Err("--format is supported only for check and deps".into());
    }
    if command == "config" || command == "explain" {
        let target = if cwd.starts_with(root) {
            cwd.as_path()
        } else {
            root
        };
        let merged =
            cargo_craftcheck::scopes::effective(target)?.ok_or("no CraftBook.toml found")?;
        let policy: cargo_craftcheck::Policy = merged
            .clone()
            .try_into()
            .map_err(|e| format!("invalid effective policy: {e}"))?;
        let findings = cargo_craftcheck::validate(&policy);
        if !findings.is_empty() {
            return Err(findings.join("\n"));
        }
        if command == "explain" {
            println!("Configuration sources (global to local):");
            for path in cargo_craftcheck::scopes::sources(target) {
                println!("  {}", path.display());
            }
            println!("Nested tables merge; scalars and arrays replace inherited values.");
        }
        println!("{merged}");
        return Ok(());
    }
    let mut diagnostics = check_policy_tree(root, format == Format::Text)?;

    if command == "deps" {
        let manifests = match manifest {
            Some(path) => vec![
                std::fs::canonicalize(&path)
                    .map_err(|error| format!("{}: {error}", path.display()))?,
            ],
            None => cargo_craftcheck::scopes::discover(root, "Cargo.toml")?,
        };
        if manifests.is_empty() {
            return Err(format!("no Cargo.toml found below {}", root.display()));
        }
        let mut seen_workspaces = HashSet::new();
        for path in manifests {
            let workspace_root = match cargo_craftcheck::deps::workspace_root(&path) {
                Ok(root) => root,
                Err(error) => {
                    diagnostics.push(Diagnostic::new("CB003", error, &path));
                    continue;
                }
            };
            if !seen_workspaces.insert(workspace_root) {
                continue;
            }
            match cargo_craftcheck::deps::inspect_workspace_scoped(&path) {
                Ok(violations) => {
                    for violation in violations {
                        let message = format!(
                            "{} must not depend on {} (rule #{} in {})",
                            violation.from,
                            violation.disallow,
                            violation.rule_index + 1,
                            violation.config.display()
                        );
                        let mut diagnostic = Diagnostic::new("CB002", message, &violation.manifest)
                            .with_rule(&violation.config, violation.rule_index)
                            .with_rule_line(violation.config_line)
                            .with_rule_column(violation.config_column)
                            .with_rule_end(violation.config_end);
                        if let Some(line) = violation.manifest_line {
                            diagnostic =
                                diagnostic.at(line, violation.manifest_column.unwrap_or(1));
                        }
                        if let Some((line, column)) = violation.manifest_end {
                            diagnostic = diagnostic.with_end(line, column);
                        }
                        diagnostics.push(diagnostic);
                    }
                }
                Err(error) => diagnostics.push(Diagnostic::new("CB003", error, &path)),
            }
        }
    }
    diagnostics.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then(a.line.cmp(&b.line))
            .then(a.column.cmp(&b.column))
            .then(a.rule_id.cmp(&b.rule_id))
            .then(a.message.cmp(&b.message))
    });
    diagnostics.dedup_by(|a, b| {
        a.path == b.path
            && a.line == b.line
            && a.column == b.column
            && a.rule_id == b.rule_id
            && a.message == b.message
    });
    let root_absolute = root.to_path_buf();
    let output = render_with_root(format, &diagnostics, &root_absolute)?;
    if format == Format::Text {
        if diagnostics.is_empty() {
            println!("Craftbook checks passed");
        } else {
            eprintln!("{output}");
        }
    } else {
        println!("{output}");
    }
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(format!("{} violation(s)", diagnostics.len()))
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("craftcheck: {error}");
            ExitCode::FAILURE
        }
    }
}
