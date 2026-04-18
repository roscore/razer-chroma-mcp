//! `uninstall` — remove our entry from Claude Desktop / Code configs cleanly.
//! Spec §7.5.

use crate::config::merge::OUR_KEY;
use crate::config::paths;
use anyhow::Result;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn run() -> Result<()> {
    if is_claude_desktop_running() {
        println!("Claude Desktop is currently running.");
        println!(
            "Please close Claude Desktop before uninstalling to avoid config corruption. Continue anyway? [y/N]"
        );
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).ok();
        if !line.trim().eq_ignore_ascii_case("y") {
            println!("Aborting.");
            return Ok(());
        }
    }

    if let Some(p) = paths::claude_desktop_config() {
        remove_entry(&p)?;
    }
    if let Some(p) = paths::claude_code_config() {
        remove_entry(&p)?;
    }
    println!("Uninstall complete.");
    Ok(())
}

fn is_claude_desktop_running() -> bool {
    #[cfg(windows)]
    {
        use std::process::Command;
        Command::new("tasklist")
            .arg("/FI")
            .arg("IMAGENAME eq Claude.exe")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains("Claude.exe"))
            .unwrap_or(false)
    }
    #[cfg(not(windows))]
    {
        false
    }
}

fn remove_entry(path: &Path) -> Result<()> {
    let Ok(text) = fs::read_to_string(path) else {
        return Ok(()); // missing = no-op
    };
    let Ok(mut v) = serde_json::from_str::<Value>(&text) else {
        eprintln!("warn: {} is not valid JSON; skipping", path.display());
        return Ok(());
    };
    let Some(obj) = v.as_object_mut() else {
        return Ok(());
    };
    let Some(Value::Object(servers)) = obj.get_mut("mcpServers") else {
        return Ok(());
    };
    if servers.remove(OUR_KEY).is_none() {
        return Ok(());
    }
    // Backup
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let stem = path
        .file_name()
        .map(|s| s.to_os_string())
        .unwrap_or_default();
    let mut bak_name = stem.clone();
    bak_name.push(".");
    bak_name.push(format!("bak-{ts}"));
    let bak = parent.join(bak_name);
    let _ = fs::copy(path, &bak);
    // Atomic write via tmp + rename.
    let mut tmp_name = std::ffi::OsString::from(".");
    tmp_name.push(&stem);
    tmp_name.push(format!(".tmp-{}", std::process::id()));
    let tmp = parent.join(tmp_name);
    fs::write(&tmp, serde_json::to_string_pretty(&v)?)?;
    fs::rename(&tmp, path)?;
    println!("Removed razer-chroma from {}", path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn removes_our_key_and_preserves_others() {
        let tmp = tempdir().unwrap();
        let p = tmp.path().join("claude.json");
        fs::write(
            &p,
            r#"{"mcpServers":{"razer-chroma":{"command":"x"},"other":{"command":"y"}},"otherTop":1}"#,
        )
        .unwrap();
        remove_entry(&p).unwrap();
        let v: Value = serde_json::from_str(&fs::read_to_string(&p).unwrap()).unwrap();
        assert!(v["mcpServers"]["razer-chroma"].is_null());
        assert_eq!(v["mcpServers"]["other"]["command"], "y");
        assert_eq!(v["otherTop"], 1);
    }

    #[test]
    fn missing_file_is_noop() {
        let tmp = tempdir().unwrap();
        let p = tmp.path().join("absent.json");
        // Should not error.
        remove_entry(&p).unwrap();
    }

    #[test]
    fn no_entry_no_write() {
        let tmp = tempdir().unwrap();
        let p = tmp.path().join("c.json");
        fs::write(&p, r#"{"mcpServers":{"other":{"command":"y"}}}"#).unwrap();
        let before = fs::metadata(&p).unwrap().modified().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
        remove_entry(&p).unwrap();
        let after = fs::metadata(&p).unwrap().modified().unwrap();
        assert_eq!(before, after, "file should be untouched");
    }
}
