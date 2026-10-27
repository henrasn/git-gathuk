use anyhow::{Context, Result};
use std::process::Command;

/// Shell out to git to get the list of conflicted files.
pub fn get_conflicted_files() -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["diff", "--name-only", "--diff-filter=U"])
        .output()
        .context("Failed to execute `git diff --name-only --diff-filter=U`")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git diff error: {}", err.trim());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let files = stdout
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(files)
}

/// Returns the current branch name (or short commit hash if detached).
pub fn get_active_branch() -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .context("Failed to execute `git rev-parse --abbrev-ref HEAD`")?;

    if !output.status.success() {
        return Ok("HEAD".to_string());
    }

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(if branch.is_empty() { "HEAD".to_string() } else { branch })
}

/// Returns the absolute path of the repository root.
pub fn get_repo_root() -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("Failed to execute `git rev-parse --show-toplevel`")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git rev-parse error: {}", err.trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Retrieves the Local (HEAD) version of a file from Git index stage 2.
pub fn get_local_content(filepath: &str) -> Result<String> {
    let spec = format!(":2:{}", filepath);
    let output = Command::new("git")
        .args(["show", &spec])
        .output()
        .context(format!("Failed to execute `git show {}`", spec))?;

    if !output.status.success() {
        // Fallback: try reading the file directly if stage 2 is missing
        return Ok(String::new());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Retrieves the Remote (MERGE_HEAD / incoming) version of a file from Git index stage 3.
pub fn get_remote_content(filepath: &str) -> Result<String> {
    let spec = format!(":3:{}", filepath);
    let output = Command::new("git")
        .args(["show", &spec])
        .output()
        .context(format!("Failed to execute `git show {}`", spec))?;

    if !output.status.success() {
        return Ok(String::new());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Stages a file into the Git index.
pub fn stage_file(filepath: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["add", filepath])
        .output()
        .context(format!("Failed to execute `git add {}`", filepath))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git add failed for {}: {}", filepath, err.trim());
    }

    Ok(())
}
