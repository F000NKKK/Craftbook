//! Cargo subcommand entrypoint: cargo craftcheck [check] [--config PATH].

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

fn usage() {
    println!("Usage: cargo craftcheck [check] [--config PATH]");
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1).peekable();
    // Cargo passes the subcommand name when invoking cargo-craftcheck.
    if args.peek().is_some_and(|arg| arg == "craftcheck") {
        args.next();
    }
    if args.peek().is_some_and(|arg| arg == "check") {
        args.next();
    }

    let mut config = PathBuf::from("craftbook.toml");
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--config" => {
                config = PathBuf::from(args.next().ok_or("--config requires a path")?);
            }
            "-h" | "--help" => {
                usage();
                return Ok(());
            }
            other => return Err(format!("unexpected argument: {other}")),
        }
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
