use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TabLayout {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tab {
    pub id: usize,
    pub url: String,
    pub title: String,
    pub favicon: Option<String>,
    pub is_pinned: bool,
    pub is_muted: bool,
    pub is_loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub history: Vec<String>,
    pub history_idx: usize,
    pub blocked_ads: usize,
    pub detected_videos: usize,
    pub blocked_popups: usize,
}

impl Tab {
    pub fn new(id: usize, url: &str) -> Self {
        let initial_url = if url.trim().is_empty() {
            "browsy://welcome".to_string()
        } else {
            url.trim().to_string()
        };

        let title = if initial_url == "browsy://welcome" {
            "Browsy - Welcome".to_string()
        } else {
            initial_url.clone()
        };

        Self {
            id,
            url: initial_url.clone(),
            title,
            favicon: None,
            is_pinned: false,
            is_muted: false,
            is_loading: false,
            can_go_back: false,
            can_go_forward: false,
            history: vec![initial_url],
            history_idx: 0,
            blocked_ads: 0,
            detected_videos: 0,
            blocked_popups: 0,
        }
    }

    pub fn navigate(&mut self, url: &str) {
        let normalized = normalize_url(url);
        self.url = normalized.clone();
        self.title = normalized.clone();
        self.is_loading = true;

        if self.history_idx + 1 < self.history.len() {
            self.history.truncate(self.history_idx + 1);
        }
        self.history.push(normalized);
        self.history_idx = self.history.len() - 1;

        self.can_go_back = self.history_idx > 0;
        self.can_go_forward = false;
    }

    pub fn go_back(&mut self) -> Option<&str> {
        if self.history_idx > 0 {
            self.history_idx -= 1;
            self.url = self.history[self.history_idx].clone();
            self.title = self.url.clone();
            self.can_go_back = self.history_idx > 0;
            self.can_go_forward = self.history_idx + 1 < self.history.len();
            Some(&self.url)
        } else {
            None
        }
    }

    pub fn go_forward(&mut self) -> Option<&str> {
        if self.history_idx + 1 < self.history.len() {
            self.history_idx += 1;
            self.url = self.history[self.history_idx].clone();
            self.title = self.url.clone();
            self.can_go_back = self.history_idx > 0;
            self.can_go_forward = self.history_idx + 1 < self.history.len();
            Some(&self.url)
        } else {
            None
        }
    }
}

pub struct TabManager {
    pub tabs: Vec<Tab>,
    pub active_tab_id: usize,
    pub layout: TabLayout,
    pub sidebar_expanded: bool,
    pub sidebar_width: f32,
    pub tab_filter: String,
    next_tab_id: usize,
}

impl TabManager {
    pub fn new() -> Self {
        let mut manager = Self {
            tabs: Vec::new(),
            active_tab_id: 1,
            layout: TabLayout::Horizontal,
            sidebar_expanded: true,
            sidebar_width: 220.0,
            tab_filter: String::new(),
            next_tab_id: 1,
        };

        manager.new_tab("browsy://welcome");
        manager
    }

    pub fn new_tab(&mut self, url: &str) -> usize {
        let id = self.next_tab_id;
        self.next_tab_id += 1;

        let tab = Tab::new(id, url);
        self.tabs.push(tab);
        self.active_tab_id = id;
        id
    }

    pub fn close_tab(&mut self, id: usize) {
        if self.tabs.len() <= 1 {
            // Keep at least one tab open
            if let Some(tab) = self.tabs.first_mut() {
                *tab = Tab::new(self.next_tab_id, "browsy://welcome");
                self.next_tab_id += 1;
                self.active_tab_id = tab.id;
            }
            return;
        }

        let idx = self.tabs.iter().position(|t| t.id == id);
        if let Some(i) = idx {
            self.tabs.remove(i);
            if self.active_tab_id == id {
                // Select adjacent tab
                let new_idx = if i >= self.tabs.len() {
                    self.tabs.len() - 1
                } else {
                    i
                };
                self.active_tab_id = self.tabs[new_idx].id;
            }
        }
    }

    pub fn active_tab(&self) -> Option<&Tab> {
        self.tabs.iter().find(|t| t.id == self.active_tab_id)
    }

    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.tabs.iter_mut().find(|t| t.id == self.active_tab_id)
    }

    pub fn select_tab(&mut self, id: usize) {
        if self.tabs.iter().any(|t| t.id == id) {
            self.active_tab_id = id;
        }
    }

    pub fn toggle_pinned(&mut self, id: usize) {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
            tab.is_pinned = !tab.is_pinned;
        }
    }

    pub fn toggle_layout(&mut self) {
        self.layout = match self.layout {
            TabLayout::Horizontal => TabLayout::Vertical,
            TabLayout::Vertical => TabLayout::Horizontal,
        };
    }

    pub fn toggle_sidebar(&mut self) {
        self.sidebar_expanded = !self.sidebar_expanded;
    }
}

impl Default for TabManager {
    fn default() -> Self {
        Self::new()
    }
}

pub fn normalize_url(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return "browsy://welcome".to_string();
    }
    if trimmed.starts_with("browsy://") || trimmed.starts_with("about:") {
        return trimmed.to_string();
    }
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") || trimmed.starts_with("file://") {
        return trimmed.to_string();
    }
    if trimmed.contains('.') && !trimmed.contains(' ') {
        return format!("https://{}", trimmed);
    }
    // Search query fallback (DuckDuckGo / Google)
    let encoded = urlencoding_simple(trimmed);
    format!("https://duckduckgo.com/?q={}", encoded)
}

fn urlencoding_simple(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => "+".to_string(),
            c if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '~' => c.to_string(),
            c => format!("%{:02X}", c as u32),
        })
        .collect()
}
