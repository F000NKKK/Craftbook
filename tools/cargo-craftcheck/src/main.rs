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
                    && let Some(span) = cargo_craftcheck::source::SourceSpan::new(
                        range.start,
                        range.end,
                        &source,
                    )
                {
                    diagnostic = diagnostic.with_span(&source, &span);
                }
                findings.push(diagnostic);
                continue;
            }
        };
        match merged.try_into() {
            Ok(policy) => {
                let policy: cargo_craftcheck::Policy = policy;
                for message in cargo_craftcheck::validate(&policy) {
                    findings.push(Diagnostic::new("CB001", message, &path));
                }
            }
            Err(error) => findings.push(Diagnostic::new("CB001", format!("{error}"), &path)),
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
    let root_config = match config {
        Some(path) => path,
        None => cargo_craftcheck::scopes::nearest(&cwd)
            .ok_or("CraftBook.toml not found in the current directory or parents")?,
    };
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
            Some(path) => vec![path],
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
                            .with_rule_end(violation.config_end);
                        if let Some(line) = violation.manifest_line {
                            diagnostic = diagnostic.at(line, 1);
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
    let root_absolute = std::fs::canonicalize(root).map_err(|error| error.to_string())?;
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
