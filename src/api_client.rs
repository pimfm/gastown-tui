use std::collections::HashMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

// ── API response types ──────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct DashboardResponse {
    pub town_name: String,
    pub overseer: String,
    pub stats: DashboardStats,
    pub infrastructure: InfraStatus,
    pub rigs: Vec<RigSummary>,
    pub recent_beads: Vec<BeadSummary>,
    pub recent_events: Vec<EventSummary>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct DashboardStats {
    pub rigs: u32,
    pub agents_running: usize,
    pub agents_total: usize,
    pub polecats: u32,
    pub active_hooks: u32,
    pub beads_total: usize,
    pub beads_in_progress: usize,
    pub beads_open: usize,
    pub unread_mail: u32,
    pub convoys_total: usize,
    pub convoys_active: usize,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct InfraStatus {
    pub daemon_running: bool,
    pub dolt_running: bool,
    pub dolt_port: Option<u16>,
    pub tmux_running: bool,
    pub tmux_sessions: u32,
    pub workspace: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RigSummary {
    pub name: String,
    pub polecats: u32,
    pub crews: u32,
    pub has_witness: bool,
    pub has_refinery: bool,
    pub hooks_active: usize,
    pub hooks_total: usize,
    pub mq_health: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BeadSummary {
    pub id: String,
    pub title: String,
    pub status: String,
    pub priority: i32,
    pub rig: String,
    pub assignee: Option<String>,
    pub issue_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventSummary {
    pub timestamp: Option<String>,
    pub event_type: Option<String>,
    pub actor: Option<String>,
    pub detail: Option<String>,
    pub rig: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AgentsResponse {
    pub agents: Vec<AgentInfo>,
    pub total: usize,
    pub running: usize,
    pub with_work: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AgentInfo {
    pub name: String,
    pub address: String,
    pub role: Option<String>,
    pub state: Option<String>,
    pub running: bool,
    pub has_work: bool,
    pub work_title: Option<String>,
    pub hook_bead: Option<String>,
    pub unread_mail: u32,
    pub runtime: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConvoysResponse {
    pub convoys: Vec<ConvoyInfo>,
    pub total: usize,
    pub active: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConvoyInfo {
    pub id: String,
    pub title: String,
    pub status: String,
    pub created_at: Option<String>,
    pub completed: u32,
    pub total: u32,
    pub progress: f64,
    pub workers: Vec<WorkerInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkerInfo {
    pub name: String,
    pub blocked: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BeadsResponse {
    pub beads: Vec<BeadSummary>,
    pub total: usize,
    pub by_status: HashMap<String, usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReposResponse {
    pub repos: Vec<RepoSummary>,
    pub total: usize,
    pub dirty: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RepoSummary {
    pub name: String,
    pub path: String,
    pub branch: String,
    pub dirty: bool,
    pub ahead: u32,
    pub behind: u32,
    pub last_commit: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActionResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct CreateBeadRequest {
    pub rig: String,
    pub title: String,
    pub issue_type: String,
    pub priority: u8,
}

#[derive(Debug, Serialize)]
pub struct SpawnRequest {
    pub rig: String,
    pub bead_id: String,
}

// ── HTTP client ─────────────────────────────────────────────────────

pub struct ApiClient {
    base_url: String,
    agent: ureq::Agent,
}

impl ApiClient {
    pub fn new(base_url: String) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_read(Duration::from_secs(3))
            .timeout_write(Duration::from_secs(3))
            .build();
        Self { base_url, agent }
    }

    pub fn dashboard(&self) -> Option<DashboardResponse> {
        self.agent
            .get(&format!("{}/api/dashboard", self.base_url))
            .call()
            .ok()?
            .into_json()
            .ok()
    }

    pub fn agents(&self) -> Option<AgentsResponse> {
        self.agent
            .get(&format!("{}/api/agents", self.base_url))
            .call()
            .ok()?
            .into_json()
            .ok()
    }

    pub fn convoys(&self) -> Option<ConvoysResponse> {
        self.agent
            .get(&format!("{}/api/convoys", self.base_url))
            .call()
            .ok()?
            .into_json()
            .ok()
    }

    pub fn beads(&self) -> Option<BeadsResponse> {
        self.agent
            .get(&format!("{}/api/beads", self.base_url))
            .call()
            .ok()?
            .into_json()
            .ok()
    }

    pub fn repos(&self) -> Option<ReposResponse> {
        self.agent
            .get(&format!("{}/api/repos", self.base_url))
            .call()
            .ok()?
            .into_json()
            .ok()
    }

    pub fn create_bead(&self, rig: &str, title: &str) -> Result<String, String> {
        let resp: ActionResponse = self
            .agent
            .post(&format!("{}/api/beads", self.base_url))
            .send_json(ureq::json!({
                "rig": rig,
                "title": title,
                "issue_type": "task",
                "priority": 2
            }))
            .map_err(|e| e.to_string())?
            .into_json()
            .map_err(|e| e.to_string())?;
        if resp.success {
            Ok(resp.message)
        } else {
            Err(resp.message)
        }
    }

    pub fn spawn_polecat(&self, rig: &str, bead_id: &str) -> Result<String, String> {
        let resp: ActionResponse = self
            .agent
            .post(&format!("{}/api/spawn", self.base_url))
            .send_json(ureq::json!({
                "rig": rig,
                "bead_id": bead_id,
            }))
            .map_err(|e| e.to_string())?
            .into_json()
            .map_err(|e| e.to_string())?;
        if resp.success {
            Ok(resp.message)
        } else {
            Err(resp.message)
        }
    }

    pub fn is_reachable(&self) -> bool {
        self.agent
            .get(&format!("{}/api/dashboard", self.base_url))
            .call()
            .is_ok()
    }
}
