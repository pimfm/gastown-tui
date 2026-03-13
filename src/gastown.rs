use serde::{Deserialize, Serialize};
use std::process::Command;

/// Check if the `gt` CLI is available on the system.
pub fn gt_available() -> bool {
    Command::new("gt")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}

/// Check if the `bd` (beads) CLI is available.
pub fn bd_available() -> bool {
    Command::new("bd")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub name: String,
    pub runtime: String,
    pub rig: String,
    pub status: AgentStatus,
    pub current_bead: Option<String>,
    pub beads_completed: u32,
    pub uptime_secs: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    Active,
    Idle,
    Stuck,
    Offline,
}

impl AgentStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Active => "ACTIVE",
            Self::Idle => "IDLE",
            Self::Stuck => "STUCK",
            Self::Offline => "OFFLINE",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Convoy {
    pub id: String,
    pub name: String,
    pub total_beads: u32,
    pub completed_beads: u32,
    pub in_progress_beads: u32,
    pub status: ConvoyStatus,
    pub agents_assigned: Vec<String>,
}

impl Convoy {
    pub fn progress(&self) -> f64 {
        if self.total_beads == 0 {
            return 0.0;
        }
        self.completed_beads as f64 / self.total_beads as f64
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConvoyStatus {
    Active,
    Completed,
    Paused,
    Blocked,
}

impl ConvoyStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Active => "ACTIVE",
            Self::Completed => "DONE",
            Self::Paused => "PAUSED",
            Self::Blocked => "BLOCKED",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bead {
    pub id: String,
    pub title: String,
    pub status: BeadStatus,
    pub assigned_to: Option<String>,
    pub convoy_id: Option<String>,
    pub rig: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BeadStatus {
    Open,
    InProgress,
    Done,
    Blocked,
}

impl BeadStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Open => "OPEN",
            Self::InProgress => "IN PROG",
            Self::Done => "DONE",
            Self::Blocked => "BLOCKED",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedEntry {
    pub timestamp: String,
    pub agent: String,
    pub action: String,
    pub detail: String,
    pub severity: Severity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Success,
}

/// Attempt to fetch live data from Gas Town CLI. Falls back to demo data.
pub fn fetch_agents() -> Vec<Agent> {
    if gt_available() {
        if let Some(agents) = try_gt_command(&["agents", "--json"]) {
            if let Ok(parsed) = serde_json::from_str::<Vec<Agent>>(&agents) {
                return parsed;
            }
        }
    }
    demo_agents()
}

pub fn fetch_convoys() -> Vec<Convoy> {
    if gt_available() {
        if let Some(out) = try_gt_command(&["convoy", "list", "--json"]) {
            if let Ok(parsed) = serde_json::from_str::<Vec<Convoy>>(&out) {
                return parsed;
            }
        }
    }
    demo_convoys()
}

pub fn fetch_beads() -> Vec<Bead> {
    if bd_available() {
        if let Some(out) = try_bd_command(&["list", "--json"]) {
            if let Ok(parsed) = serde_json::from_str::<Vec<Bead>>(&out) {
                return parsed;
            }
        }
    }
    demo_beads()
}

pub fn fetch_feed() -> Vec<FeedEntry> {
    if gt_available() {
        if let Some(out) = try_gt_command(&["feed", "--json", "--limit", "50"]) {
            if let Ok(parsed) = serde_json::from_str::<Vec<FeedEntry>>(&out) {
                return parsed;
            }
        }
    }
    demo_feed()
}

fn try_gt_command(args: &[&str]) -> Option<String> {
    Command::new("gt")
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
}

fn try_bd_command(args: &[&str]) -> Option<String> {
    Command::new("bd")
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
}

// ── Demo data for when Gas Town isn't installed ──────────────────────

fn demo_agents() -> Vec<Agent> {
    let runtimes = ["claude", "claude", "claude", "gemini", "codex", "claude", "cursor", "claude"];
    let rigs = ["gastown-tui", "neuroplan", "profit-cli", "matrix", "specflow", "healthcentral", "pimfm", "token-tracker"];
    let statuses = [
        AgentStatus::Active, AgentStatus::Active, AgentStatus::Idle,
        AgentStatus::Active, AgentStatus::Stuck, AgentStatus::Active,
        AgentStatus::Idle, AgentStatus::Active,
    ];
    let beads_done = [12, 8, 5, 15, 3, 7, 9, 11];
    let uptimes = [3600, 7200, 1800, 5400, 900, 4500, 2700, 6300];

    let mut agents = Vec::new();
    for i in 0..40 {
        let idx = i % 8;
        let name = if i < 8 {
            format!("polecat-{:02}", i + 1)
        } else {
            format!("polecat-{:02}", i + 1)
        };
        agents.push(Agent {
            name,
            runtime: runtimes[idx].to_string(),
            rig: rigs[idx].to_string(),
            status: statuses[idx],
            current_bead: if statuses[idx] == AgentStatus::Active {
                Some(format!("gt-{:05x}", (i * 7 + 42) % 0xfffff))
            } else {
                None
            },
            beads_completed: beads_done[idx] + (i as u32 / 8) * 3,
            uptime_secs: uptimes[idx] + (i as u64) * 300,
        });
    }
    agents
}

fn demo_convoys() -> Vec<Convoy> {
    vec![
        Convoy {
            id: "cv-001".into(), name: "API Refactor".into(),
            total_beads: 24, completed_beads: 18, in_progress_beads: 4,
            status: ConvoyStatus::Active,
            agents_assigned: vec!["polecat-01".into(), "polecat-02".into(), "polecat-09".into()],
        },
        Convoy {
            id: "cv-002".into(), name: "Auth Middleware Rewrite".into(),
            total_beads: 16, completed_beads: 16, in_progress_beads: 0,
            status: ConvoyStatus::Completed,
            agents_assigned: vec!["polecat-03".into(), "polecat-04".into()],
        },
        Convoy {
            id: "cv-003".into(), name: "Frontend Dashboard".into(),
            total_beads: 32, completed_beads: 12, in_progress_beads: 8,
            status: ConvoyStatus::Active,
            agents_assigned: vec!["polecat-05".into(), "polecat-06".into(), "polecat-07".into(), "polecat-08".into()],
        },
        Convoy {
            id: "cv-004".into(), name: "Data Pipeline v2".into(),
            total_beads: 20, completed_beads: 5, in_progress_beads: 2,
            status: ConvoyStatus::Active,
            agents_assigned: vec!["polecat-10".into(), "polecat-11".into()],
        },
        Convoy {
            id: "cv-005".into(), name: "Mobile Integration".into(),
            total_beads: 12, completed_beads: 0, in_progress_beads: 0,
            status: ConvoyStatus::Paused,
            agents_assigned: vec![],
        },
        Convoy {
            id: "cv-006".into(), name: "Observability Stack".into(),
            total_beads: 8, completed_beads: 2, in_progress_beads: 1,
            status: ConvoyStatus::Blocked,
            agents_assigned: vec!["polecat-12".into()],
        },
        Convoy {
            id: "cv-007".into(), name: "CI/CD Overhaul".into(),
            total_beads: 14, completed_beads: 10, in_progress_beads: 3,
            status: ConvoyStatus::Active,
            agents_assigned: vec!["polecat-13".into(), "polecat-14".into(), "polecat-15".into()],
        },
        Convoy {
            id: "cv-008".into(), name: "Perf Optimization".into(),
            total_beads: 18, completed_beads: 6, in_progress_beads: 4,
            status: ConvoyStatus::Active,
            agents_assigned: vec!["polecat-16".into(), "polecat-17".into()],
        },
    ]
}

fn demo_beads() -> Vec<Bead> {
    let titles = [
        "Implement REST endpoint for /users",
        "Add JWT validation middleware",
        "Refactor database connection pool",
        "Write integration tests for auth flow",
        "Update OpenAPI spec",
        "Fix N+1 query in orders endpoint",
        "Add rate limiting to public API",
        "Implement webhook retry logic",
        "Add Prometheus metrics exporter",
        "Set up Grafana dashboards",
        "Configure alerting rules",
        "Migrate to new ORM version",
        "Add pagination to list endpoints",
        "Implement search functionality",
        "Add caching layer for hot paths",
        "Write E2E tests for checkout flow",
        "Implement SSO integration",
        "Add audit logging",
        "Refactor error handling",
        "Update CI pipeline for monorepo",
    ];
    let statuses = [
        BeadStatus::Done, BeadStatus::Done, BeadStatus::InProgress,
        BeadStatus::InProgress, BeadStatus::Open, BeadStatus::Done,
        BeadStatus::InProgress, BeadStatus::Blocked, BeadStatus::InProgress,
        BeadStatus::Open, BeadStatus::Open, BeadStatus::Done,
        BeadStatus::InProgress, BeadStatus::Open, BeadStatus::InProgress,
        BeadStatus::Blocked, BeadStatus::Open, BeadStatus::Open,
        BeadStatus::Done, BeadStatus::InProgress,
    ];
    let rigs = [
        "profit-cli", "profit-cli", "neuroplan", "neuroplan", "matrix",
        "healthcentral", "healthcentral", "specflow", "specflow", "pimfm",
        "pimfm", "token-tracker", "token-tracker", "matrix", "matrix",
        "specflow", "neuroplan", "profit-cli", "profit-cli", "gastown-tui",
    ];

    titles.iter().enumerate().map(|(i, title)| {
        let status = statuses[i];
        Bead {
            id: format!("gt-{:05x}", i * 13 + 100),
            title: title.to_string(),
            status,
            assigned_to: if status == BeadStatus::InProgress {
                Some(format!("polecat-{:02}", (i % 15) + 1))
            } else {
                None
            },
            convoy_id: Some(format!("cv-{:03}", (i / 3) + 1)),
            rig: rigs[i].to_string(),
        }
    }).collect()
}

fn demo_feed() -> Vec<FeedEntry> {
    vec![
        FeedEntry { timestamp: "14:23:01".into(), agent: "polecat-01".into(), action: "completed".into(), detail: "gt-00064 Implement REST endpoint".into(), severity: Severity::Success },
        FeedEntry { timestamp: "14:22:45".into(), agent: "polecat-04".into(), action: "started".into(), detail: "gt-0012c Migrate to new ORM".into(), severity: Severity::Info },
        FeedEntry { timestamp: "14:22:12".into(), agent: "mayor".into(), action: "slung".into(), detail: "gt-00096 to polecat-05 in specflow".into(), severity: Severity::Info },
        FeedEntry { timestamp: "14:21:58".into(), agent: "polecat-05".into(), action: "stuck".into(), detail: "Merge conflict in src/handlers.rs".into(), severity: Severity::Warning },
        FeedEntry { timestamp: "14:21:30".into(), agent: "polecat-02".into(), action: "completed".into(), detail: "gt-0007d Add JWT validation".into(), severity: Severity::Success },
        FeedEntry { timestamp: "14:20:15".into(), agent: "polecat-08".into(), action: "blocked".into(), detail: "Waiting on dependency gt-00096".into(), severity: Severity::Error },
        FeedEntry { timestamp: "14:19:42".into(), agent: "mayor".into(), action: "created".into(), detail: "Convoy cv-008 Perf Optimization".into(), severity: Severity::Info },
        FeedEntry { timestamp: "14:18:55".into(), agent: "polecat-10".into(), action: "started".into(), detail: "gt-000c8 Add caching layer".into(), severity: Severity::Info },
        FeedEntry { timestamp: "14:17:30".into(), agent: "polecat-03".into(), action: "completed".into(), detail: "gt-000af Refactor error handling".into(), severity: Severity::Success },
        FeedEntry { timestamp: "14:16:22".into(), agent: "polecat-12".into(), action: "started".into(), detail: "gt-000be Update CI pipeline".into(), severity: Severity::Info },
        FeedEntry { timestamp: "14:15:00".into(), agent: "mayor".into(), action: "slung".into(), detail: "gt-000d2 to polecat-16 in matrix".into(), severity: Severity::Info },
        FeedEntry { timestamp: "14:14:33".into(), agent: "polecat-07".into(), action: "completed".into(), detail: "gt-00091 Add pagination".into(), severity: Severity::Success },
    ]
}
