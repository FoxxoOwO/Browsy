use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use adblock::engine::Engine;
use adblock::lists::{FilterSet, ParseOptions};
use adblock::request::Request;
use serde::{Deserialize, Serialize};

/// Record of an intercepted/blocked network request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedRequestRecord {
    pub url: String,
    pub domain: String,
    pub source_url: String,
    pub resource_type: String,
    pub matched_rule: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Local>,
}

/// Decision made by the adblock engine
#[derive(Debug, Clone)]
pub struct AdblockDecision {
    pub should_block: bool,
    pub matched_rule: Option<String>,
}

/// Statistics for adblocking
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AdblockStats {
    pub total_blocked: usize,
    pub blocked_by_domain: HashMap<String, usize>,
    pub blocked_history: Vec<BlockedRequestRecord>,
}

/// Integrated uBlock Origin / Brave adblock engine
pub struct AdblockEngine {
    engine: Mutex<Engine>,
    enabled: Mutex<bool>,
    allowlist: Mutex<HashSet<String>>,
    custom_rules: Mutex<Vec<String>>,
    stats: Arc<Mutex<AdblockStats>>,
}

impl AdblockEngine {
    /// Creates a new AdblockEngine with built-in uBlock Origin & EasyList rules
    pub fn new() -> Self {
        let default_rules = Self::default_filter_rules();
        let mut filter_set = FilterSet::new(true);
        let joined = default_rules.join("\n");
        filter_set.add_filter_list(joined, ParseOptions::default());

        let engine = Engine::new_with_filter_set(filter_set);

        Self {
            engine: Mutex::new(engine),
            enabled: Mutex::new(true),
            allowlist: Mutex::new(HashSet::new()),
            custom_rules: Mutex::new(default_rules),
            stats: Arc::new(Mutex::new(AdblockStats::default())),
        }
    }

    /// Default baseline rules resembling core uBlock Origin / EasyList rules
    fn default_filter_rules() -> Vec<String> {
        vec![
            // Generic ad network rules
            r#"||googlesyndication.com^$third-party"#.to_string(),
            r#"||googleadservices.com^$third-party"#.to_string(),
            r#"||doubleclick.net^$third-party"#.to_string(),
            r#"||adservice.google.*^"#.to_string(),
            r#"||pagead2.googlesyndication.com^"#.to_string(),
            r#"||analytics.google.com^"#.to_string(),
            r#"||google-analytics.com^"#.to_string(),
            r#"||facebook.net/signals/*"#.to_string(),
            r#"||connect.facebook.net/*/fbevents.js"#.to_string(),
            r#"||adnxs.com^$third-party"#.to_string(),
            r#"||criteo.com^$third-party"#.to_string(),
            r#"||taboola.com^$third-party"#.to_string(),
            r#"||outbrain.com^$third-party"#.to_string(),
            r#"||scorecardresearch.com^$third-party"#.to_string(),
            r#"||rubiconproject.com^$third-party"#.to_string(),
            r#"||pubmatic.com^$third-party"#.to_string(),
            r#"||openx.net^$third-party"#.to_string(),
            r#"||amazon-adsystem.com^$third-party"#.to_string(),
            r#"||adroll.com^$third-party"#.to_string(),
            r#"||quantserve.com^$third-party"#.to_string(),
            r#"||hotjar.com^$third-party"#.to_string(),
            r#"||clarity.ms^$third-party"#.to_string(),
            r#"||smartadserver.com^$third-party"#.to_string(),
            r#"||popads.net^"#.to_string(),
            r#"||popcash.net^"#.to_string(),
            r#"||propellerads.com^"#.to_string(),
            r#"||adsterra.com^"#.to_string(),
            // Path-based generic ad blocks
            r#"/ads.js"#.to_string(),
            r#"/advertisement.js"#.to_string(),
            r#"/ad-banner.js"#.to_string(),
            r#"/ads/banner_"#.to_string(),
            r#"/prebid.js"#.to_string(),
            // Cosmetic element hiding rules (uBlock syntax)
            r#"##.adsbygoogle"#.to_string(),
            r#"##.ad-container"#.to_string(),
            r#"##.ad-banner"#.to_string(),
            r#"##.advertisement"#.to_string(),
            r#"##[id^="google_ads_"]"#.to_string(),
            r#"##[id^="div-gpt-ad"]"#.to_string(),
            r#"##.sponsored-post"#.to_string(),
            r#"##.outbrain_widget"#.to_string(),
            r#"##.taboola-placeholder"#.to_string(),
        ]
    }

    /// Checks whether an outgoing network request should be blocked
    pub fn check_request(&self, url: &str, source_url: &str, resource_type: &str) -> AdblockDecision {
        if !*self.enabled.lock().unwrap() {
            return AdblockDecision {
                should_block: false,
                matched_rule: None,
            };
        }

        // Check if host is allowlisted
        if let Ok(parsed) = url::Url::parse(source_url) {
            if let Some(host) = parsed.host_str() {
                if self.is_allowlisted(host) {
                    return AdblockDecision {
                        should_block: false,
                        matched_rule: None,
                    };
                }
            }
        }

        // Normalize resource type to standard ABP types
        let req_type = match resource_type.to_lowercase().as_str() {
            "script" => "script",
            "image" | "img" => "image",
            "stylesheet" | "css" => "stylesheet",
            "subdocument" | "iframe" => "subdocument",
            "xmlhttprequest" | "xhr" | "fetch" => "xmlhttprequest",
            "media" | "video" | "audio" => "media",
            _ => "other",
        };

        if let Ok(req) = Request::new(url, source_url, req_type, "GET") {
            let engine = self.engine.lock().unwrap();
            let result = engine.check_network_request(&req);
            let should_block = result.should_block();

            if should_block {
                let rule_info = result
                    .filter
                    .as_ref()
                    .and_then(|f| f.raw_line.clone())
                    .or_else(|| Some("adblock-rule".to_string()));

                // Extract domain for stats
                let domain = url::Url::parse(url)
                    .ok()
                    .and_then(|u| u.host_str().map(|s| s.to_string()))
                    .unwrap_or_else(|| "unknown".to_string());

                let mut stats = self.stats.lock().unwrap();
                stats.total_blocked += 1;
                *stats.blocked_by_domain.entry(domain.clone()).or_insert(0) += 1;

                if stats.blocked_history.len() >= 100 {
                    stats.blocked_history.remove(0);
                }
                stats.blocked_history.push(BlockedRequestRecord {
                    url: url.to_string(),
                    domain,
                    source_url: source_url.to_string(),
                    resource_type: req_type.to_string(),
                    matched_rule: rule_info.clone(),
                    timestamp: chrono::Local::now(),
                });

                return AdblockDecision {
                    should_block: true,
                    matched_rule: rule_info,
                };
            }
        }

        AdblockDecision {
            should_block: false,
            matched_rule: None,
        }
    }

    /// Generates cosmetic filtering CSS for a given page URL
    pub fn get_cosmetic_css(&self, url: &str) -> String {
        if !*self.enabled.lock().unwrap() {
            return String::new();
        }

        if let Ok(parsed) = url::Url::parse(url) {
            if let Some(host) = parsed.host_str() {
                if self.is_allowlisted(host) {
                    return String::new();
                }
            }
        }

        let engine = self.engine.lock().unwrap();
        let resources = engine.url_cosmetic_resources(url);

        if resources.hide_selectors.is_empty() {
            return String::new();
        }

        let mut css = String::with_capacity(resources.hide_selectors.len() * 40);
        for selector in resources.hide_selectors {
            css.push_str(&selector);
            css.push_str(" { display: none !important; }\n");
        }
        css
    }

    /// Check if a domain is allowlisted
    pub fn is_allowlisted(&self, domain: &str) -> bool {
        let allow = self.allowlist.lock().unwrap();
        allow.contains(domain)
            || domain
                .split('.')
                .enumerate()
                .any(|(i, _)| i > 0 && allow.contains(&domain.split('.').skip(i).collect::<Vec<_>>().join(".")))
    }

    /// Toggle allowlist for a domain
    pub fn toggle_allowlist(&self, domain: &str) -> bool {
        let mut allow = self.allowlist.lock().unwrap();
        if allow.contains(domain) {
            allow.remove(domain);
            false
        } else {
            allow.insert(domain.to_string());
            true
        }
    }

    /// Enable or disable the entire adblock engine
    pub fn set_enabled(&self, enabled: bool) {
        *self.enabled.lock().unwrap() = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        *self.enabled.lock().unwrap()
    }

    /// Get current statistics
    pub fn stats(&self) -> AdblockStats {
        self.stats.lock().unwrap().clone()
    }

    /// Get blocked count for a specific page URL
    pub fn blocked_for_url(&self, url: &str) -> usize {
        let stats = self.stats.lock().unwrap();
        if let Ok(parsed) = url::Url::parse(url) {
            if let Some(host) = parsed.host_str() {
                return stats.blocked_by_domain.get(host).copied().unwrap_or(0);
            }
        }
        0
    }

    /// Add more custom filter rules
    pub fn add_rules(&self, rules: &[String]) {
        let mut all_rules = self.custom_rules.lock().unwrap();
        for r in rules {
            if !r.trim().is_empty() {
                all_rules.push(r.trim().to_string());
            }
        }

        let joined = all_rules.join("\n");
        let mut filter_set = FilterSet::new(true);
        filter_set.add_filter_list(joined, ParseOptions::default());
        *self.engine.lock().unwrap() = Engine::new_with_filter_set(filter_set);
    }
}

impl Default for AdblockEngine {
    fn default() -> Self {
        Self::new()
    }
}
