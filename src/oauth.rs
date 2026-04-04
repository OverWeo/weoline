use std::path::Path;

use crate::types::CredentialsFile;

pub fn get_oauth_token(credentials_file: &Path) -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        if let Some(token) = try_keychain() {
            return Some(token);
        }
    }

    // Fallback to credentials file
    let data = std::fs::read(credentials_file).ok()?;
    let creds: CredentialsFile = serde_json::from_slice(&data).ok()?;
    creds.claude_ai_oauth?.access_token
}

#[cfg(target_os = "macos")]
fn try_keychain() -> Option<String> {
    let output = std::process::Command::new("security")
        .args(["find-generic-password", "-s", "Claude Code-credentials", "-w"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let raw = std::str::from_utf8(&output.stdout).ok()?.trim();
    let creds: CredentialsFile = serde_json::from_str(raw).ok()?;
    creds.claude_ai_oauth?.access_token
}
