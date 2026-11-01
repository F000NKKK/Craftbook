//! Discover and merge directory-scoped CraftBook.toml policies.
use std::fs;
use std::path::{Path, PathBuf};

const SKIP: &[&str] = &[".git", "target", "node_modules", ".idea", ".venv"];

/// Find the closest configuration in the current directory or an ancestor.
pub fn nearest(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .map(|dir| dir.join("CraftBook.toml"))
        .find(|p| p.is_file())
}

/// Merge tables recursively; the child wins for scalars and replaces arrays.
pub fn merge(parent: &mut toml::Value, child: toml::Value) {
    match (parent, child) {
        (toml::Value::Table(base), toml::Value::Table(next)) => {
            for (key, value) in next {
                if let Some(existing) = base.get_mut(&key) {
                    merge(existing, value);
                } else {
                    base.insert(key, value);
                }
            }
        }
        (base, next) => *base = next,
    }
}

/// Resolve all policies from filesystem root to the requested directory.
pub fn effective(dir: &Path) -> Result<Option<toml::Value>, String> {
    let mut chain: Vec<_> = dir.ancestors().collect();
    chain.reverse();
    let mut merged: Option<toml::Value> = None;
    for directory in chain {
        let path = directory.join("CraftBook.toml");
        if !path.is_file() {
            continue;
        }
        let raw = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let value = raw
            .parse::<toml::Value>()
            .map_err(|e| format!("{}: {e}", path.display()))?;
        if !value.is_table() {
            return Err(format!("{}: expected a TOML table", path.display()));
        }
        match &mut merged {
            Some(base) => merge(base, value),
            None => merged = Some(value),
        }
    }
    Ok(merged)
}

/// Walk a directory tree without following directory symlinks.
pub fn discover(root: &Path, filename: &str) -> Result<Vec<PathBuf>, String> {
    let mut found = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        let entries = fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("{}: {e}", dir.display()))?;
            let kind = entry.file_type().map_err(|e| e.to_string())?;
            let path = entry.path();
            if kind.is_dir() {
                if !SKIP.contains(&entry.file_name().to_string_lossy().as_ref()) {
                    pending.push(path);
                }
            } else if kind.is_file() && entry.file_name() == filename {
                found.push(path);
            }
        }
    }
    found.sort();
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_table_overrides_one_key_without_erasing_siblings() {
        let mut base: toml::Value = "[assurance]\ntest=true\nclippy=true".parse().unwrap();
        let child = "[assurance]\nclippy=false".parse().unwrap();
        merge(&mut base, child);
        assert_eq!(base["assurance"]["test"].as_bool(), Some(true));
        assert_eq!(base["assurance"]["clippy"].as_bool(), Some(false));
    }

    #[test]
    fn nested_configuration_inherits_parent_and_overrides_selected_fields() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let sequence = SERIAL.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "craftbook-scopes-{}-{sequence}",
            std::process::id()
        ));
        let nested = root.join("subsystem");
        fs::create_dir_all(&nested).unwrap();
        fs::write(
            root.join("CraftBook.toml"),
            "version=2\nproject='root'\nprofiles=['simple']\n[assurance]\nformat=true\ntest=true",
        )
        .unwrap();
        fs::write(
            nested.join("CraftBook.toml"),
            "profiles=['systems']\n[assurance]\ntest=false",
        )
        .unwrap();

        let merged = effective(&nested).unwrap().unwrap();
        assert_eq!(merged["project"].as_str(), Some("root"));
        assert_eq!(merged["profiles"][0].as_str(), Some("systems"));
        assert_eq!(merged["assurance"]["format"].as_bool(), Some(true));
        assert_eq!(merged["assurance"]["test"].as_bool(), Some(false));
        assert_eq!(discover(&root, "CraftBook.toml").unwrap().len(), 2);

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn arrays_replace_parent_arrays() {
        let mut base: toml::Value = "profiles=['ddd']".parse().unwrap();
        merge(&mut base, "profiles=['systems']".parse().unwrap());
        assert_eq!(base["profiles"][0].as_str(), Some("systems"));
    }
}
