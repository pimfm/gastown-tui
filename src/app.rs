use crate::gastown::{Agent, AgentStatus, Bead, BeadStatus, Convoy, ConvoyStatus, FeedEntry};
use crate::repos::RepoInfo;

pub const TAB_COUNT: usize = 5;
pub const TAB_NAMES: [&str; TAB_COUNT] = ["Dashboard", "Agents", "Convoys", "Beads", "Repos"];

pub struct App {
    pub tab: usize,
    pub scroll: usize,
    pub max_scroll: usize,

    // Data
    pub agents: Vec<Agent>,
    pub convoys: Vec<Convoy>,
    pub beads: Vec<Bead>,
    pub feed: Vec<FeedEntry>,
    pub repos: Vec<RepoInfo>,

    // Derived stats
    pub agents_active: usize,
    pub agents_idle: usize,
    pub agents_stuck: usize,
    pub agents_offline: usize,
    pub total_beads_done: u32,
    pub total_beads_open: u32,
    pub total_beads_progress: u32,
    pub total_beads_blocked: u32,

    // Filter
    pub filter_active: bool,
    pub filter_text: String,

    // Spawn dialog
    pub show_spawn_dialog: bool,
    pub spawn_field: usize, // 0=rig, 1=runtime, 2=task
    pub spawn_rig: String,
    pub spawn_runtime_idx: usize,
    pub spawn_task: String,

    // Animation tick
    pub tick: u64,

    // Status
    pub gt_available: bool,
    pub bd_available: bool,
}

const RUNTIMES: [&str; 5] = ["claude", "gemini", "codex", "cursor", "copilot"];

impl App {
    pub fn new() -> Self {
        Self {
            tab: 0,
            scroll: 0,
            max_scroll: 0,
            agents: Vec::new(),
            convoys: Vec::new(),
            beads: Vec::new(),
            feed: Vec::new(),
            repos: Vec::new(),
            agents_active: 0,
            agents_idle: 0,
            agents_stuck: 0,
            agents_offline: 0,
            total_beads_done: 0,
            total_beads_open: 0,
            total_beads_progress: 0,
            total_beads_blocked: 0,
            filter_active: false,
            filter_text: String::new(),
            show_spawn_dialog: false,
            spawn_field: 0,
            spawn_rig: String::new(),
            spawn_runtime_idx: 0,
            spawn_task: String::new(),
            tick: 0,
            gt_available: false,
            bd_available: false,
        }
    }

    pub fn refresh_all(&mut self) {
        self.gt_available = crate::gastown::gt_available();
        self.bd_available = crate::gastown::bd_available();
        self.agents = crate::gastown::fetch_agents();
        self.convoys = crate::gastown::fetch_convoys();
        self.beads = crate::gastown::fetch_beads();
        self.feed = crate::gastown::fetch_feed();
        self.repos = crate::repos::scan_repos();
        self.recompute_stats();
    }

    fn recompute_stats(&mut self) {
        self.agents_active = self
            .agents
            .iter()
            .filter(|a| a.status == AgentStatus::Active)
            .count();
        self.agents_idle = self
            .agents
            .iter()
            .filter(|a| a.status == AgentStatus::Idle)
            .count();
        self.agents_stuck = self
            .agents
            .iter()
            .filter(|a| a.status == AgentStatus::Stuck)
            .count();
        self.agents_offline = self
            .agents
            .iter()
            .filter(|a| a.status == AgentStatus::Offline)
            .count();

        self.total_beads_done = self
            .beads
            .iter()
            .filter(|b| b.status == BeadStatus::Done)
            .count() as u32;
        self.total_beads_open = self
            .beads
            .iter()
            .filter(|b| b.status == BeadStatus::Open)
            .count() as u32;
        self.total_beads_progress = self
            .beads
            .iter()
            .filter(|b| b.status == BeadStatus::InProgress)
            .count() as u32;
        self.total_beads_blocked = self
            .beads
            .iter()
            .filter(|b| b.status == BeadStatus::Blocked)
            .count() as u32;
    }

    pub fn on_tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
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
            0 => {
                self.spawn_rig.pop();
            }
            2 => {
                self.spawn_task.pop();
            }
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
        // In a real setup this would call `gt sling` or spawn a new agent process
        self.show_spawn_dialog = false;
    }

    pub fn action_enter(&mut self) {
        // Placeholder for detail view / drill-down
    }

    // ── Filtered data accessors ──────────────────────────────────────

    pub fn filtered_agents(&self) -> Vec<&Agent> {
        let f = self.filter_text.to_lowercase();
        self.agents
            .iter()
            .filter(|a| {
                f.is_empty()
                    || a.name.to_lowercase().contains(&f)
                    || a.rig.to_lowercase().contains(&f)
                    || a.runtime.to_lowercase().contains(&f)
                    || a.status.label().to_lowercase().contains(&f)
            })
            .collect()
    }

    pub fn filtered_convoys(&self) -> Vec<&Convoy> {
        let f = self.filter_text.to_lowercase();
        self.convoys
            .iter()
            .filter(|c| {
                f.is_empty()
                    || c.name.to_lowercase().contains(&f)
                    || c.id.to_lowercase().contains(&f)
                    || c.status.label().to_lowercase().contains(&f)
            })
            .collect()
    }

    pub fn filtered_beads(&self) -> Vec<&Bead> {
        let f = self.filter_text.to_lowercase();
        self.beads
            .iter()
            .filter(|b| {
                f.is_empty()
                    || b.title.to_lowercase().contains(&f)
                    || b.id.to_lowercase().contains(&f)
                    || b.rig.to_lowercase().contains(&f)
                    || b.status.label().to_lowercase().contains(&f)
            })
            .collect()
    }

    pub fn filtered_repos(&self) -> Vec<&RepoInfo> {
        let f = self.filter_text.to_lowercase();
        self.repos
            .iter()
            .filter(|r| {
                f.is_empty()
                    || r.name.to_lowercase().contains(&f)
                    || r.branch.to_lowercase().contains(&f)
                    || r.path.to_string_lossy().to_lowercase().contains(&f)
            })
            .collect()
    }

    pub fn active_convoys(&self) -> Vec<&Convoy> {
        self.convoys
            .iter()
            .filter(|c| c.status == ConvoyStatus::Active)
            .collect()
    }
}
