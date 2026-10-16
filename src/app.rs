use std::sync::{Arc, Mutex};
use eframe::egui;

use crate::adblock::AdblockEngine;
use crate::engine::BrowserEngine;
use crate::extensions::ExtensionManager;
use crate::popup::PopupBlocker;
use crate::tabs::TabManager;
use crate::text_expander::TextExpander;
use crate::ui::BrowserUi;
use crate::video_downloader::VideoDownloader;

pub struct BrowsyApp {
    pub adblock: Arc<AdblockEngine>,
    pub popup_blocker: Arc<PopupBlocker>,
    pub video_downloader: Arc<VideoDownloader>,
    pub text_expander: Arc<TextExpander>,
    pub extensions: Arc<ExtensionManager>,
    pub tabs: Arc<Mutex<TabManager>>,
    pub engine: BrowserEngine,
    pub ui: BrowserUi,
}

impl BrowsyApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let adblock = Arc::new(AdblockEngine::new());
        let popup_blocker = Arc::new(PopupBlocker::new());
        let video_downloader = Arc::new(VideoDownloader::new());
        let text_expander = Arc::new(TextExpander::new());
        let extensions = Arc::new(ExtensionManager::new());
        let tabs = Arc::new(Mutex::new(TabManager::new()));

        let engine = BrowserEngine::new(
            adblock.clone(),
            popup_blocker.clone(),
            video_downloader.clone(),
            text_expander.clone(),
            extensions.clone(),
            tabs.clone(),
        );

        let ui = BrowserUi::new();

        Self {
            adblock,
            popup_blocker,
            video_downloader,
            text_expander,
            extensions,
            tabs,
            engine,
            ui,
        }
    }
}

impl eframe::App for BrowsyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Handle global keyboard shortcuts
        ui.input(|i| {
            // Ctrl+T: New Tab
            if i.modifiers.command && i.key_pressed(egui::Key::T) && !i.modifiers.shift {
                let mut tabs = self.tabs.lock().unwrap();
                tabs.new_tab("browsy://welcome");
                self.ui.omnibox_input = "browsy://welcome".to_string();
            }

            // Ctrl+W: Close active tab
            if i.modifiers.command && i.key_pressed(egui::Key::W) {
                let mut tabs = self.tabs.lock().unwrap();
                let cur_id = tabs.active_tab_id;
                tabs.close_tab(cur_id);
                if let Some(tab) = tabs.active_tab() {
                    self.ui.omnibox_input = tab.url.clone();
                }
            }

            // Ctrl+Shift+V: Toggle Vertical / Horizontal Tab strip
            if i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::V) {
                let mut tabs = self.tabs.lock().unwrap();
                tabs.toggle_layout();
            }

            // Ctrl+Shift+E: Toggle Text Expander modal
            if i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::E) {
                self.ui.modal_state.show_text_expander = !self.ui.modal_state.show_text_expander;
            }

            // Record user interaction for pop-up blocker gesture detection
            if i.pointer.any_click() || i.keys_down.len() > 0 {
                let tabs = self.tabs.lock().unwrap();
                self.popup_blocker.record_user_interaction(tabs.active_tab_id);
            }
        });

        // Spin Servo engine event loop
        self.engine.spin_event_loop();

        // Render Browser UI
        let mut tabs_guard = self.tabs.lock().unwrap();
        self.ui.render(
            ui,
            &mut *tabs_guard,
            &self.adblock,
            &self.popup_blocker,
            &self.video_downloader,
            &self.text_expander,
            &self.extensions,
            &self.engine,
        );
    }
}
