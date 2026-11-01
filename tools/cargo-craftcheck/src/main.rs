//! Cargo subcommand entry point: cargo craftcheck [check|deps].

use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn usage() {
    println!(
        "Usage: cargo craftcheck [check|deps|config|explain] [--config CraftBook.toml] [--manifest-path Cargo.toml]"
    );
}

fn check_policy_tree(root: &Path) -> Result<(), String> {
    let configs = cargo_craftcheck::scopes::discover(root, "CraftBook.toml")?;
    for path in configs {
        let dir = path.parent().ok_or("invalid configuration path")?;
        let merged = cargo_craftcheck::scopes::effective(dir)?
            .ok_or_else(|| format!("no effective policy for {}", path.display()))?;
        let policy: cargo_craftcheck::Policy = merged
            .try_into()
            .map_err(|e| format!("{}: {e}", path.display()))?;
        let findings = cargo_craftcheck::validate(&policy);
        if !findings.is_empty() {
            return Err(format!("{}: {}", path.display(), findings.join("; ")));
        }
        println!("Policy valid: {}", path.display());
    }
    Ok(())
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
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--config" => {
                config = Some(PathBuf::from(
                    args.next().ok_or("--config requires a path")?,
                ));
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
    if command == "config" || command == "explain" {
        let target = if cwd.starts_with(root) { cwd.as_path() } else { root };
        let merged = cargo_craftcheck::scopes::effective(target)?
            .ok_or("no CraftBook.toml found")?;
        let policy: cargo_craftcheck::Policy = merged.clone().try_into()
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
    check_policy_tree(root)?;

    if command == "deps" {
        let manifests = match manifest {
            Some(path) => vec![path],
            None => cargo_craftcheck::scopes::discover(root, "Cargo.toml")?,
        };
        if manifests.is_empty() {
            return Err(format!("no Cargo.toml found below {}", root.display()));
        }
        let mut findings = Vec::new();
        for path in manifests {
            let errors = cargo_craftcheck::deps::check_workspace_scoped(&path)
                .map_err(|e| format!("{}: {e}", path.display()))?;
            findings.extend(errors);
        }
        findings.sort();
        findings.dedup();
        if !findings.is_empty() {
            return Err(findings.join("\n"));
        }
        println!("Workspace dependency rules passed");
    }
    Ok(())
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
