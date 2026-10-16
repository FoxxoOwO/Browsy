use std::collections::HashMap;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snippet {
    pub id: u64,
    pub trigger: String,
    pub replacement: String,
    pub description: String,
    pub enabled: bool,
}

pub struct TextExpander {
    snippets: Mutex<Vec<Snippet>>,
    next_id: Mutex<u64>,
}

impl TextExpander {
    pub fn new() -> Self {
        let default_snippets = vec![
            Snippet {
                id: 1,
                trigger: ":email".to_string(),
                replacement: "user@example.com".to_string(),
                description: "Personal email address".to_string(),
                enabled: true,
            },
            Snippet {
                id: 2,
                trigger: ":date".to_string(),
                replacement: "{date}".to_string(),
                description: "Current date (YYYY-MM-DD)".to_string(),
                enabled: true,
            },
            Snippet {
                id: 3,
                trigger: ":time".to_string(),
                replacement: "{time}".to_string(),
                description: "Current time (HH:MM:SS)".to_string(),
                enabled: true,
            },
            Snippet {
                id: 4,
                trigger: ":shrug".to_string(),
                replacement: r#"¯\_(ツ)_/¯"#.to_string(),
                description: "Shrug emoji".to_string(),
                enabled: true,
            },
            Snippet {
                id: 5,
                trigger: ":br".to_string(),
                replacement: "Best regards,\nJohn Doe".to_string(),
                description: "Email sign-off".to_string(),
                enabled: true,
            },
            Snippet {
                id: 6,
                trigger: ":lipsum".to_string(),
                replacement: "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.".to_string(),
                description: "Lorem ipsum sample text".to_string(),
                enabled: true,
            },
            Snippet {
                id: 7,
                trigger: ":gh".to_string(),
                replacement: "https://github.com/".to_string(),
                description: "GitHub URL prefix".to_string(),
                enabled: true,
            },
        ];

        Self {
            snippets: Mutex::new(default_snippets),
            next_id: Mutex::new(8),
        }
    }

    /// List all snippets
    pub fn all_snippets(&self) -> Vec<Snippet> {
        self.snippets.lock().unwrap().clone()
    }

    /// Add a new snippet
    pub fn add_snippet(&self, trigger: &str, replacement: &str, description: &str) -> u64 {
        let id = {
            let mut id_lock = self.next_id.lock().unwrap();
            let cur = *id_lock;
            *id_lock += 1;
            cur
        };

        let snippet = Snippet {
            id,
            trigger: trigger.trim().to_string(),
            replacement: replacement.to_string(),
            description: description.trim().to_string(),
            enabled: true,
        };

        self.snippets.lock().unwrap().push(snippet);
        id
    }

    /// Remove a snippet by id
    pub fn remove_snippet(&self, id: u64) {
        let mut list = self.snippets.lock().unwrap();
        list.retain(|s| s.id != id);
    }

    /// Toggle snippet enabled state
    pub fn toggle_enabled(&self, id: u64) {
        let mut list = self.snippets.lock().unwrap();
        if let Some(s) = list.iter_mut().find(|s| s.id == id) {
            s.enabled = !s.enabled;
        }
    }

    /// Evaluate dynamic variables in snippet replacement
    pub fn expand_variables(&self, template: &str) -> String {
        let now = chrono::Local::now();
        let date_str = now.format("%Y-%m-%d").to_string();
        let time_str = now.format("%H:%M:%S").to_string();
        let datetime_str = now.format("%Y-%m-%d %H:%M:%S").to_string();

        template
            .replace("{date}", &date_str)
            .replace("{time}", &time_str)
            .replace("{datetime}", &datetime_str)
    }

    /// Check if a text input ends with an active snippet trigger and expand it
    /// Returns Some(expanded_full_text) if a trigger matched, None otherwise.
    pub fn try_expand_input(&self, current_text: &str) -> Option<String> {
        let list = self.snippets.lock().unwrap();
        for snippet in list.iter() {
            if !snippet.enabled || snippet.trigger.is_empty() {
                continue;
            }

            if current_text.ends_with(&snippet.trigger) {
                let prefix_len = current_text.len() - snippet.trigger.len();
                let expanded_val = self.expand_variables(&snippet.replacement);
                let mut result = current_text[..prefix_len].to_string();
                result.push_str(&expanded_val);
                return Some(result);
            }
        }
        None
    }

    /// Generates the client-side JavaScript content script injected into web pages
    /// to handle text expansion directly in webpage forms and textareas!
    pub fn generate_content_script(&self) -> String {
        let list = self.snippets.lock().unwrap();
        let mut snippet_map: HashMap<String, String> = HashMap::new();
        for s in list.iter() {
            if s.enabled {
                snippet_map.insert(s.trigger.clone(), self.expand_variables(&s.replacement));
            }
        }

        let json_snippets = serde_json::to_string(&snippet_map).unwrap_or_else(|_| "{}".to_string());

        format!(
            r#"(function() {{
    const snippets = {json_snippets};
    let buffer = "";

    function checkAndExpand(target) {{
        if (!target || !("value" in target)) return;
        const val = target.value;
        for (const [trigger, replacement] of Object.entries(snippets)) {{
            if (val.endsWith(trigger)) {{
                const start = target.selectionStart;
                const end = target.selectionEnd;
                const before = val.slice(0, val.length - trigger.length);
                target.value = before + replacement;
                const newPos = before.length + replacement.length;
                if (target.setSelectionRange) {{
                    target.setSelectionRange(newPos, newPos);
                }}
                target.dispatchEvent(new Event('input', {{ bubbles: true }}));
                break;
            }}
        }}
    }}

    document.addEventListener('input', function(e) {{
        checkAndExpand(e.target);
    }}, true);

    console.log("[Browsy] Text Expander active with " + Object.keys(snippets).length + " snippets.");
}})();"#
        )
    }
}

impl Default for TextExpander {
    fn default() -> Self {
        Self::new()
    }
}
