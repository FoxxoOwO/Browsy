use std::sync::{Arc, Mutex};
use servo::{
    CreateNewWebViewRequest, LoadStatus, WebResourceLoad, WebResourceResponse, WebView,
    WebViewDelegate,
};
use url::Url;

use crate::adblock::AdblockEngine;
use crate::popup::PopupBlocker;
use crate::tabs::TabManager;
use crate::video_downloader::VideoDownloader;

pub struct BrowsyDelegate {
    pub tab_id: usize,
    pub adblock: Arc<AdblockEngine>,
    pub popup_blocker: Arc<PopupBlocker>,
    pub video_downloader: Arc<VideoDownloader>,
    pub tabs: Arc<Mutex<TabManager>>,
}

impl BrowsyDelegate {
    pub fn new(
        tab_id: usize,
        adblock: Arc<AdblockEngine>,
        popup_blocker: Arc<PopupBlocker>,
        video_downloader: Arc<VideoDownloader>,
        tabs: Arc<Mutex<TabManager>>,
    ) -> Self {
        Self {
            tab_id,
            adblock,
            popup_blocker,
            video_downloader,
            tabs,
        }
    }
}

impl WebViewDelegate for BrowsyDelegate {
    fn notify_url_changed(&self, webview: WebView, url: Url) {
        let mut tabs = self.tabs.lock().unwrap();
        if let Some(tab) = tabs.tabs.iter_mut().find(|t| t.id == self.tab_id) {
            tab.url = url.to_string();
            tab.can_go_back = webview.can_go_back();
            tab.can_go_forward = webview.can_go_forward();
        }
    }

    fn notify_page_title_changed(&self, _webview: WebView, title: Option<String>) {
        let mut tabs = self.tabs.lock().unwrap();
        if let Some(tab) = tabs.tabs.iter_mut().find(|t| t.id == self.tab_id) {
            if let Some(t) = title {
                if !t.is_empty() {
                    tab.title = t;
                }
            }
        }
    }

    fn notify_load_status_changed(&self, _webview: WebView, status: LoadStatus) {
        let mut tabs = self.tabs.lock().unwrap();
        if let Some(tab) = tabs.tabs.iter_mut().find(|t| t.id == self.tab_id) {
            tab.is_loading = status != LoadStatus::Complete;
        }
    }

    /// Intercept HTTP / HTTPS requests to enforce uBlock Origin filtering & video stream sniffing!
    fn load_web_resource(&self, webview: WebView, load: WebResourceLoad) {
        let req_url = load.request.url.to_string();
        let page_url = webview.url().map(|u| u.to_string()).unwrap_or_default();
        let page_title = webview.page_title().unwrap_or_default();

        // 1. Video Downloader sniffer check
        self.video_downloader.sniff_url(&req_url, &page_url, &page_title);

        // Update detected video count on tab
        let detected = self.video_downloader.streams_for_page(&page_url).len();
        {
            let mut tabs = self.tabs.lock().unwrap();
            if let Some(tab) = tabs.tabs.iter_mut().find(|t| t.id == self.tab_id) {
                tab.detected_videos = detected;
            }
        }

        // 2. uBlock Origin adblock filter check
        let decision = self.adblock.check_request(&req_url, &page_url, "other");
        if decision.should_block {
            // Update blocked ads count on tab
            let mut tabs = self.tabs.lock().unwrap();
            if let Some(tab) = tabs.tabs.iter_mut().find(|t| t.id == self.tab_id) {
                tab.blocked_ads += 1;
            }

            // Cancel the intercepted resource load!
            if let Ok(parsed_url) = Url::parse(&req_url) {
                let intercept = load.intercept(WebResourceResponse::new(parsed_url));
                intercept.cancel();
            }
        }
    }

    /// Intercept `window.open` requests to enforce the Pop-up Blocker!
    fn request_create_new(&self, parent_webview: WebView, _request: CreateNewWebViewRequest) {
        let source_url = parent_webview.url().map(|u| u.to_string()).unwrap_or_default();
        // Since CreateNewWebViewRequest does not yet expose target_url until navigation,
        // we check user gesture heuristic on the parent tab
        let allowed = self.popup_blocker.evaluate_popup_request(self.tab_id, &source_url, "about:blank");

        if !allowed {
            // Pop-up blocked!
            let mut tabs = self.tabs.lock().unwrap();
            if let Some(tab) = tabs.tabs.iter_mut().find(|t| t.id == self.tab_id) {
                tab.blocked_popups += 1;
            }
            // By dropping `request` without calling `.builder(...)`, Servo cancels the popup window!
            return;
        }

        // Allowed: create a new tab in Browsy
        let mut tabs = self.tabs.lock().unwrap();
        let new_tab_id = tabs.new_tab("about:blank");
        log::info!("Created auxiliary tab {} for allowed pop-up", new_tab_id);
    }
}
