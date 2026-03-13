use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RepoInfo {
    pub name: String,
    pub path: PathBuf,
    pub branch: String,
    pub dirty: bool,
    pub ahead: u32,
    pub behind: u32,
    pub last_commit: String,
    pub remote_url: String,
}

/// Scan known directories for git repositories.
pub fn scan_repos() -> Vec<RepoInfo> {
    let home = dirs::home_dir().unwrap_or_default();
    let search_dirs = [
        home.join("vibe"),
        home.join("code"),
        home.join("fm"),
        home.join("solo"),
        home.join("flock"),
    ];

    let mut repos = Vec::new();
    for dir in &search_dirs {
        if dir.is_dir() {
            scan_dir(dir, 2, &mut repos);
        }
    }
    repos.sort_by(|a, b| a.name.cmp(&b.name));
    repos
}

fn scan_dir(dir: &Path, depth: u32, repos: &mut Vec<RepoInfo>) {
    if depth == 0 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if path.file_name().is_some_and(|n| n.to_string_lossy().starts_with('.')) {
            continue;
        }
        if path.join(".git").is_dir() {
            if let Some(info) = repo_info(&path) {
                repos.push(info);
            }
        } else {
            scan_dir(&path, depth - 1, repos);
        }
    }
}

fn repo_info(path: &Path) -> Option<RepoInfo> {
    let name = path.file_name()?.to_string_lossy().to_string();

    let branch = git_output(path, &["rev-parse", "--abbrev-ref", "HEAD"])
        .unwrap_or_else(|| "unknown".into());

    let dirty = git_output(path, &["status", "--porcelain"])
        .is_some_and(|s| !s.trim().is_empty());

    let (ahead, behind) = git_ahead_behind(path);

    let last_commit = git_output(path, &["log", "-1", "--format=%s", "--no-walk"])
        .unwrap_or_else(|| "no commits".into());

    let remote_url = git_output(path, &["remote", "get-url", "origin"])
        .unwrap_or_default();

    Some(RepoInfo {
        name,
        path: path.to_path_buf(),
        branch,
        dirty,
        ahead,
        behind,
        last_commit,
        remote_url,
    })
}

fn git_output(path: &Path, args: &[&str]) -> Option<String> {
    Command::new("git")
        .current_dir(path)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

fn git_ahead_behind(path: &Path) -> (u32, u32) {
    let out = git_output(path, &["rev-list", "--left-right", "--count", "HEAD...@{upstream}"]);
    match out {
        Some(s) => {
            let parts: Vec<&str> = s.split_whitespace().collect();
            if parts.len() == 2 {
                let ahead = parts[0].parse().unwrap_or(0);
                let behind = parts[1].parse().unwrap_or(0);
                (ahead, behind)
            } else {
                (0, 0)
            }
        }
        None => (0, 0),
    }
}
