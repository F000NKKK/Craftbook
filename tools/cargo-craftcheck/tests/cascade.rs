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
    assert_eq!(
        sarif_value["runs"][0]["results"][0]["properties"]["ruleIndex"],
        0
    );
    assert_eq!(
        sarif_value["runs"][0]["results"][0]["locations"][0]["physicalLocation"]["region"]["startLine"],
        6
    );
    assert_eq!(
        sarif_value["runs"][0]["results"][0]["relatedLocations"][0]["physicalLocation"]["region"]["startLine"],
        4
    );
    assert!(
        sarif_value["runs"][0]["results"][0]["properties"]["configPath"]
            .as_str()
            .unwrap()
            .ends_with("CraftBook.toml")
    );
    assert!(sarif_value["runs"][0]["results"][0]["locations"][0]["physicalLocation"]
        ["artifactLocation"]["uri"]
        .as_str()
        .unwrap()
        .ends_with("alpha/Cargo.toml"));
    assert_eq!(
        sarif_value["runs"][0]["results"].as_array().unwrap().len(),
        1
    );
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

#[test]
fn independent_nested_workspace_is_checked_separately() {
    let directory = std::env::temp_dir().join(format!(
        "craftbook-multi-workspace-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir_all(&directory).unwrap();
    write(
        &directory.join("CraftBook.toml"),
        "version = 2\nproject = 'multi'\nprofiles = ['simple']\n",
    );
    write(
        &directory.join("Cargo.toml"),
        "[workspace]\nmembers = ['first']\nresolver = '2'\n",
    );
    write(
        &directory.join("first/Cargo.toml"),
        "[package]\nname = 'first'\nversion = '0.1.0'\nedition = '2024'\n",
    );
    write(&directory.join("first/src/lib.rs"), "pub fn first() {}\n");
    write(
        &directory.join("isolated/Cargo.toml"),
        "[workspace]\nmembers = ['second']\nresolver = '2'\n",
    );
    write(
        &directory.join("isolated/second/Cargo.toml"),
        "[package]\nname = 'second'\nversion = '0.1.0'\nedition = '2024'\n",
    );
    write(
        &directory.join("isolated/second/src/lib.rs"),
        "pub fn second() {}\n",
    );
    write(
        &directory.join("isolated/CraftBook.toml"),
        "profiles = ['not-a-valid-profile']\n",
    );
    let output = Command::new(env!("CARGO_BIN_EXE_cargo-craftcheck"))
        .args(["check", "--format", "json"])
        .current_dir(&directory)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["diagnostics"].as_array().unwrap().len(), 1);
    assert!(
        json["diagnostics"][0]["path"]
            .as_str()
            .unwrap()
            .replace('\\', "/").ends_with("isolated/CraftBook.toml")
    );
    fs::remove_dir_all(&directory).unwrap();
}

#[test]
fn invalid_nested_toml_produces_structured_diagnostic() {
    let directory = std::env::temp_dir().join(format!(
        "craftbook-invalid-toml-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir_all(&directory).unwrap();
    write(
        &directory.join("CraftBook.toml"),
        "version = 2\nproject = 'fixture'\nprofiles = ['simple']\n",
    );
    write(
        &directory.join("nested/CraftBook.toml"),
        "profiles = ['simple'\n",
    );
    let output = Command::new(env!("CARGO_BIN_EXE_cargo-craftcheck"))
        .args(["check", "--format", "json"])
        .current_dir(&directory)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["success"], false);
    assert_eq!(result["diagnostics"][0]["rule_id"], "CB001");
    assert_eq!(result["diagnostics"][0]["line"], 1);
    assert!(result["diagnostics"][0]["column"].as_u64().is_some());
    assert!(result["diagnostics"][0]["end_line"].as_u64().is_some());
    assert!(result["diagnostics"][0]["end_column"].as_u64().is_some());
    assert!(
        result["diagnostics"][0]["path"]
            .as_str()
            .unwrap()
            .replace('\\', "/").ends_with("nested/CraftBook.toml")
    );
    let sarif = Command::new(env!("CARGO_BIN_EXE_cargo-craftcheck"))
        .args(["check", "--format", "sarif"])
        .current_dir(&directory)
        .output()
        .unwrap();
    assert!(!sarif.status.success());
    let report: serde_json::Value = serde_json::from_slice(&sarif.stdout).unwrap();
    let region = &report["runs"][0]["results"][0]["locations"][0]["physicalLocation"]["region"];
    assert_eq!(region["startLine"], 1);
    assert!(region["startColumn"].as_u64().is_some());
    assert!(region["endLine"].as_u64().is_some());
    assert!(region["endColumn"].as_u64().is_some());
    fs::remove_dir_all(&directory).unwrap();
}

#[test]
fn reports_multiple_workspace_edges_with_aliases_and_target_sections() {
    let directory = std::env::temp_dir().join(format!(
        "craftbook-multiple-edges-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir_all(&directory).unwrap();
    write(
        &directory.join("CraftBook.toml"),
        "version = 2\nproject = 'fixture'\nprofiles = ['simple']\n[[dependency_rule]]\nfrom = 'app'\ndisallow = 'core'\n[[dependency_rule]]\nfrom = 'app'\ndisallow = 'utils'\n",
    );
    write(
        &directory.join("Cargo.toml"),
        "[workspace]\nmembers = ['app', 'core', 'utils']\nresolver = '2'\n",
    );
    write(
        &directory.join("app/Cargo.toml"),
        "[package]\nname = 'app'\nversion = '0.1.0'\nedition = '2024'\n[dependencies]\nrenamed_core = { package = 'core', path = '../core' }\n[target.'cfg(unix)'.dependencies]\nutils = { path = '../utils' }\n",
    );
    write(
        &directory.join("app/src/lib.rs"),
        "pub fn app() { let _ = renamed_core::core(); }\n",
    );
    for name in ["core", "utils"] {
        write(
            &directory.join(name).join("Cargo.toml"),
            &format!("[package]\nname = '{name}'\nversion = '0.1.0'\nedition = '2024'\n"),
        );
        write(
            &directory.join(name).join("src/lib.rs"),
            &format!("pub fn {name}() {{}}\n"),
        );
    }

    let executable = env!("CARGO_BIN_EXE_cargo-craftcheck");
    let output = Command::new(executable)
        .args(["deps", "--format", "json"])
        .current_dir(&directory)
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let diagnostics = json["diagnostics"].as_array().unwrap();
    assert_eq!(diagnostics.len(), 2);
    assert!(diagnostics.iter().all(|entry| entry["rule_id"] == "CB002"));
    assert_eq!(diagnostics[0]["line"], 6);
    assert_eq!(diagnostics[1]["line"], 8);
    assert_ne!(diagnostics[0]["rule_index"], diagnostics[1]["rule_index"]);
    assert!(
        diagnostics
            .iter()
            .all(|entry| entry["end_line"].as_u64().is_some())
    );
    assert!(
        diagnostics
            .iter()
            .all(|entry| entry["end_column"].as_u64().is_some())
    );
    assert!(
        diagnostics
            .iter()
            .all(|entry| entry["config_end_line"].as_u64().is_some())
    );

    let sarif = Command::new(executable)
        .args(["deps", "--format", "sarif"])
        .current_dir(&directory)
        .output()
        .unwrap();
    assert!(!sarif.status.success());
    let report: serde_json::Value = serde_json::from_slice(&sarif.stdout).unwrap();
    assert_eq!(report["runs"][0]["results"].as_array().unwrap().len(), 2);
    fs::remove_dir_all(&directory).unwrap();
}

#[test]
fn isolated_nested_workspace_dependency_violation_is_reported() {
    let directory = std::env::temp_dir().join(format!(
        "craftbook-nested-deps-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir_all(&directory).unwrap();
    write(
        &directory.join("CraftBook.toml"),
        "version = 2\nproject = 'fixture'\nprofiles = ['simple']\n",
    );
    write(
        &directory.join("Cargo.toml"),
        "[workspace]\nmembers = ['outer']\nresolver = '2'\n",
    );
    write(
        &directory.join("outer/Cargo.toml"),
        "[package]\nname = 'outer'\nversion = '0.1.0'\nedition = '2024'\n",
    );
    write(&directory.join("outer/src/lib.rs"), "pub fn outer() {}\n");
    write(
        &directory.join("nested/CraftBook.toml"),
        "[[dependency_rule]]\nfrom = 'inner'\ndisallow = 'leaf'\n",
    );
    write(
        &directory.join("nested/Cargo.toml"),
        "[workspace]\nmembers = ['inner', 'leaf']\nresolver = '2'\n",
    );
    write(
        &directory.join("nested/inner/Cargo.toml"),
        "[package]\nname = 'inner'\nversion = '0.1.0'\nedition = '2024'\n[dependencies]\nleaf = { path = '../leaf' }\n",
    );
    write(
        &directory.join("nested/inner/src/lib.rs"),
        "pub fn inner() { leaf::leaf(); }\n",
    );
    write(
        &directory.join("nested/leaf/Cargo.toml"),
        "[package]\nname = 'leaf'\nversion = '0.1.0'\nedition = '2024'\n",
    );
    write(
        &directory.join("nested/leaf/src/lib.rs"),
        "pub fn leaf() {}\n",
    );
    let output = Command::new(env!("CARGO_BIN_EXE_cargo-craftcheck"))
        .args(["deps", "--format", "json"])
        .current_dir(&directory)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let diagnostics = report["diagnostics"].as_array().unwrap();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0]["rule_id"], "CB002");
    assert!(
        diagnostics[0]["config_path"]
            .as_str()
            .unwrap()
            .replace('\\', "/").ends_with("nested/CraftBook.toml")
    );
    fs::remove_dir_all(&directory).unwrap();
}

#[test]
fn semantic_errors_point_to_original_policy_fields() {
    let directory = std::env::temp_dir().join(format!(
        "craftbook-semantic-ranges-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir_all(&directory).unwrap();
    write(
        &directory.join("CraftBook.toml"),
        "version = 1\nproject = '  '\nprofiles = ['simple']\n",
    );
    write(
        &directory.join("nested/CraftBook.toml"),
        "profiles = ['unknown', 'ddd', 'ddd']\n[[dependency_rule]]\nfrom = ''\ndisallow = 'api'\n",
    );
    let output = Command::new(env!("CARGO_BIN_EXE_cargo-craftcheck"))
        .args(["check", "--format", "json"])
        .current_dir(&directory)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let diagnostics = report["diagnostics"].as_array().unwrap();
    assert!(
        diagnostics
            .iter()
            .any(|d| d["message"] == "version must be 2"
                && d["path"].as_str().unwrap().ends_with("CraftBook.toml")
                && d["line"] == 1
                && d["column"].as_u64().is_some()
                && d["end_column"].as_u64().is_some())
    );
    assert!(
        diagnostics
            .iter()
            .any(|d| d["message"] == "project must be a nonempty string"
                && d["line"] == 2
                && d["end_line"].as_u64().is_some())
    );
    assert!(diagnostics.iter().any(|d| {
        d["message"] == "unknown profile: unknown"
            && d["path"]
                .as_str()
                .unwrap()
                .replace('\\', "/").ends_with("nested/CraftBook.toml")
            && d["line"] == 1
            && d["column"].as_u64().is_some()
    }));
    assert!(
        diagnostics
            .iter()
            .any(|d| d["message"] == "duplicate profile: ddd"
                && d["line"] == 1
                && d["column"].as_u64().is_some())
    );
    assert!(diagnostics.iter().any(|d| d["message"]
        == "dependency rules require nonempty from and disallow"
        && d["line"] == 3
        && d["end_line"].as_u64().is_some()));
    fs::remove_dir_all(&directory).unwrap();
}

#[test]
fn invalid_policy_structure_has_exact_json_and_sarif_ranges() {
    let directory = std::env::temp_dir().join(format!(
        "craftbook-structure-ranges-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir_all(&directory).unwrap();
    write(
        &directory.join("CraftBook.toml"),
        "version = 'bad'\r\nproject = 'app'\r\n[assurance]\r\nclippy = 'yes'\r\n",
    );
    let exe = env!("CARGO_BIN_EXE_cargo-craftcheck");
    for format in ["json", "sarif"] {
        let output = Command::new(exe)
            .args(["check", "--format", format])
            .current_dir(&directory)
            .output()
            .unwrap();
        assert!(!output.status.success());
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        if format == "json" {
            let diagnostics = report["diagnostics"].as_array().unwrap();
            assert!(diagnostics.iter().any(|d| {
                d["message"] == "version must be integer"
                    && d["line"] == 1
                    && d["column"] == 11
                    && d["end_column"] == 16
            }));
            assert!(diagnostics.iter().any(|d| {
                d["message"] == "assurance.clippy must be boolean"
                    && d["line"] == 4
                    && d["column"] == 10
            }));
        } else {
            let results = report["runs"][0]["results"].as_array().unwrap();
            assert!(results.iter().any(|d| {
                d["message"]["text"] == "version must be integer"
                    && d["locations"][0]["physicalLocation"]["region"]["startLine"] == 1
                    && d["locations"][0]["physicalLocation"]["region"]["endColumn"] == 16
            }));
        }
    }
    fs::remove_dir_all(&directory).unwrap();
}

#[test]
fn relative_config_path_matches_absolute_path() {
    let directory = std::env::temp_dir().join(format!(
        "craftbook-relative-paths-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&directory);
    write(
        &directory.join("CraftBook.toml"),
        "version = 2\nproject = 'fixture'\n",
    );
    let executable = env!("CARGO_BIN_EXE_cargo-craftcheck");
    let relative = Command::new(executable)
        .args(["check", "--config", "CraftBook.toml", "--format", "json"])
        .current_dir(&directory)
        .output()
        .unwrap();
    let absolute = Command::new(executable)
        .args([
            "check",
            "--config",
            directory.join("CraftBook.toml").to_str().unwrap(),
            "--format",
            "json",
        ])
        .current_dir(&directory)
        .output()
        .unwrap();
    assert!(
        relative.status.success(),
        "{}",
        String::from_utf8_lossy(&relative.stderr)
    );
    assert_eq!(relative.stdout, absolute.stdout);
    fs::remove_dir_all(&directory).unwrap();
}

#[test]
fn diagnostics_are_ordered_deterministically() {
    let directory = std::env::temp_dir().join(format!(
        "craftbook-ordering-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&directory);
    write(
        &directory.join("CraftBook.toml"),
        "version = 1\nproject = ''\nprofiles = ['unknown']\n",
    );
    let executable = env!("CARGO_BIN_EXE_cargo-craftcheck");
    let run = || {
        Command::new(executable)
            .args(["check", "--format", "json"])
            .current_dir(&directory)
            .output()
            .unwrap()
    };
    let first = run();
    let second = run();
    assert!(!first.status.success());
    assert_eq!(first.stdout, second.stdout);
    let report: serde_json::Value = serde_json::from_slice(&first.stdout).unwrap();
    assert_eq!(report["diagnostics"].as_array().unwrap().len(), 3);
    fs::remove_dir_all(&directory).unwrap();
}

#[test]
fn missing_required_field_is_not_hidden_by_unrelated_type_error() {
    let directory = std::env::temp_dir().join(format!(
        "craftbook-missing-key-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&directory);
    write(&directory.join("CraftBook.toml"), "version = 'invalid'\n");
    let output = Command::new(env!("CARGO_BIN_EXE_cargo-craftcheck"))
        .args(["check", "--format", "json"])
        .current_dir(&directory)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let diagnostics = report["diagnostics"].as_array().unwrap();
    assert!(
        diagnostics
            .iter()
            .any(|d| d["message"] == "version must be integer")
    );
    assert!(diagnostics.iter().any(|d| {
        d["message"]
            .as_str()
            .unwrap_or("")
            .contains("missing field")
    }));
    fs::remove_dir_all(&directory).unwrap();
}
