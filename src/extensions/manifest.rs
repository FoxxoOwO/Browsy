use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentScriptDef {
    pub matches: Vec<String>,
    #[serde(default)]
    pub js: Vec<String>,
    #[serde(default)]
    pub css: Vec<String>,
    #[serde(default)]
    pub run_at: Option<String>,
    #[serde(default)]
    pub all_frames: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionDef {
    pub default_title: Option<String>,
    pub default_popup: Option<String>,
    pub default_icon: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundDef {
    pub scripts: Option<Vec<String>>,
    pub service_worker: Option<String>,
    #[serde(rename = "type")]
    pub worker_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionManifest {
    pub manifest_version: u32,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub host_permissions: Vec<String>,
    #[serde(default)]
    pub content_scripts: Vec<ContentScriptDef>,
    #[serde(default)]
    pub background: Option<BackgroundDef>,
    #[serde(default)]
    pub action: Option<ActionDef>,
    #[serde(default)]
    pub browser_action: Option<ActionDef>,
    #[serde(default)]
    pub icons: HashMap<String, String>,
}

impl ExtensionManifest {
    pub fn parse_str(content: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(content)
    }

    pub fn parse_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let manifest = Self::parse_str(&content)?;
        Ok(manifest)
    }

    pub fn get_popup_file(&self) -> Option<String> {
        if let Some(act) = &self.action {
            if act.default_popup.is_some() {
                return act.default_popup.clone();
            }
        }
        if let Some(bact) = &self.browser_action {
            if bact.default_popup.is_some() {
                return bact.default_popup.clone();
            }
        }
        None
    }
}
