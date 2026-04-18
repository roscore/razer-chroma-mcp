//! Platform config path resolution + quick "does our entry exist" checks.
//! The full install/remove flow lives in M6 (setup/uninstall); this module
//! already provides the read-only helpers used by `doctor` (T5.4).

use std::path::PathBuf;

pub fn claude_desktop_config() -> Option<PathBuf> {
    std::env::var("APPDATA")
        .ok()
        .map(|p| PathBuf::from(p).join("Claude").join("claude_desktop_config.json"))
}

pub fn claude_code_config() -> Option<PathBuf> {
    std::env::var("USERPROFILE")
        .ok()
        .map(|p| PathBuf::from(p).join(".claude.json"))
}

pub fn claude_desktop_config_has_our_entry() -> Option<bool> {
    has_entry(claude_desktop_config()?)
}

pub fn claude_code_config_has_our_entry() -> Option<bool> {
    has_entry(claude_code_config()?)
}

fn has_entry(path: PathBuf) -> Option<bool> {
    let text = std::fs::read_to_string(&path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    Some(
        v.get("mcpServers")
            .and_then(|s| s.as_object())
            .map(|m| m.contains_key("razer-chroma"))
            .unwrap_or(false),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn has_entry_detects_mcp_servers_key() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(
            f,
            r#"{{ "mcpServers": {{ "razer-chroma": {{ "command": "x" }} }} }}"#
        )
        .unwrap();
        assert_eq!(has_entry(f.path().to_path_buf()), Some(true));
    }

    #[test]
    fn has_entry_returns_false_when_key_absent() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, r#"{{ "mcpServers": {{ "other": {{}} }} }}"#).unwrap();
        assert_eq!(has_entry(f.path().to_path_buf()), Some(false));
    }

    #[test]
    fn has_entry_returns_none_on_missing_file() {
        assert_eq!(has_entry(PathBuf::from("<nope>/zzz.json")), None);
    }
}
