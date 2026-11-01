//! Cargo subcommand entrypoint: cargo craftcheck [check] [--config PATH].

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

fn usage() {
    println!("Usage: cargo craftcheck [check|deps] [--config PATH] [--manifest-path PATH]");
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1).peekable();
    // Cargo passes the subcommand name when invoking cargo-craftcheck.
    if args.peek().is_some_and(|arg| arg == "craftcheck") {
        args.next();
    }
    let command = match args.peek().map(String::as_str) {
        Some("check") => { args.next(); "check" }
        Some("deps") => { args.next(); "deps" }
        _ => "check",
    };

    let mut config = PathBuf::from("craftbook.toml");
    let mut manifest = PathBuf::from("Cargo.toml");
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--config" => {
                config = PathBuf::from(args.next().ok_or("--config requires a path")?);
            }
            "--manifest-path" => {
                manifest = PathBuf::from(args.next().ok_or("--manifest-path requires a path")?);
            }
            "-h" | "--help" => {
                usage();
                return Ok(());
            }
            other => return Err(format!("unexpected argument: {other}")),
        }
    }

    if command == "deps" {
        let text = std::fs::read_to_string(&config).map_err(|e| e.to_string())?;
        let policy = cargo_craftcheck::parse(&text)?;
        let violations = cargo_craftcheck::validate(&policy);
        if !violations.is_empty() {
            return Err(violations.join("\n"));
        }
        let violations = cargo_craftcheck::deps::check_workspace(&manifest, &policy.dependency_rule)?;
        if !violations.is_empty() {
            return Err(violations.join("\n"));
        }
        println!("Workspace dependency rules passed");
        return Ok(());
    }
    match cargo_craftcheck::check_file(&config) {
        Ok(()) => {
            println!("Craftbook v2 policy valid: {}", config.display());
            Ok(())
        }
        Err(errors) => Err(errors.join("\n")),
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
