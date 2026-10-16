pub mod manifest;
pub mod matcher;
pub mod polyfill;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use manifest::ExtensionManifest;
use matcher::PatternMatcher;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: Option<String>,
    pub enabled: bool,
    pub permissions: Vec<String>,
    pub has_popup: bool,
    pub popup_file: Option<String>,
    pub root_path: PathBuf,
}

pub struct Extension {
    pub manifest: ExtensionManifest,
    pub root_path: PathBuf,
    pub enabled: bool,
    pub matchers: Vec<(PatternMatcher, manifest::ContentScriptDef)>,
}

pub struct ExtensionManager {
    extensions: Mutex<HashMap<String, Extension>>,
    extensions_dir: PathBuf,
}

impl ExtensionManager {
    pub fn new() -> Self {
        let ext_dir = dirs_or_fallback_ext_dir();
        let _ = std::fs::create_dir_all(&ext_dir);

        let manager = Self {
            extensions: Mutex::new(HashMap::new()),
            extensions_dir: ext_dir,
        };

        // Create a built-in sample demo WebExtension so the user has immediate proof of extension support
        manager.create_sample_extension_if_empty();
        manager.scan_extensions_dir();

        manager
    }

    /// Load an unpacked extension from a folder containing `manifest.json`
    pub fn load_unpacked(&self, path: &Path) -> Result<String, String> {
        let manifest_path = path.join("manifest.json");
        if !manifest_path.exists() {
            return Err("manifest.json not found in directory".to_string());
        }

        let manifest = ExtensionManifest::parse_file(&manifest_path)
            .map_err(|e| format!("Failed to parse manifest.json: {}", e))?;

        let id = format!("{}_{}", sanitize_id(&manifest.name), manifest.version);

        let mut matchers = Vec::new();
        for cs in &manifest.content_scripts {
            for pattern in &cs.matches {
                if let Some(m) = PatternMatcher::new(pattern) {
                    matchers.push((m, cs.clone()));
                }
            }
        }

        let ext = Extension {
            manifest,
            root_path: path.to_path_buf(),
            enabled: true,
            matchers,
        };

        self.extensions.lock().unwrap().insert(id.clone(), ext);
        Ok(id)
    }

    /// Scan default extensions directory for installed extensions
    pub fn scan_extensions_dir(&self) {
        if let Ok(entries) = std::fs::read_dir(&self.extensions_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() && p.join("manifest.json").exists() {
                    let _ = self.load_unpacked(&p);
                }
            }
        }
    }

    /// Toggle extension on or off
    pub fn toggle_extension(&self, id: &str) -> bool {
        let mut map = self.extensions.lock().unwrap();
        if let Some(ext) = map.get_mut(id) {
            ext.enabled = !ext.enabled;
            ext.enabled
        } else {
            false
        }
    }

    /// Remove extension
    pub fn remove_extension(&self, id: &str) {
        self.extensions.lock().unwrap().remove(id);
    }

    /// Get list of installed extensions for UI
    pub fn list_extensions(&self) -> Vec<ExtensionInfo> {
        let map = self.extensions.lock().unwrap();
        map.iter()
            .map(|(id, ext)| {
                let popup = ext.manifest.get_popup_file();
                ExtensionInfo {
                    id: id.clone(),
                    name: ext.manifest.name.clone(),
                    version: ext.manifest.version.clone(),
                    description: ext.manifest.description.clone().unwrap_or_default(),
                    author: ext.manifest.author.clone(),
                    enabled: ext.enabled,
                    permissions: ext.manifest.permissions.clone(),
                    has_popup: popup.is_some(),
                    popup_file: popup,
                    root_path: ext.root_path.clone(),
                }
            })
            .collect()
    }

    /// Get matching content scripts and styles for a webpage URL
    /// Returns (vec of polyfill+js sources, vec of css styles)
    pub fn get_scripts_for_url(&self, target_url: &str) -> (Vec<String>, Vec<String>) {
        let mut scripts = Vec::new();
        let mut styles = Vec::new();

        let map = self.extensions.lock().unwrap();
        for (id, ext) in map.iter() {
            if !ext.enabled {
                continue;
            }

            let mut matched_ext = false;
            let mut ext_js_files = Vec::new();
            let mut ext_css_files = Vec::new();

            for (matcher, cs) in &ext.matchers {
                if matcher.matches(target_url) {
                    matched_ext = true;
                    for js_file in &cs.js {
                        if !ext_js_files.contains(js_file) {
                            ext_js_files.push(js_file.clone());
                        }
                    }
                    for css_file in &cs.css {
                        if !ext_css_files.contains(css_file) {
                            ext_css_files.push(css_file.clone());
                        }
                    }
                }
            }

            if matched_ext {
                // Prepend the WebExtensions polyfill
                scripts.push(polyfill::generate_webextensions_polyfill(id, &ext.manifest.name));

                // Read and append content script files
                for js_rel in ext_js_files {
                    let full_path = ext.root_path.join(&js_rel);
                    if let Ok(content) = std::fs::read_to_string(&full_path) {
                        scripts.push(format!("/* Extension: {} - {} */\n{}", ext.manifest.name, js_rel, content));
                    }
                }

                // Read and append stylesheet files
                for css_rel in ext_css_files {
                    let full_path = ext.root_path.join(&css_rel);
                    if let Ok(content) = std::fs::read_to_string(&full_path) {
                        styles.push(content);
                    }
                }
            }
        }

        (scripts, styles)
    }

    /// Creates a sample built-in Dark Mode / Page Highlighter extension
    fn create_sample_extension_if_empty(&self) {
        let sample_dir = self.extensions_dir.join("sample_highlighter");
        if sample_dir.exists() {
            return;
        }

        let _ = std::fs::create_dir_all(&sample_dir);

        let manifest_content = r#"{
  "manifest_version": 3,
  "name": "Web Highlighter & Dark Mode",
  "version": "1.0.0",
  "description": "Sample Firefox and Chrome compatible WebExtension for Browsy",
  "permissions": ["storage", "activeTab"],
  "content_scripts": [
    {
      "matches": ["<all_urls>"],
      "js": ["content.js"],
      "css": ["style.css"],
      "run_at": "document_idle"
    }
  ],
  "action": {
    "default_title": "Web Highlighter",
    "default_popup": "popup.html"
  }
}"#;

        let content_js = r#"console.log("[Browsy WebExtension] Web Highlighter injected successfully via chrome.* API!");
document.addEventListener("DOMContentLoaded", () => {
    console.log("[Browsy WebExtension] Page ready: " + document.title);
});
"#;

        let style_css = r#"/* Sample WebExtension styles */
.browsy-extension-highlight {
    background-color: yellow !important;
}
"#;

        let popup_html = r#"<!DOCTYPE html>
<html>
<head><style>body { font-family: sans-serif; padding: 10px; width: 200px; }</style></head>
<body>
  <h3>Browsy WebExtension</h3>
  <p>Active and running with Chrome & Firefox WebExtensions API!</p>
</body>
</html>"#;

        let _ = std::fs::write(sample_dir.join("manifest.json"), manifest_content);
        let _ = std::fs::write(sample_dir.join("content.js"), content_js);
        let _ = std::fs::write(sample_dir.join("style.css"), style_css);
        let _ = std::fs::write(sample_dir.join("popup.html"), popup_html);
    }
}

impl Default for ExtensionManager {
    fn default() -> Self {
        Self::new()
    }
}

fn sanitize_id(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect::<String>()
        .to_lowercase()
}

fn dirs_or_fallback_ext_dir() -> PathBuf {
    if let Some(user_dirs) = std::env::var_os("USERPROFILE") {
        return PathBuf::from(user_dirs).join(".browsy").join("extensions");
    }
    PathBuf::from("./extensions")
}
