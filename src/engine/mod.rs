pub mod delegate;

use std::rc::Rc;
use std::sync::{Arc, Mutex};
use servo::{
    EventLoopWaker, Servo, ServoBuilder, UserContentManager, UserScript,
};
use embedder_traits::user_contents::UserStyleSheet;
use url::Url;

use crate::adblock::AdblockEngine;
use crate::extensions::ExtensionManager;
use crate::popup::PopupBlocker;
use crate::tabs::TabManager;
use crate::text_expander::TextExpander;
use crate::video_downloader::VideoDownloader;

#[derive(Clone)]
pub struct SimpleWaker;

impl EventLoopWaker for SimpleWaker {
    fn clone_box(&self) -> Box<dyn EventLoopWaker> {
        Box::new(SimpleWaker)
    }
    fn wake(&self) {}
}

/// Browser Engine managing Servo and content injection
pub struct BrowserEngine {
    pub servo: Option<Servo>,
    pub user_content_manager: Option<UserContentManager>,
    pub adblock: Arc<AdblockEngine>,
    pub popup_blocker: Arc<PopupBlocker>,
    pub video_downloader: Arc<VideoDownloader>,
    pub text_expander: Arc<TextExpander>,
    pub extensions: Arc<ExtensionManager>,
    pub tabs: Arc<Mutex<TabManager>>,
    pub is_servo_active: bool,
}

impl BrowserEngine {
    pub fn new(
        adblock: Arc<AdblockEngine>,
        popup_blocker: Arc<PopupBlocker>,
        video_downloader: Arc<VideoDownloader>,
        text_expander: Arc<TextExpander>,
        extensions: Arc<ExtensionManager>,
        tabs: Arc<Mutex<TabManager>>,
    ) -> Self {
        // Initialize Servo instance with waker
        let servo_res = std::panic::catch_unwind(|| {
            let servo = ServoBuilder::default()
                .event_loop_waker(Box::new(SimpleWaker))
                .build();
            let ucm = UserContentManager::new(&servo);
            (servo, ucm)
        });

        let (servo, ucm, is_active) = match servo_res {
            Ok((s, ucm)) => {
                log::info!("Servo Web Engine initialized successfully.");
                (Some(s), Some(ucm), true)
            }
            Err(e) => {
                log::warn!("Running with embedded Servo web engine abstractions: {:?}", e);
                (None, None, false)
            }
        };

        Self {
            servo,
            user_content_manager: ucm,
            adblock,
            popup_blocker,
            video_downloader,
            text_expander,
            extensions,
            tabs,
            is_servo_active: is_active,
        }
    }

    /// Prepare user content (uBlock Origin cosmetic CSS, WebExtensions content scripts, Text Expander script)
    /// for a newly navigated page URL
    pub fn prepare_page_content(&self, url: &str) {
        if let Some(ucm) = &self.user_content_manager {
            // 1. Inject uBlock Origin cosmetic element hiding CSS
            let cosmetic_css = self.adblock.get_cosmetic_css(url);
            if !cosmetic_css.is_empty() {
                if let Ok(parsed_url) = Url::parse(url) {
                    let user_style = UserStyleSheet::new(cosmetic_css, parsed_url);
                    ucm.add_stylesheet(Rc::new(user_style));
                }
            }

            // 2. Inject Text Expander content script into web pages
            let expander_js = self.text_expander.generate_content_script();
            let expander_script = UserScript::new(expander_js, None);
            ucm.add_script(Rc::new(expander_script));

            // 3. Inject matching Firefox & Chrome WebExtensions scripts
            let (ext_scripts, ext_styles) = self.extensions.get_scripts_for_url(url);
            for s in ext_scripts {
                let script_obj = UserScript::new(s, None);
                ucm.add_script(Rc::new(script_obj));
            }
            for css in ext_styles {
                if let Ok(parsed_url) = Url::parse(url) {
                    let style_obj = UserStyleSheet::new(css, parsed_url);
                    ucm.add_stylesheet(Rc::new(style_obj));
                }
            }
        }
    }

    /// Creates a Servo WebViewDelegate connected to the browser's adblock, pop-up blocker, and video sniffer
    pub fn create_delegate_for_tab(&self, tab_id: usize) -> delegate::BrowsyDelegate {
        delegate::BrowsyDelegate::new(
            tab_id,
            self.adblock.clone(),
            self.popup_blocker.clone(),
            self.video_downloader.clone(),
            self.tabs.clone(),
        )
    }

    /// Spin the Servo event loop to process user events, network loads, scripts
    pub fn spin_event_loop(&self) {
        if let Some(servo) = &self.servo {
            servo.spin_event_loop();
        }
    }

    /// Generates content for internal `browsy://` pages
    pub fn get_internal_page_html(&self, page: &str) -> String {
        match page {
            "welcome" | "" => self.html_welcome_page(),
            "adblock" => self.html_adblock_page(),
            "extensions" => self.html_extensions_page(),
            "expander" => self.html_expander_page(),
            "downloads" => self.html_downloads_page(),
            _ => self.html_not_found(page),
        }
    }

    fn html_welcome_page(&self) -> String {
        let stats = self.adblock.stats();
        let total_blocked = stats.total_blocked;
        let ext_count = self.extensions.list_extensions().len();
        let snippet_count = self.text_expander.all_snippets().len();
        let video_count = self.video_downloader.total_detected();

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Welcome to Browsy</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; background: #0f172a; color: #f8fafc; margin: 0; padding: 40px; text-align: center; }}
        .hero {{ max-width: 800px; margin: 0 auto; }}
        h1 {{ font-size: 48px; margin-bottom: 8px; color: #38bdf8; }}
        p.subtitle {{ font-size: 18px; color: #94a3b8; margin-bottom: 40px; }}
        .grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 20px; margin-top: 30px; text-align: left; }}
        .card {{ background: #1e293b; padding: 20px; border-radius: 12px; border: 1px solid #334155; }}
        .card h3 {{ margin-top: 0; color: #38bdf8; font-size: 18px; }}
        .card p {{ color: #cbd5e1; font-size: 14px; margin-bottom: 0; }}
        .badge {{ background: #0284c7; color: white; padding: 3px 8px; border-radius: 6px; font-size: 12px; font-weight: bold; }}
    </style>
</head>
<body>
    <div class="hero">
        <h1>🌐 Browsy</h1>
        <p class="subtitle">High-performance Rust web browser powered by the <strong>Servo</strong> web engine</p>
        
        <div class="grid">
            <div class="card">
                <h3>🛡️ uBlock Origin Shield <span class="badge">Active</span></h3>
                <p>Native Rust Adblock Plus / uBlock engine with EasyList filters & cosmetic hiding. Blocked <strong>{total_blocked}</strong> ads.</p>
            </div>
            <div class="card">
                <h3>🚫 Pop-up Blocker <span class="badge">Active</span></h3>
                <p>Smart user-activation heuristics intercept unsolicited scripts, pop-unders, and redirect loops.</p>
            </div>
            <div class="card">
                <h3>🎬 Video Downloader <span class="badge">{video_count} detected</span></h3>
                <p>Sniffs HLS (m3u8), DASH (mpd), MP4, and WebM streams with direct chunked downloader & yt-dlp integration.</p>
            </div>
            <div class="card">
                <h3>⚡ Text Expander <span class="badge">{snippet_count} snippets</span></h3>
                <p>Trigger shortcuts like <code>:email</code>, <code>:date</code>, <code>:shrug</code> everywhere in form inputs & the address bar.</p>
            </div>
            <div class="card">
                <h3>🧩 WebExtensions <span class="badge">{ext_count} installed</span></h3>
                <p>Compatible with Google Chrome & Mozilla Firefox WebExtensions (Manifest V2 & V3 API).</p>
            </div>
            <div class="card">
                <h3>📑 Dual Tab Modes</h3>
                <p>Seamlessly toggle between classic <strong>Horizontal</strong> and modern <strong>Vertical</strong> sidebar tabs (Ctrl+Shift+V).</p>
            </div>
        </div>
    </div>
</body>
</html>"#
        )
    }

    fn html_adblock_page(&self) -> String {
        let stats = self.adblock.stats();
        let mut domain_rows = String::new();
        for (dom, count) in &stats.blocked_by_domain {
            domain_rows.push_str(&format!(
                "<tr><td>{}</td><td><strong>{}</strong></td></tr>",
                dom, count
            ));
        }

        format!(
            r#"<!DOCTYPE html>
<html>
<head><title>uBlock Origin Shield - Browsy</title>
<style>body {{ font-family: sans-serif; background: #0f172a; color: #fff; padding: 30px; }}
table {{ width: 100%; max-width: 600px; border-collapse: collapse; margin-top: 20px; }}
th, td {{ padding: 10px; border-bottom: 1px solid #334155; text-align: left; }}
th {{ background: #1e293b; color: #38bdf8; }}
</style></head>
<body>
<h1>🛡️ Integrated uBlock Origin Shield</h1>
<p>Total ads and tracking requests blocked: <strong>{}</strong></p>
<table>
<thead><tr><th>Blocked Domain</th><th>Requests Blocked</th></tr></thead>
<tbody>{}</tbody>
</table>
</body></html>"#,
            stats.total_blocked, domain_rows
        )
    }

    fn html_extensions_page(&self) -> String {
        let exts = self.extensions.list_extensions();
        let mut list = String::new();
        for ext in exts {
            list.push_str(&format!(
                "<div style='background:#1e293b; padding:15px; border-radius:8px; margin-bottom:10px;'>
                    <h3>{} v{}</h3>
                    <p>{}</p>
                    <small>Permissions: {:?}</small>
                </div>",
                ext.name, ext.version, ext.description, ext.permissions
            ));
        }

        format!(
            r#"<!DOCTYPE html><html><head><title>WebExtensions</title>
<style>body {{ font-family: sans-serif; background: #0f172a; color: #fff; padding: 30px; }}</style></head>
<body><h1>🧩 Installed Firefox & Chrome Extensions</h1>{}</body></html>"#,
            list
        )
    }

    fn html_expander_page(&self) -> String {
        let snippets = self.text_expander.all_snippets();
        let mut rows = String::new();
        for s in snippets {
            rows.push_str(&format!(
                "<tr><td><code>{}</code></td><td>{}</td><td>{}</td></tr>",
                s.trigger, s.replacement, s.description
            ));
        }

        format!(
            r#"<!DOCTYPE html><html><head><title>Text Expander</title>
<style>body {{ font-family: sans-serif; background: #0f172a; color: #fff; padding: 30px; }}
table {{ width: 100%; border-collapse: collapse; }}
th, td {{ padding: 8px; border-bottom: 1px solid #334155; text-align: left; }}</style></head>
<body><h1>⚡ Text Expander Snippets</h1><table><thead><tr><th>Trigger</th><th>Replacement</th><th>Description</th></tr></thead><tbody>{}</tbody></table></body></html>"#,
            rows
        )
    }

    fn html_downloads_page(&self) -> String {
        let downloads = self.video_downloader.all_downloads();
        let mut rows = String::new();
        for d in downloads {
            rows.push_str(&format!(
                "<tr><td>{}</td><td>{:?}</td></tr>",
                d.filename, d.state
            ));
        }

        format!(
            r#"<!DOCTYPE html><html><head><title>Downloads</title>
<style>body {{ font-family: sans-serif; background: #0f172a; color: #fff; padding: 30px; }}</style></head>
<body><h1>🎬 Video & File Downloads</h1><table><tbody>{}</tbody></table></body></html>"#,
            rows
        )
    }

    fn html_not_found(&self, page: &str) -> String {
        format!(
            r#"<!DOCTYPE html><html><body style="background:#0f172a; color:#fff; font-family:sans-serif; padding:40px;">
<h1>404 - Page Not Found</h1><p>Internal page <code>browsy://{}</code> does not exist.</p></body></html>"#,
            page
        )
    }
}
