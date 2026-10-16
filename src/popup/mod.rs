use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

/// Record of a blocked popup window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedPopupRecord {
    pub id: u64,
    pub target_url: String,
    pub source_url: String,
    pub source_domain: String,
    pub timestamp: chrono::DateTime<chrono::Local>,
    pub was_unsolicited: bool,
}

/// Active notification to display in the UI banner
#[derive(Debug, Clone)]
pub struct PopupNotification {
    pub id: u64,
    pub tab_id: usize,
    pub target_url: String,
    pub source_domain: String,
    pub count: usize,
}

/// Configuration and state for the integrated Pop-up Blocker
pub struct PopupBlocker {
    enabled: Mutex<bool>,
    /// Domains that are always allowed to open pop-ups (e.g. banking, SSO)
    allowlist: Mutex<HashSet<String>>,
    /// Timestamp of last user click/keystroke per tab (for user activation detection)
    last_user_interaction: Mutex<HashMap<usize, Instant>>,
    /// Allowed user gesture activation window (e.g. 1200 ms)
    activation_window: Duration,
    /// Counter of blocked pop-ups
    total_blocked: Mutex<usize>,
    /// History of blocked pop-ups
    history: Mutex<Vec<BlockedPopupRecord>>,
    /// Active banner notifications per tab
    pending_notifications: Mutex<HashMap<usize, PopupNotification>>,
    next_id: Mutex<u64>,
}

impl PopupBlocker {
    pub fn new() -> Self {
        let mut allowlist = HashSet::new();
        // Pre-allow common OAuth / SSO providers
        allowlist.insert("accounts.google.com".to_string());
        allowlist.insert("login.microsoftonline.com".to_string());
        allowlist.insert("github.com".to_string());
        allowlist.insert("appleid.apple.com".to_string());

        Self {
            enabled: Mutex::new(true),
            allowlist: Mutex::new(allowlist),
            last_user_interaction: Mutex::new(HashMap::new()),
            activation_window: Duration::from_millis(1500),
            total_blocked: Mutex::new(0),
            history: Mutex::new(Vec::new()),
            pending_notifications: Mutex::new(HashMap::new()),
            next_id: Mutex::new(1),
        }
    }

    /// Record a user interaction (click, keypress) in a given tab
    pub fn record_user_interaction(&self, tab_id: usize) {
        let mut map = self.last_user_interaction.lock().unwrap();
        map.insert(tab_id, Instant::now());
    }

    /// Determine whether a popup request should be allowed or blocked
    /// Returns `true` if allowed, `false` if blocked.
    pub fn evaluate_popup_request(
        &self,
        tab_id: usize,
        source_url: &str,
        target_url: &str,
    ) -> bool {
        if !*self.enabled.lock().unwrap() {
            return true;
        }

        let source_domain = url::Url::parse(source_url)
            .ok()
            .and_then(|u| u.host_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "unknown".to_string());

        // Check if the source domain is on the allowlist
        if self.is_allowlisted(&source_domain) {
            return true;
        }

        // Check if there was recent user activation in this tab
        let has_user_activation = {
            let map = self.last_user_interaction.lock().unwrap();
            if let Some(&last_time) = map.get(&tab_id) {
                last_time.elapsed() <= self.activation_window
            } else {
                false
            }
        };

        // If user explicitly clicked and triggered the popup, allow it
        if has_user_activation {
            return true;
        }

        // Otherwise: It is an unsolicited pop-up (ad, timer, redirect, pop-under)! BLOCK IT!
        self.record_blocked_popup(tab_id, source_url, target_url, &source_domain, true);
        false
    }

    fn record_blocked_popup(
        &self,
        tab_id: usize,
        source_url: &str,
        target_url: &str,
        source_domain: &str,
        was_unsolicited: bool,
    ) {
        let id = {
            let mut id_lock = self.next_id.lock().unwrap();
            let current = *id_lock;
            *id_lock += 1;
            current
        };

        *self.total_blocked.lock().unwrap() += 1;

        let record = BlockedPopupRecord {
            id,
            target_url: target_url.to_string(),
            source_url: source_url.to_string(),
            source_domain: source_domain.to_string(),
            timestamp: chrono::Local::now(),
            was_unsolicited,
        };

        let mut hist = self.history.lock().unwrap();
        if hist.len() >= 100 {
            hist.remove(0);
        }
        hist.push(record);

        // Update pending notification for this tab
        let mut notifs = self.pending_notifications.lock().unwrap();
        let count = notifs.get(&tab_id).map(|n| n.count + 1).unwrap_or(1);
        notifs.insert(
            tab_id,
            PopupNotification {
                id,
                tab_id,
                target_url: target_url.to_string(),
                source_domain: source_domain.to_string(),
                count,
            },
        );
    }

    pub fn get_pending_notification(&self, tab_id: usize) -> Option<PopupNotification> {
        self.pending_notifications.lock().unwrap().get(&tab_id).cloned()
    }

    pub fn dismiss_notification(&self, tab_id: usize) {
        self.pending_notifications.lock().unwrap().remove(&tab_id);
    }

    pub fn is_allowlisted(&self, domain: &str) -> bool {
        let list = self.allowlist.lock().unwrap();
        list.contains(domain)
            || domain
                .split('.')
                .enumerate()
                .any(|(i, _)| i > 0 && list.contains(&domain.split('.').skip(i).collect::<Vec<_>>().join(".")))
    }

    pub fn toggle_allowlist(&self, domain: &str) -> bool {
        let mut list = self.allowlist.lock().unwrap();
        if list.contains(domain) {
            list.remove(domain);
            false
        } else {
            list.insert(domain.to_string());
            true
        }
    }

    pub fn allow_always(&self, domain: &str) {
        self.allowlist.lock().unwrap().insert(domain.to_string());
    }

    pub fn set_enabled(&self, enabled: bool) {
        *self.enabled.lock().unwrap() = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        *self.enabled.lock().unwrap()
    }

    pub fn total_blocked(&self) -> usize {
        *self.total_blocked.lock().unwrap()
    }

    pub fn history(&self) -> Vec<BlockedPopupRecord> {
        self.history.lock().unwrap().clone()
    }
}

impl Default for PopupBlocker {
    fn default() -> Self {
        Self::new()
    }
}
