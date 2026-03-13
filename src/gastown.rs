#![allow(dead_code)]

use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Command;

// ── Gas Town workspace discovery ─────────────────────────────────────

pub fn find_gt_root() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let gt = home.join("gt");
    if gt.join("mayor").is_dir() {
        Some(gt)
    } else {
        None
    }
}

pub fn gt_available() -> bool {
    Command::new("gt")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}

pub fn bd_available() -> bool {
    Command::new("bd")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}

// ── JSON structs matching real `gt status --json` ────────────────────

#[derive(Debug, Deserialize)]
pub struct TownStatus {
    pub name: String,
    pub location: String,
    pub overseer: Option<Overseer>,
    pub daemon: Option<ServiceStatus>,
    pub dolt: Option<DoltStatus>,
    pub tmux: Option<TmuxStatus>,
    pub agents: Vec<GtAgent>,
    pub rigs: Vec<GtRig>,
    pub summary: Option<TownSummary>,
}

#[derive(Debug, Deserialize)]
pub struct Overseer {
    pub name: String,
    pub email: String,
    pub unread_mail: u32,
}

#[derive(Debug, Deserialize)]
pub struct ServiceStatus {
    pub running: bool,
    pub pid: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct DoltStatus {
    pub running: bool,
    pub pid: Option<u64>,
    pub port: Option<u16>,
    pub data_dir: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TmuxStatus {
    pub socket: Option<String>,
    pub running: bool,
    pub session_count: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GtAgent {
    pub name: String,
    pub address: String,
    pub session: Option<String>,
    pub role: Option<String>,
    pub running: bool,
    pub has_work: bool,
    pub state: Option<String>,
    pub work_title: Option<String>,
    pub hook_bead: Option<String>,
    pub unread_mail: Option<u32>,
    pub agent_alias: Option<String>,
    pub agent_info: Option<String>,
    #[serde(default)]
    pub acp: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GtRig {
    pub name: String,
    pub polecats: Option<Vec<String>>,
    pub polecat_count: u32,
    pub crews: Option<Vec<String>>,
    pub crew_count: u32,
    pub has_witness: bool,
    pub has_refinery: bool,
    pub hooks: Vec<GtHook>,
    pub agents: Vec<GtAgent>,
    pub mq: Option<GtMq>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GtHook {
    pub agent: String,
    pub role: String,
    pub has_work: bool,
    pub molecule: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GtMq {
    pub pending: u32,
    pub in_flight: u32,
    pub blocked: u32,
    pub state: Option<String>,
    pub health: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TownSummary {
    pub rig_count: u32,
    pub polecat_count: u32,
    pub crew_count: u32,
    pub witness_count: u32,
    pub refinery_count: u32,
    pub active_hooks: u32,
}

// ── JSON structs matching real `gt convoy list --json` ────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct GtConvoy {
    pub id: String,
    pub title: String,
    pub status: String,
    pub created_at: Option<String>,
    pub tracked: Option<Vec<ConvoyTracked>>,
    pub completed: u32,
    pub total: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConvoyTracked {
    pub id: String,
    pub title: String,
    pub status: String,
    pub assignee: Option<String>,
    pub worker: Option<String>,
    pub worker_age: Option<String>,
    pub blocked: Option<bool>,
}

// ── JSON structs matching real `bd list --json --all` ─────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct BdIssue {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: i32,
    pub issue_type: String,
    pub assignee: Option<String>,
    pub owner: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub labels: Option<Vec<String>>,
    pub dependency_count: Option<u32>,
    pub dependent_count: Option<u32>,
    pub comment_count: Option<u32>,
}

// ── JSON structs matching real `gt mail inbox --json` ─────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct GtMail {
    pub id: String,
    pub from: String,
    pub to: Option<String>,
    pub subject: String,
    pub body: Option<String>,
    pub timestamp: Option<String>,
    pub read: bool,
    pub priority: Option<String>,
    #[serde(rename = "type")]
    pub msg_type: Option<String>,
}

// ── Events from .events.jsonl ────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct GtEvent {
    pub timestamp: Option<String>,
    pub event_type: Option<String>,
    pub actor: Option<String>,
    pub target: Option<String>,
    pub detail: Option<String>,
    pub rig: Option<String>,
}

// ── Bootstrap: start all Gas Town services ──────────────────────────

/// Run `gt up` to boot dolt, daemon, deacon, mayor, witnesses, and refineries.
/// This is idempotent — only starts services that aren't already running.
pub fn boot_services(gt_root: &Path) -> Result<String, String> {
    let out = Command::new("gt")
        .current_dir(gt_root)
        .args(["up", "--quiet"])
        .output()
        .map_err(|e| format!("failed to run gt up: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

// ── Fetching functions ───────────────────────────────────────────────

pub fn fetch_status(gt_root: &Path) -> Option<TownStatus> {
    let out = Command::new("gt")
        .current_dir(gt_root)
        .args(["status", "--json"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    serde_json::from_slice(&out.stdout).ok()
}

pub fn fetch_convoys(gt_root: &Path) -> Vec<GtConvoy> {
    let out = Command::new("gt")
        .current_dir(gt_root)
        .args(["convoy", "list", "--json"])
        .output();
    match out {
        Ok(o) if o.status.success() => {
            serde_json::from_slice(&o.stdout).unwrap_or_default()
        }
        _ => Vec::new(),
    }
}

pub fn fetch_beads_for_rig(gt_root: &Path, rig_name: &str) -> Vec<BdIssue> {
    let rig_dir = gt_root.join(rig_name);
    if !rig_dir.is_dir() {
        return Vec::new();
    }
    let out = Command::new("bd")
        .current_dir(&rig_dir)
        .args(["list", "--json", "--all", "-n", "0"])
        .output();
    match out {
        Ok(o) if o.status.success() => {
            serde_json::from_slice(&o.stdout).unwrap_or_default()
        }
        _ => Vec::new(),
    }
}

pub fn fetch_hq_beads(gt_root: &Path) -> Vec<BdIssue> {
    let out = Command::new("bd")
        .current_dir(gt_root)
        .args(["list", "--json", "--all", "-n", "0"])
        .output();
    match out {
        Ok(o) if o.status.success() => {
            serde_json::from_slice(&o.stdout).unwrap_or_default()
        }
        _ => Vec::new(),
    }
}

pub fn fetch_mail(gt_root: &Path) -> Vec<GtMail> {
    let out = Command::new("gt")
        .current_dir(gt_root)
        .args(["mail", "inbox", "--json"])
        .output();
    match out {
        Ok(o) if o.status.success() => {
            serde_json::from_slice(&o.stdout).unwrap_or_default()
        }
        _ => Vec::new(),
    }
}

pub fn fetch_events(gt_root: &Path, limit: usize) -> Vec<GtEvent> {
    let events_file = gt_root.join(".events.jsonl");
    if !events_file.exists() {
        return Vec::new();
    }
    let content = std::fs::read_to_string(&events_file).unwrap_or_default();
    let mut events: Vec<GtEvent> = content
        .lines()
        .rev()
        .take(limit)
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();
    events.reverse();
    events
}

// ── Actions ──────────────────────────────────────────────────────────

pub fn sling_bead(gt_root: &Path, bead_id: &str, rig: &str) -> Result<String, String> {
    let out = Command::new("gt")
        .current_dir(gt_root)
        .args(["sling", bead_id, rig])
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).to_string())
    }
}

pub fn create_bead(
    gt_root: &Path,
    rig_name: &str,
    title: &str,
    issue_type: &str,
    priority: u8,
) -> Result<String, String> {
    let rig_dir = gt_root.join(rig_name);
    let out = Command::new("bd")
        .current_dir(&rig_dir)
        .args([
            "create",
            title,
            "-t",
            issue_type,
            "-p",
            &priority.to_string(),
            "--json",
            "--silent",
        ])
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).to_string())
    }
}

pub fn spawn_polecat(gt_root: &Path, rig: &str, bead_id: &str) -> Result<String, String> {
    let out = Command::new("gt")
        .current_dir(gt_root)
        .args(["sling", bead_id, rig])
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).to_string())
    }
}
