//! `setup` — install wizard. Spec §7.2.
//!
//! Walks a first-time user through: Synapse check → Chroma session init →
//! hardware demo → Claude Desktop/Code config merge → final doctor run.
//! Every message is bilingual (EN + KO); the primary language defaults to
//! the system locale but can be forced via `--lang`.

use crate::chroma::client::ChromaClient;
use crate::chroma::session::{Session, SessionConfig};
use crate::config::merge::{merge, MergeInput, MergeOutcome};
use crate::config::paths;
use anyhow::{bail, Context, Result};
use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::time::Duration;

pub async fn run(lang: Option<String>) -> Result<()> {
    let lang = lang.unwrap_or_else(detect_lang);
    let stdin = std::io::stdin();
    let mut reader = stdin.lock();

    say(
        &lang,
        "Welcome to razer-chroma-mcp setup.",
        "razer-chroma-mcp 설치 마법사입니다.",
    );

    // 1. Detect Synapse
    if !synapse_ok().await {
        say(
            &lang,
            "Razer Synapse is not reachable.",
            "Razer Synapse가 실행 중이지 않습니다.",
        );
        say(
            &lang,
            "Please install Synapse from https://www.razer.com/synapse, then press Enter to continue.",
            "https://www.razer.com/synapse 에서 Synapse를 설치한 뒤 Enter를 눌러 주세요.",
        );
        wait_enter(&mut reader);
    }
    while !synapse_ok().await {
        say(
            &lang,
            "Still can't reach Synapse. Press Enter to retry, or Ctrl+C to exit.",
            "Synapse를 찾을 수 없습니다. Enter=재시도, Ctrl+C=종료.",
        );
        wait_enter(&mut reader);
    }
    say(&lang, "[OK] Synapse reachable.", "[OK] Synapse 연결됨.");

    // 2. Verify Chroma SDK
    let client = ChromaClient::default();
    match Session::init(client.clone(), SessionConfig::default()).await {
        Ok(s) => {
            say(
                &lang,
                "[OK] Chroma session initialized.",
                "[OK] Chroma 세션 초기화 성공.",
            );
            // 3. Hardware demo
            say(
                &lang,
                "Playing a 3-second terracotta sweep on your keyboard…",
                "키보드에 3초 동안 테라코타 스윕을 재생합니다...",
            );
            let _ = crate::cli::demo::run().await;
            say(
                &lang,
                "Did you see the lights move? [Y/n]",
                "키보드에 조명 효과가 보였나요? [Y/n]",
            );
            let ans = read_line(&mut reader);
            if ans.trim().eq_ignore_ascii_case("n") {
                say(
                    &lang,
                    "If the lights did not move, your device may not be Chroma-compatible or Synapse may be blocking us. Run `razer-chroma-mcp doctor` for details.",
                    "조명이 보이지 않으면 기기가 Chroma를 지원하지 않거나 Synapse가 차단 중일 수 있습니다. `razer-chroma-mcp doctor`로 진단해 주세요.",
                );
            }
            s.shutdown().await;
        }
        Err(e) => {
            say(
                &lang,
                &format!("[FAIL] Chroma session init failed: {e}"),
                &format!("[FAIL] Chroma 세션 초기화 실패: {e}"),
            );
        }
    }

    // 4–5. Register with Claude Desktop / Code
    let bin = resolve_current_exe(std::env::current_exe)?;
    if let Some(path) = paths::claude_desktop_config() {
        offer_merge(&lang, &path, &bin, "Claude Desktop", &mut reader);
    }
    if let Some(path) = paths::claude_code_config() {
        offer_merge(&lang, &path, &bin, "Claude Code", &mut reader);
    }

    // 6. Final doctor
    say(
        &lang,
        "Running final diagnostics…",
        "최종 진단을 실행합니다...",
    );
    let _ = crate::cli::doctor::run().await;

    // 7. Restart prompt
    say(
        &lang,
        "Setup complete. Please restart Claude Desktop to activate.",
        "설치가 완료되었습니다. Claude Desktop을 재시작해 주세요.",
    );
    Ok(())
}

fn detect_lang() -> String {
    #[cfg(windows)]
    {
        use std::process::Command;
        // Fall back to English unless locale is Korean.
        let out = Command::new("powershell")
            .args(["-NoProfile", "-Command", "(Get-Culture).Name"])
            .output();
        if let Ok(o) = out {
            if String::from_utf8_lossy(&o.stdout).trim().starts_with("ko") {
                return "ko".into();
            }
        }
    }
    "en".into()
}

fn resolve_current_exe<F>(current_exe: F) -> Result<PathBuf>
where
    F: FnOnce() -> std::io::Result<PathBuf>,
{
    let path = current_exe().context(
        "Could not determine the absolute path of this executable. \
         Run setup again from the installed executable. \
         실행 파일의 절대 경로를 확인하지 못했습니다. 설치된 실행 파일에서 setup을 다시 실행해 주세요.",
    )?;
    if !path.is_absolute() {
        bail!(
            "The executable path is not absolute: {}. \
             Run setup again from the installed executable. \
             실행 파일 경로가 절대 경로가 아닙니다. 설치된 실행 파일에서 setup을 다시 실행해 주세요.",
            path.display()
        );
    }
    Ok(path)
}

fn say(lang: &str, en: &str, ko: &str) {
    if lang == "ko" {
        println!("{ko}");
        println!("  ({en})");
    } else {
        println!("{en}");
        println!("  ({ko})");
    }
}

fn read_line<R: BufRead>(r: &mut R) -> String {
    let mut s = String::new();
    r.read_line(&mut s).ok();
    s
}

fn wait_enter<R: BufRead>(r: &mut R) {
    print!("> ");
    let _ = std::io::stdout().flush();
    let _ = read_line(r);
}

async fn synapse_ok() -> bool {
    let c = reqwest::Client::builder()
        .timeout(Duration::from_millis(800))
        .build()
        .unwrap();
    c.get("http://localhost:54235/")
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

fn offer_merge<R: BufRead>(
    lang: &str,
    path: &std::path::Path,
    bin: &std::path::Path,
    label: &str,
    reader: &mut R,
) {
    say(
        lang,
        &format!(
            "Register razer-chroma with {label}? ({}) [Y/n]",
            path.display()
        ),
        &format!(
            "{label}에 razer-chroma를 등록할까요? ({}) [Y/n]",
            path.display()
        ),
    );
    if read_line(reader).trim().eq_ignore_ascii_case("n") {
        return;
    }
    let input = MergeInput {
        path,
        binary_path: bin,
        extra_args: &[],
    };
    match merge(&input) {
        Ok(MergeOutcome::Created) | Ok(MergeOutcome::Merged) | Ok(MergeOutcome::Replaced) => {
            say(
                lang,
                &format!("[OK] {label} configured."),
                &format!("[OK] {label} 설정 완료."),
            );
        }
        Ok(MergeOutcome::Idempotent) => {
            say(
                lang,
                &format!("[OK] {label} already configured."),
                &format!("[OK] {label} 이미 설정되어 있습니다."),
            );
        }
        Err(e) => {
            say(
                lang,
                &format!("[FAIL] Could not write {label} config: {e}"),
                &format!("[FAIL] {label} 설정을 쓰지 못했습니다: {e}"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::path::Path;

    #[test]
    fn current_exe_failure_is_reported_instead_of_using_relative_fallback() {
        let error = resolve_current_exe(|| {
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "injected current_exe failure",
            ))
        })
        .unwrap_err();
        let message = error.to_string();

        assert!(message.contains("Could not determine the absolute path of this executable"));
        assert!(message.contains("실행 파일의 절대 경로를 확인하지 못했습니다"));
        assert!(message.contains("Run setup again from the installed executable"));
    }

    #[test]
    fn current_exe_success_preserves_the_absolute_path() {
        let expected = Path::new("/opt/razer-chroma-mcp/bin/razer-chroma-mcp");
        let actual = resolve_current_exe(|| Ok(expected.to_path_buf())).unwrap();

        assert_eq!(actual, expected);
        assert!(actual.is_absolute());
    }
}
