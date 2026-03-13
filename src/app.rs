use crate::api_client::{
    AgentInfo, ApiClient, BeadSummary, ConvoyInfo, DashboardStats, InfraStatus, RepoSummary,
    RigSummary,
};

pub const TAB_COUNT: usize = 5;
pub const TAB_NAMES: [&str; TAB_COUNT] = ["Dashboard", "Agents", "Convoys", "Beads", "Repos"];

const DEFAULT_API_URL: &str = "http://localhost:3333";

pub struct App {
    pub tab: usize,
    pub scroll: usize,
    pub max_scroll: usize,

    // API connection
    pub api: ApiClient,
    pub api_url: String,
    pub connected: bool,

    // Dashboard data
    pub town_name: String,
    pub overseer_name: String,
    pub stats: DashboardStats,
    pub infra: InfraStatus,

    // List data
    pub rigs: Vec<RigSummary>,
    pub agents: Vec<AgentInfo>,
    pub convoys: Vec<ConvoyInfo>,
    pub beads: Vec<BeadSummary>,
    pub repos: Vec<RepoSummary>,

    // Filter
    pub filter_active: bool,
    pub filter_text: String,

    // Spawn dialog
    pub show_spawn_dialog: bool,
    pub spawn_field: usize,
    pub spawn_rig: String,
    pub spawn_runtime_idx: usize,
    pub spawn_task: String,

    // Status message
    pub status_msg: Option<String>,

    // Animation tick
    pub tick: u64,
}

const RUNTIMES: [&str; 5] = ["claude", "gemini", "codex", "cursor", "copilot"];

impl App {
    pub fn new() -> Self {
        let api_url =
            std::env::var("GASTOWN_API").unwrap_or_else(|_| DEFAULT_API_URL.to_string());
        let api = ApiClient::new(api_url.clone());

        Self {
            tab: 0,
            scroll: 0,
            max_scroll: 0,
            api,
            api_url: api_url.clone(),
            connected: false,
            town_name: String::new(),
            overseer_name: String::new(),
            stats: DashboardStats::default(),
            infra: InfraStatus::default(),
            rigs: Vec::new(),
            agents: Vec::new(),
            convoys: Vec::new(),
            beads: Vec::new(),
            repos: Vec::new(),
            filter_active: false,
            filter_text: String::new(),
            show_spawn_dialog: false,
            spawn_field: 0,
            spawn_rig: String::new(),
            spawn_runtime_idx: 0,
            spawn_task: String::new(),
            status_msg: Some(format!("Connecting to API at {api_url}...")),
            tick: 0,
        }
    }

    pub fn refresh_all(&mut self) {
        if let Some(d) = self.api.dashboard() {
            self.connected = true;
            self.town_name = d.town_name;
            self.overseer_name = d.overseer;
            self.stats = d.stats;
            self.infra = d.infrastructure;
            self.rigs = d.rigs;
            // Clear the initial connecting message
            if self
                .status_msg
                .as_deref()
                .is_some_and(|m| m.starts_with("Connecting to API") || m.starts_with("Cannot reach API"))
            {
                self.status_msg = None;
            }
        } else {
            self.connected = false;
            self.status_msg = Some(format!("Cannot reach API at {}", self.api_url));
            return;
        }

        if let Some(a) = self.api.agents() {
            self.agents = a.agents;
        }
        if let Some(c) = self.api.convoys() {
            self.convoys = c.convoys;
        }
        if let Some(b) = self.api.beads() {
            self.beads = b.beads;
        }
        if let Some(r) = self.api.repos() {
            self.repos = r.repos;
        }
    }

    pub fn on_tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
    }

    // ── Derived stats ────────────────────────────────────────────────

    pub fn agents_running(&self) -> usize {
        self.stats.agents_running
    }

    pub fn agents_with_work(&self) -> usize {
        self.agents.iter().filter(|a| a.has_work).count()
    }

    // ── Navigation ───────────────────────────────────────────────────

    pub fn next_tab(&mut self) {
        self.tab = (self.tab + 1) % TAB_COUNT;
        self.scroll = 0;
    }

    pub fn prev_tab(&mut self) {
        self.tab = if self.tab == 0 {
            TAB_COUNT - 1
        } else {
            self.tab - 1
        };
        self.scroll = 0;
    }

    pub fn select_tab(&mut self, idx: usize) {
        if idx < TAB_COUNT {
            self.tab = idx;
            self.scroll = 0;
        }
    }

    pub fn scroll_down(&mut self) {
        if self.scroll < self.max_scroll {
            self.scroll += 1;
        }
    }

    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }

    pub fn scroll_top(&mut self) {
        self.scroll = 0;
    }

    pub fn scroll_bottom(&mut self) {
        self.scroll = self.max_scroll;
    }

    // ── Filter ───────────────────────────────────────────────────────

    pub fn is_filtering(&self) -> bool {
        self.filter_active
    }

    pub fn toggle_filter(&mut self) {
        self.filter_active = !self.filter_active;
        if !self.filter_active {
            self.filter_text.clear();
        }
    }

    pub fn filter_push(&mut self, c: char) {
        self.filter_text.push(c);
        self.scroll = 0;
    }

    pub fn filter_pop(&mut self) {
        self.filter_text.pop();
        self.scroll = 0;
    }

    pub fn dismiss(&mut self) {
        if self.show_spawn_dialog {
            self.show_spawn_dialog = false;
        } else if self.filter_active {
            self.filter_active = false;
            self.filter_text.clear();
        }
        self.status_msg = None;
    }

    // ── Spawn dialog ─────────────────────────────────────────────────

    pub fn action_spawn(&mut self) {
        self.show_spawn_dialog = true;
        self.spawn_field = 0;
        self.spawn_rig.clear();
        self.spawn_runtime_idx = 0;
        self.spawn_task.clear();
    }

    pub fn spawn_next_field(&mut self) {
        self.spawn_field = (self.spawn_field + 1) % 3;
    }

    pub fn spawn_type_char(&mut self, c: char) {
        match self.spawn_field {
            0 => self.spawn_rig.push(c),
            2 => self.spawn_task.push(c),
            _ => {}
        }
    }

    pub fn spawn_backspace(&mut self) {
        match self.spawn_field {
            0 => { self.spawn_rig.pop(); }
            2 => { self.spawn_task.pop(); }
            _ => {}
        }
    }

    pub fn spawn_next_option(&mut self) {
        if self.spawn_field == 1 {
            self.spawn_runtime_idx = (self.spawn_runtime_idx + 1) % RUNTIMES.len();
        }
    }

    pub fn spawn_prev_option(&mut self) {
        if self.spawn_field == 1 {
            self.spawn_runtime_idx = if self.spawn_runtime_idx == 0 {
                RUNTIMES.len() - 1
            } else {
                self.spawn_runtime_idx - 1
            };
        }
    }

    pub fn spawn_runtime(&self) -> &str {
        RUNTIMES[self.spawn_runtime_idx]
    }

    pub fn confirm_spawn(&mut self) {
        let rig = self.spawn_rig.clone();
        let task = self.spawn_task.clone();
        if rig.is_empty() || task.is_empty() {
            self.status_msg = Some("Rig and task are required".into());
            return;
        }
        match self.api.create_bead(&rig, &task) {
            Ok(bead_id) => match self.api.spawn_polecat(&rig, &bead_id) {
                Ok(_) => {
                    self.status_msg = Some(format!("Spawned: {bead_id} in {rig}"));
                }
                Err(e) => {
                    self.status_msg = Some(format!("Created {bead_id}, sling failed: {e}"));
                }
            },
            Err(e) => {
                self.status_msg = Some(format!("Create failed: {e}"));
            }
        }
        self.show_spawn_dialog = false;
        self.refresh_all();
    }

    pub fn action_enter(&mut self) {}

    // ── Filtered data accessors ──────────────────────────────────────

    pub fn filtered_agents(&self) -> Vec<&AgentInfo> {
        let f = self.filter_text.to_lowercase();
        self.agents
            .iter()
            .filter(|a| {
                f.is_empty()
                    || a.name.to_lowercase().contains(&f)
                    || a.address.to_lowercase().contains(&f)
                    || a.role.as_deref().unwrap_or("").to_lowercase().contains(&f)
                    || a.state.as_deref().unwrap_or("").to_lowercase().contains(&f)
            })
            .collect()
    }

    pub fn filtered_convoys(&self) -> Vec<&ConvoyInfo> {
        let f = self.filter_text.to_lowercase();
        self.convoys
            .iter()
            .filter(|c| {
                f.is_empty()
                    || c.title.to_lowercase().contains(&f)
                    || c.id.to_lowercase().contains(&f)
                    || c.status.to_lowercase().contains(&f)
            })
            .collect()
    }

    pub fn filtered_beads(&self) -> Vec<&BeadSummary> {
        let f = self.filter_text.to_lowercase();
        self.beads
            .iter()
            .filter(|b| {
                f.is_empty()
                    || b.title.to_lowercase().contains(&f)
                    || b.id.to_lowercase().contains(&f)
                    || b.status.to_lowercase().contains(&f)
                    || b.issue_type.to_lowercase().contains(&f)
                    || b.rig.to_lowercase().contains(&f)
            })
            .collect()
    }

    pub fn filtered_repos(&self) -> Vec<&RepoSummary> {
        let f = self.filter_text.to_lowercase();
        self.repos
            .iter()
            .filter(|r| {
                f.is_empty()
                    || r.name.to_lowercase().contains(&f)
                    || r.branch.to_lowercase().contains(&f)
                    || r.path.to_lowercase().contains(&f)
            })
            .collect()
    }
}
