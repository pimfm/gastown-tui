use std::path::PathBuf;

#[allow(unused_imports)]
use crate::gastown::{
    BdIssue, GtAgent, GtConvoy, GtMail, GtRig, TownSummary,
};
use crate::repos::RepoInfo;

pub const TAB_COUNT: usize = 5;
pub const TAB_NAMES: [&str; TAB_COUNT] = ["Dashboard", "Agents", "Convoys", "Beads", "Repos"];

pub struct App {
    pub tab: usize,
    pub scroll: usize,
    pub max_scroll: usize,

    // Gas Town workspace
    pub gt_root: Option<PathBuf>,
    pub gt_available: bool,
    pub bd_available: bool,

    // Real data from Gas Town
    pub town_name: String,
    pub overseer_name: String,
    pub unread_mail: u32,
    pub daemon_running: bool,
    pub dolt_running: bool,
    pub tmux_running: bool,
    pub tmux_sessions: u32,

    pub agents: Vec<GtAgent>,
    pub rigs: Vec<GtRig>,
    pub summary: TownSummary,
    pub convoys: Vec<GtConvoy>,
    pub all_beads: Vec<(String, BdIssue)>, // (rig_name, issue)
    pub mail: Vec<GtMail>,
    pub repos: Vec<RepoInfo>,

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

impl Default for TownSummary {
    fn default() -> Self {
        Self {
            rig_count: 0,
            polecat_count: 0,
            crew_count: 0,
            witness_count: 0,
            refinery_count: 0,
            active_hooks: 0,
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            tab: 0,
            scroll: 0,
            max_scroll: 0,
            gt_root: crate::gastown::find_gt_root(),
            gt_available: crate::gastown::gt_available(),
            bd_available: crate::gastown::bd_available(),
            town_name: String::new(),
            overseer_name: String::new(),
            unread_mail: 0,
            daemon_running: false,
            dolt_running: false,
            tmux_running: false,
            tmux_sessions: 0,
            agents: Vec::new(),
            rigs: Vec::new(),
            summary: TownSummary::default(),
            convoys: Vec::new(),
            all_beads: Vec::new(),
            mail: Vec::new(),
            repos: Vec::new(),
            filter_active: false,
            filter_text: String::new(),
            show_spawn_dialog: false,
            spawn_field: 0,
            spawn_rig: String::new(),
            spawn_runtime_idx: 0,
            spawn_task: String::new(),
            status_msg: None,
            tick: 0,
        }
    }

    pub fn refresh_all(&mut self) {
        self.gt_available = crate::gastown::gt_available();
        self.bd_available = crate::gastown::bd_available();
        self.gt_root = crate::gastown::find_gt_root();

        if let Some(root) = &self.gt_root {
            let root = root.clone();
            self.refresh_status(&root);
            self.refresh_convoys(&root);
            self.refresh_beads(&root);
            self.refresh_mail(&root);
        }
        self.repos = crate::repos::scan_repos();
    }

    fn refresh_status(&mut self, root: &PathBuf) {
        if let Some(status) = crate::gastown::fetch_status(root) {
            self.town_name = status.name;
            if let Some(ref overseer) = status.overseer {
                self.overseer_name = overseer.name.clone();
                self.unread_mail = overseer.unread_mail;
            }
            if let Some(ref daemon) = status.daemon {
                self.daemon_running = daemon.running;
            }
            if let Some(ref dolt) = status.dolt {
                self.dolt_running = dolt.running;
            }
            if let Some(ref tmux) = status.tmux {
                self.tmux_running = tmux.running;
                self.tmux_sessions = tmux.session_count;
            }
            // Flatten all agents: town-level + per-rig
            self.agents.clear();
            self.agents.extend(status.agents);
            for rig in &status.rigs {
                for agent in &rig.agents {
                    let mut a = agent.clone();
                    if !a.address.contains('/') {
                        a.address = format!("{}/{}", rig.name, a.name);
                    }
                    self.agents.push(a);
                }
            }
            self.rigs = status.rigs;
            if let Some(summary) = status.summary {
                self.summary = summary;
            }
        }
    }

    fn refresh_convoys(&mut self, root: &PathBuf) {
        self.convoys = crate::gastown::fetch_convoys(root);
    }

    fn refresh_beads(&mut self, root: &PathBuf) {
        self.all_beads.clear();
        for issue in crate::gastown::fetch_hq_beads(root) {
            self.all_beads.push(("hq".to_string(), issue));
        }
        let rig_names: Vec<String> = self.rigs.iter().map(|r| r.name.clone()).collect();
        for rig_name in &rig_names {
            for issue in crate::gastown::fetch_beads_for_rig(root, rig_name) {
                self.all_beads.push((rig_name.clone(), issue));
            }
        }
    }

    fn refresh_mail(&mut self, root: &PathBuf) {
        self.mail = crate::gastown::fetch_mail(root);
    }

    pub fn on_tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
    }

    // ── Derived stats ────────────────────────────────────────────────

    pub fn agents_running(&self) -> usize {
        self.agents.iter().filter(|a| a.running).count()
    }

    pub fn agents_with_work(&self) -> usize {
        self.agents.iter().filter(|a| a.has_work).count()
    }

    pub fn user_beads(&self) -> Vec<&(String, BdIssue)> {
        self.all_beads
            .iter()
            .filter(|(_, b)| {
                b.issue_type != "molecule"
                    && !b.labels.as_ref().is_some_and(|l| l.iter().any(|l| l.starts_with("gt:")))
                    && !b.id.contains("-rig-")
                    && !b.title.ends_with("Patrol")
                    && !b.id.contains("-witness")
                    && !b.id.contains("-refinery")
            })
            .collect()
    }

    pub fn beads_by_status(&self, status: &str) -> usize {
        self.user_beads()
            .iter()
            .filter(|(_, b)| b.status == status)
            .count()
    }

    pub fn active_convoys(&self) -> Vec<&GtConvoy> {
        self.convoys.iter().filter(|c| c.status == "open").collect()
    }

    // ── Navigation ───────────────────────────────────────────────────

    pub fn next_tab(&mut self) {
        self.tab = (self.tab + 1) % TAB_COUNT;
        self.scroll = 0;
    }

    pub fn prev_tab(&mut self) {
        self.tab = if self.tab == 0 { TAB_COUNT - 1 } else { self.tab - 1 };
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
        if let Some(root) = &self.gt_root {
            let root = root.clone();
            let rig = self.spawn_rig.clone();
            let task = self.spawn_task.clone();
            if rig.is_empty() || task.is_empty() {
                self.status_msg = Some("Rig and task are required".into());
                return;
            }
            match crate::gastown::create_bead(&root, &rig, &task, "task", 2) {
                Ok(bead_id) => {
                    match crate::gastown::spawn_polecat(&root, &rig, &bead_id) {
                        Ok(_) => {
                            self.status_msg = Some(format!("Spawned: {bead_id} in {rig}"));
                        }
                        Err(e) => {
                            self.status_msg = Some(format!("Created {bead_id}, sling failed: {e}"));
                        }
                    }
                }
                Err(e) => {
                    self.status_msg = Some(format!("Create failed: {e}"));
                }
            }
        } else {
            self.status_msg = Some("No Gas Town workspace found at ~/gt".into());
        }
        self.show_spawn_dialog = false;
        self.refresh_all();
    }

    pub fn action_enter(&mut self) {}

    // ── Filtered data accessors ──────────────────────────────────────

    pub fn filtered_agents(&self) -> Vec<&GtAgent> {
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

    pub fn filtered_convoys(&self) -> Vec<&GtConvoy> {
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

    pub fn filtered_beads(&self) -> Vec<&(String, BdIssue)> {
        let f = self.filter_text.to_lowercase();
        self.user_beads()
            .into_iter()
            .filter(|(rig, b)| {
                f.is_empty()
                    || b.title.to_lowercase().contains(&f)
                    || b.id.to_lowercase().contains(&f)
                    || b.status.to_lowercase().contains(&f)
                    || b.issue_type.to_lowercase().contains(&f)
                    || rig.to_lowercase().contains(&f)
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
}
