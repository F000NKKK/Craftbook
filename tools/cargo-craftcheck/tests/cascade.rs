//! End-to-end coverage of hierarchical policies with a real Cargo workspace.
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

#[test]
fn inherited_dependency_policy_can_be_replaced_locally() {
    let directory = std::env::temp_dir().join(format!(
        "craftbook-integration-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir_all(&directory).unwrap();
    write(
        &directory.join("CraftBook.toml"),
        "version = 2\nproject = 'fixture'\nprofiles = ['simple']\n[[dependency_rule]]\nfrom = 'alpha'\ndisallow = 'beta'\n",
    );
    write(
        &directory.join("Cargo.toml"),
        "[workspace]\nmembers = ['alpha', 'beta']\nresolver = '2'\n",
    );
    write(
        &directory.join("alpha/Cargo.toml"),
        "[package]\nname = 'alpha'\nversion = '0.1.0'\nedition = '2024'\n[dependencies]\nbeta = { path = '../beta' }\n",
    );
    write(
        &directory.join("alpha/src/lib.rs"),
        "pub fn value() -> u8 { beta::value() }\n",
    );
    write(
        &directory.join("beta/Cargo.toml"),
        "[package]\nname = 'beta'\nversion = '0.1.0'\nedition = '2024'\n",
    );
    write(
        &directory.join("beta/src/lib.rs"),
        "pub fn value() -> u8 { 1 }\n",
    );

    let executable = env!("CARGO_BIN_EXE_cargo-craftcheck");
    let run = |args: &[&str]| {
        Command::new(executable)
            .args(args)
            .current_dir(&directory)
            .output()
            .unwrap()
    };
    let first = run(&["deps"]);
    assert!(!first.status.success());
    let sarif = run(&["deps", "--format", "sarif"]);
    assert!(!sarif.status.success());
    let sarif_value: serde_json::Value = serde_json::from_slice(&sarif.stdout).unwrap();
    assert_eq!(sarif_value["runs"][0]["results"][0]["ruleId"], "CB002");
    assert!(String::from_utf8_lossy(&first.stderr).contains("alpha must not depend on beta"));

    // Replacing arrays at the child scope removes the inherited restriction.
    write(
        &directory.join("alpha/CraftBook.toml"),
        "dependency_rule = []\n",
    );
    let second = run(&["deps"]);
    assert!(
        second.status.success(),
        "{}",
        String::from_utf8_lossy(&second.stderr)
    );
    assert!(run(&["check"]).status.success());
    let json = run(&["check", "--format", "json"]);
    assert!(json.status.success());
    let json_value: serde_json::Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(json_value["success"], true);
    let config = run(&["config"]);
    assert!(config.status.success());
    let config_text = String::from_utf8_lossy(&config.stdout);
    assert!(config_text.contains("project = \"fixture\""));
    let explain = run(&["explain"]);
    assert!(explain.status.success());
    let explain_text = String::from_utf8_lossy(&explain.stdout);
    assert!(explain_text.contains("Configuration sources"));
    assert!(explain_text.contains("CraftBook.toml"));
    fs::remove_dir_all(&directory).unwrap();
}
