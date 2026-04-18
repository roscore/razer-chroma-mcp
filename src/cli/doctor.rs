//! `doctor` — environment diagnostics. See spec §7.3.

use crate::chroma::client::ChromaClient;
use crate::chroma::session::{Session, SessionConfig};
use anyhow::Result;
use std::path::PathBuf;

pub async fn run() -> Result<()> {
    let mut any_fail = false;

    println!("razer-chroma-mcp doctor\n");

    // 1. Synapse installed (registry)
    let synapse_installed = is_synapse_installed();
    report(
        "Razer Synapse installed",
        synapse_installed,
        "Install from https://www.razer.com/synapse",
    );
    any_fail |= !synapse_installed;

    // 2. Synapse running (process)
    let synapse_running = is_synapse_running();
    report(
        "Razer Synapse running",
        synapse_running,
        "Start Razer Synapse from the Start menu",
    );
    any_fail |= !synapse_running;

    // 3. Chroma REST API reachable
    let client = ChromaClient::default();
    let reachable = probe_root().await;
    report(
        "Chroma REST API reachable (port 54235)",
        reachable,
        "Open Synapse → Chroma → enable 'Chroma Apps'",
    );
    any_fail |= !reachable;

    // 4. Session init
    let init_ok = Session::init(client.clone(), SessionConfig::default()).await;
    match &init_ok {
        Ok(session) => {
            report("Session init succeeds", true, "");
            session.shutdown().await;
        }
        Err(e) => {
            report("Session init succeeds", false, &format!("{e}"));
            any_fail = true;
        }
    }

    // 5. At least one Chroma device detected — Chroma SDK does not expose
    //    enumeration, so use init success as a proxy (spec §7.3 accepts this).
    report(
        "At least one Chroma device detected",
        init_ok.is_ok(),
        "Plug in a Chroma-capable device; restart Synapse",
    );

    // 6 + 7. Claude Desktop / Claude Code entries
    use crate::config::paths;
    let desktop_ok = paths::claude_desktop_config_has_our_entry().unwrap_or(false);
    report(
        "Claude Desktop config has `razer-chroma`",
        desktop_ok,
        "Run: razer-chroma-mcp setup",
    );
    let code_ok = paths::claude_code_config_has_our_entry().unwrap_or(false);
    if code_ok {
        report("Claude Code config has `razer-chroma`", true, "");
    } else {
        println!("[ - ] Claude Code config has `razer-chroma`  (optional)");
    }

    // Log path
    println!();
    println!("Log file: {}", log_path().display());

    if any_fail {
        println!("\nSome checks failed. See per-item 'how to fix' above.");
        std::process::exit(2);
    }
    Ok(())
}

fn report(label: &str, ok: bool, fix: &str) {
    if ok {
        println!("[\u{2713}] {label}");
    } else {
        println!("[x] {label}");
        if !fix.is_empty() {
            println!("    fix: {fix}");
        }
    }
}

fn is_synapse_installed() -> bool {
    #[cfg(windows)]
    {
        use std::process::Command;
        // `reg query` is cheaper than adding a winreg dep for a single probe.
        let keys = [
            r"HKCU\Software\Razer\Synapse3",
            r"HKLM\Software\Razer\Synapse3",
            r"HKCU\Software\Razer\Synapse",
        ];
        keys.iter().any(|k| {
            Command::new("reg")
                .args(["query", k])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        })
    }
    #[cfg(not(windows))]
    {
        false
    }
}

fn is_synapse_running() -> bool {
    #[cfg(windows)]
    {
        use std::process::Command;
        Command::new("tasklist")
            .arg("/FI")
            .arg("IMAGENAME eq Razer Synapse 3.exe")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains("Razer Synapse"))
            .unwrap_or(false)
    }
    #[cfg(not(windows))]
    {
        false
    }
}

async fn probe_root() -> bool {
    let c = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(800))
        .build()
        .unwrap();
    c.get("http://localhost:54235/")
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

fn log_path() -> PathBuf {
    std::env::var("LOCALAPPDATA")
        .map(|p| PathBuf::from(p).join("razer-chroma-mcp").join("logs"))
        .unwrap_or_else(|_| PathBuf::from("./logs"))
}
