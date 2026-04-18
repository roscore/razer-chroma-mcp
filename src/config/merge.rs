//! Deep-merge our `razer-chroma` entry into Claude Desktop / Claude Code config.
//!
//! The algorithm is spec §7.6; every numbered step in the spec maps to a test
//! in the test module below. Writes are atomic (tmp + rename), corrupt files
//! are quarantined with a timestamped `.corrupt-<ts>` sibling, and the last 3
//! backups are retained as `.bak-<ts>` siblings.

use crate::error::{ChromaError, Result};
use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const OUR_KEY: &str = "razer-chroma";

pub struct MergeInput<'a> {
    pub path: &'a Path,
    pub binary_path: &'a Path,
    pub extra_args: &'a [&'a str],
}

#[derive(Debug, PartialEq, Eq)]
pub enum MergeOutcome {
    /// File did not exist; we created it.
    Created,
    /// Our entry was already byte-equivalent; no write.
    Idempotent,
    /// Merged, keeping other keys intact.
    Merged,
    /// Overwrote a previous `razer-chroma` entry differing from ours.
    Replaced,
}

pub fn merge(input: &MergeInput) -> Result<MergeOutcome> {
    let our_entry = our_entry(input.binary_path, input.extra_args);

    // 1. Read (or treat missing / empty as `{}`).
    let (mut root, existed) = match fs::read_to_string(input.path) {
        Ok(s) if s.trim().is_empty() => (Value::Object(Map::new()), true),
        Ok(s) => {
            let v = parse_strict_or_quarantine(input.path, &s)?;
            (v, true)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (Value::Object(Map::new()), false),
        Err(e) => {
            return Err(ChromaError::SessionInitFailed(format!(
                "read {}: {e}",
                input.path.display()
            )));
        }
    };

    // 3. Validate root is an object.
    let Some(obj) = root.as_object_mut() else {
        return Err(ChromaError::ConfigCorrupt {
            path: input.path.display().to_string(),
            quarantine: quarantine_copy(input.path)?.display().to_string(),
        });
    };

    // 4. Deep-merge into `mcpServers`.
    let mut replaced = false;
    match obj.get_mut("mcpServers") {
        None => {
            let mut m = Map::new();
            m.insert(OUR_KEY.to_string(), our_entry.clone());
            obj.insert("mcpServers".to_string(), Value::Object(m));
        }
        Some(Value::Object(servers)) => {
            let existing = servers.get(OUR_KEY);
            if existing == Some(&our_entry) {
                return Ok(MergeOutcome::Idempotent); // 7.
            }
            if existing.is_some() {
                replaced = true;
            }
            servers.insert(OUR_KEY.to_string(), our_entry.clone());
        }
        Some(_non_object) => {
            return Err(ChromaError::ConfigCorrupt {
                path: input.path.display().to_string(),
                quarantine: quarantine_copy(input.path)?.display().to_string(),
            });
        }
    }

    // 6. Backup (only if original existed).
    if existed {
        let _ = backup_file(input.path);
        prune_backups(input.path, 3);
    }

    // 5. Atomic write.
    atomic_write(input.path, &root)?;

    if !existed {
        Ok(MergeOutcome::Created)
    } else if replaced {
        Ok(MergeOutcome::Replaced)
    } else {
        Ok(MergeOutcome::Merged)
    }
}

fn our_entry(bin: &Path, extra_args: &[&str]) -> Value {
    let mut args = vec!["serve".to_string()];
    args.extend(extra_args.iter().map(|s| (*s).to_string()));
    serde_json::json!({
        "command": bin.display().to_string(),
        "args": args,
    })
}

fn parse_strict_or_quarantine(path: &Path, text: &str) -> Result<Value> {
    match serde_json::from_str::<Value>(text) {
        Ok(v) => Ok(v),
        Err(_) => {
            let q = quarantine_copy(path)?;
            Err(ChromaError::ConfigCorrupt {
                path: path.display().to_string(),
                quarantine: q.display().to_string(),
            })
        }
    }
}

fn sibling_with_suffix(path: &Path, suffix: &str) -> PathBuf {
    // `Path::with_extension` *replaces* the existing extension, which turns
    // `c.json` into `c.bak-123` — that breaks `prune_backups` (whose prefix is
    // `{filename}.bak-`). We want to *append* a new extension instead, so the
    // result is `c.json.bak-123`.
    let parent = path.parent().unwrap_or(Path::new("."));
    let stem = path.file_name().map(|s| s.to_os_string()).unwrap_or_default();
    let mut name = stem;
    name.push(".");
    name.push(suffix);
    parent.join(name)
}

fn quarantine_copy(path: &Path) -> Result<PathBuf> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let q = sibling_with_suffix(path, &format!("corrupt-{ts}"));
    let _ = fs::copy(path, &q);
    Ok(q)
}

fn backup_file(path: &Path) -> Result<PathBuf> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Second-resolution timestamps collide when merge runs twice in the same
    // second (e.g. in tests). Fall through to a nanosecond suffix if the
    // primary name already exists.
    let mut b = sibling_with_suffix(path, &format!("bak-{ts}"));
    if b.exists() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        b = sibling_with_suffix(path, &format!("bak-{ts}-{nanos:09}"));
    }
    fs::copy(path, &b).map_err(|e| ChromaError::SessionInitFailed(e.to_string()))?;
    Ok(b)
}

fn prune_backups(path: &Path, keep: usize) {
    let Some(dir) = path.parent() else { return };
    let Some(stem) = path.file_name().and_then(|s| s.to_str()) else {
        return;
    };
    let prefix = format!("{stem}.bak-");
    let mut entries: Vec<_> = fs::read_dir(dir)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with(&prefix))
        .collect();
    entries.sort_by_key(|e| e.file_name());
    while entries.len() > keep {
        let old = entries.remove(0);
        let _ = fs::remove_file(old.path());
    }
}

fn atomic_write(path: &Path, value: &Value) -> Result<()> {
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(dir).map_err(|e| ChromaError::SessionInitFailed(e.to_string()))?;
    let tmp = dir.join(format!(
        ".{}.tmp-{}",
        path.file_name().and_then(|s| s.to_str()).unwrap_or("cfg"),
        std::process::id()
    ));
    let text =
        serde_json::to_string_pretty(value).map_err(|e| ChromaError::SessionInitFailed(e.to_string()))?;
    fs::write(&tmp, &text).map_err(|e| ChromaError::SessionInitFailed(e.to_string()))?;
    // fsync — best-effort on Windows; rename is the actual atomic boundary.
    if let Ok(f) = fs::File::open(&tmp) {
        let _ = f.sync_all();
    }
    fs::rename(&tmp, path).map_err(|e| ChromaError::SessionInitFailed(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn run(dir: &Path, name: &str, initial: Option<&str>) -> (PathBuf, Result<MergeOutcome>) {
        let p = dir.join(name);
        if let Some(txt) = initial {
            fs::write(&p, txt).unwrap();
        }
        let bin = PathBuf::from(r"C:\Users\tester\razer-chroma-mcp.exe");
        let input = MergeInput {
            path: &p,
            binary_path: &bin,
            extra_args: &[],
        };
        (p.clone(), merge(&input))
    }

    #[test]
    fn missing_file_creates_it() {
        let tmp = tempdir().unwrap();
        let (_, r) = run(tmp.path(), "c.json", None);
        assert_eq!(r.unwrap(), MergeOutcome::Created);
    }

    #[test]
    fn empty_file_treated_as_empty_object() {
        let tmp = tempdir().unwrap();
        let (p, r) = run(tmp.path(), "c.json", Some(""));
        assert_eq!(r.unwrap(), MergeOutcome::Merged);
        let v: Value = serde_json::from_str(&fs::read_to_string(&p).unwrap()).unwrap();
        assert!(v["mcpServers"]["razer-chroma"].is_object());
    }

    #[test]
    fn bare_object_adds_mcp_servers() {
        let tmp = tempdir().unwrap();
        let (_, r) = run(tmp.path(), "c.json", Some("{}"));
        assert_eq!(r.unwrap(), MergeOutcome::Merged);
    }

    #[test]
    fn preserves_existing_other_servers() {
        let tmp = tempdir().unwrap();
        let init = r#"{ "mcpServers": { "other": { "command": "x" } }, "otherTop": true }"#;
        let (p, r) = run(tmp.path(), "c.json", Some(init));
        assert_eq!(r.unwrap(), MergeOutcome::Merged);
        let v: Value = serde_json::from_str(&fs::read_to_string(&p).unwrap()).unwrap();
        assert_eq!(v["mcpServers"]["other"]["command"], "x");
        assert_eq!(v["otherTop"], true);
        assert!(v["mcpServers"]["razer-chroma"].is_object());
    }

    #[test]
    fn malformed_json_quarantines_and_errors() {
        let tmp = tempdir().unwrap();
        let (_, r) = run(tmp.path(), "c.json", Some("{ not valid"));
        assert!(matches!(r, Err(ChromaError::ConfigCorrupt { .. })));
        assert!(fs::read_dir(tmp.path())
            .unwrap()
            .any(|e| { e.unwrap().file_name().to_string_lossy().contains("corrupt-") }));
    }

    #[test]
    fn non_object_root_errors() {
        let tmp = tempdir().unwrap();
        let (_, r) = run(tmp.path(), "c.json", Some("[1,2,3]"));
        assert!(matches!(r, Err(ChromaError::ConfigCorrupt { .. })));
    }

    #[test]
    fn non_object_mcp_servers_errors() {
        let tmp = tempdir().unwrap();
        let (_, r) = run(tmp.path(), "c.json", Some(r#"{ "mcpServers": "oops" }"#));
        assert!(matches!(r, Err(ChromaError::ConfigCorrupt { .. })));
    }

    #[test]
    fn idempotent_rerun_does_not_write() {
        let tmp = tempdir().unwrap();
        let (p, r1) = run(tmp.path(), "c.json", None);
        assert_eq!(r1.unwrap(), MergeOutcome::Created);
        let first_mtime = fs::metadata(&p).unwrap().modified().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
        let bin = PathBuf::from(r"C:\Users\tester\razer-chroma-mcp.exe");
        let input = MergeInput {
            path: &p,
            binary_path: &bin,
            extra_args: &[],
        };
        assert_eq!(merge(&input).unwrap(), MergeOutcome::Idempotent);
        let second_mtime = fs::metadata(&p).unwrap().modified().unwrap();
        assert_eq!(first_mtime, second_mtime);
    }

    #[test]
    fn replaces_existing_entry_with_different_values() {
        let tmp = tempdir().unwrap();
        let init = r#"{ "mcpServers": { "razer-chroma": { "command": "old.exe", "args": ["serve"] } } }"#;
        let (p, r) = run(tmp.path(), "c.json", Some(init));
        assert_eq!(r.unwrap(), MergeOutcome::Replaced);
        let v: Value = serde_json::from_str(&fs::read_to_string(&p).unwrap()).unwrap();
        assert_eq!(
            v["mcpServers"]["razer-chroma"]["command"],
            r"C:\Users\tester\razer-chroma-mcp.exe"
        );
    }

    #[test]
    fn backup_rotation_keeps_three() {
        let tmp = tempdir().unwrap();
        let p = tmp.path().join("c.json");
        fs::write(&p, r#"{ "mcpServers": {} }"#).unwrap();
        // Run merge 5 times, each with a slightly different binary path so
        // it's never idempotent.
        for i in 0..5 {
            let bin_i =
                PathBuf::from(format!(r"C:\Users\tester\razer-chroma-mcp-{i}.exe"));
            let input = MergeInput {
                path: &p,
                binary_path: &bin_i,
                extra_args: &[],
            };
            let _ = merge(&input).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let baks: Vec<_> = fs::read_dir(tmp.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(".bak-"))
            .collect();
        assert!(baks.len() <= 3, "expected ≤3 backups, got {}", baks.len());
    }
}
