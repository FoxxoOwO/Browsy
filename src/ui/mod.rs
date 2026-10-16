pub mod modals;

use eframe::egui::{self, Color32, CornerRadius, Frame, Margin, Panel, RichText};
use modals::ModalState;

use crate::adblock::AdblockEngine;
use crate::engine::BrowserEngine;
use crate::extensions::ExtensionManager;
use crate::popup::PopupBlocker;
use crate::tabs::{TabLayout, TabManager};
use crate::text_expander::TextExpander;
use crate::video_downloader::VideoDownloader;

pub struct BrowserUi {
    pub omnibox_input: String,
    pub modal_state: ModalState,
}

impl BrowserUi {
    pub fn new() -> Self {
        Self {
            omnibox_input: "browsy://welcome".to_string(),
            modal_state: ModalState::default(),
        }
    }

    /// Render the entire browser interface inside the root `Ui`
    pub fn render(
        &mut self,
        ui: &mut egui::Ui,
        tab_manager: &mut TabManager,
        adblock: &AdblockEngine,
        popup_blocker: &PopupBlocker,
        video_downloader: &VideoDownloader,
        text_expander: &TextExpander,
        extensions: &ExtensionManager,
        engine: &BrowserEngine,
    ) {
        let ctx = ui.ctx().clone();

        // Sync omnibox input with active tab URL if not currently typing
        if let Some(tab) = tab_manager.active_tab() {
            if !ctx.egui_wants_keyboard_input() && self.omnibox_input != tab.url {
                self.omnibox_input = tab.url.clone();
            }
        }

        // 1. Layout Mode: Horizontal vs Vertical Tabs
        match tab_manager.layout {
            TabLayout::Horizontal => {
                // Top Tab Strip
                Panel::top("horizontal_tab_strip")
                    .frame(Frame::new().fill(Color32::from_rgb(15, 23, 42)))
                    .show(ui, |ui| {
                        self.render_horizontal_tab_strip(ui, tab_manager);
                    });

                // Navigation Toolbar below tab strip
                Panel::top("navigation_toolbar")
                    .frame(Frame::new().fill(Color32::from_rgb(30, 41, 59)))
                    .show(ui, |ui| {
                        self.render_toolbar(
                            ui,
                            tab_manager,
                            adblock,
                            video_downloader,
                            text_expander,
                            extensions,
                            engine,
                        );
                    });
            }
            TabLayout::Vertical => {
                // Top Navigation Toolbar in Vertical mode
                Panel::top("navigation_toolbar")
                    .frame(Frame::new().fill(Color32::from_rgb(30, 41, 59)))
                    .show(ui, |ui| {
                        self.render_toolbar(
                            ui,
                            tab_manager,
                            adblock,
                            video_downloader,
                            text_expander,
                            extensions,
                            engine,
                        );
                    });

                // Left Sidebar Tab Strip
                Panel::left("vertical_tab_strip")
                    .resizable(true)
                    .default_size(tab_manager.sidebar_width)
                    .frame(Frame::new().fill(Color32::from_rgb(15, 23, 42)))
                    .show(ui, |ui| {
                        self.render_vertical_tab_strip(ui, tab_manager);
                    });
            }
        }

        // 2. Pop-up blocker alert notification banner (if active tab has blocked pop-ups)
        let active_tab_id = tab_manager.active_tab_id;
        if let Some(notif) = popup_blocker.get_pending_notification(active_tab_id) {
            Panel::top("popup_blocked_banner")
                .frame(Frame::new().fill(Color32::from_rgb(180, 83, 9)))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("🚫 Pop-up Blocked:").strong().color(Color32::WHITE));
                        ui.label(format!("Blocked window request to {}", notif.target_url));

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("❌ Dismiss").clicked() {
                                popup_blocker.dismiss_notification(active_tab_id);
                            }
                            if ui.button("Always Allow for Site").clicked() {
                                popup_blocker.allow_always(&notif.source_domain);
                                popup_blocker.dismiss_notification(active_tab_id);
                            }
                            if ui.button("Allow Once").clicked() {
                                tab_manager.new_tab(&notif.target_url);
                                popup_blocker.dismiss_notification(active_tab_id);
                            }
                        });
                    });
                });
        }

        // 3. Central Web Content Viewport
        egui::CentralPanel::default()
            .frame(Frame::new().fill(Color32::from_rgb(10, 15, 28)))
            .show(ui, |ui| {
                self.render_content_view(ui, tab_manager, engine);
            });

        // 4. Render Modals & Drawers
        let current_url = tab_manager.active_tab().map(|t| t.url.clone()).unwrap_or_default();
        let current_title = tab_manager.active_tab().map(|t| t.title.clone()).unwrap_or_default();

        modals::show_adblock_modal(&ctx, &mut self.modal_state, adblock, &current_url);
        modals::show_video_downloader_modal(
            &ctx,
            &mut self.modal_state,
            video_downloader,
            &current_url,
            &current_title,
        );
        modals::show_text_expander_modal(&ctx, &mut self.modal_state, text_expander);
        modals::show_extensions_modal(&ctx, &mut self.modal_state, extensions);
    }

    /// Render classic Horizontal Tab Strip
    fn render_horizontal_tab_strip(&mut self, ui: &mut egui::Ui, tab_manager: &mut TabManager) {
        ui.horizontal(|ui| {
            ui.add_space(6.0);

            let mut tab_to_close: Option<usize> = None;
            let mut tab_to_select: Option<usize> = None;

            egui::ScrollArea::horizontal().show(ui, |ui| {
                ui.horizontal(|ui| {
                    for tab in &tab_manager.tabs {
                        let is_active = tab.id == tab_manager.active_tab_id;
                        let bg = if is_active {
                            Color32::from_rgb(30, 41, 59)
                        } else {
                            Color32::from_rgb(15, 23, 42)
                        };

                        let frame = Frame::new()
                            .fill(bg)
                            .corner_radius(CornerRadius { nw: 8, ne: 8, sw: 0, se: 0 })
                            .inner_margin(Margin::symmetric(10, 6));

                        frame.show(ui, |ui| {
                            ui.horizontal(|ui| {
                                if tab.is_pinned {
                                    ui.label("📌");
                                } else {
                                    ui.label("🌐");
                                }

                                // Truncate title
                                let title: String = tab.title.chars().take(18).collect();
                                let title_text = if tab.is_loading {
                                    format!("⏳ {}", title)
                                } else {
                                    title
                                };

                                let tab_btn = ui.selectable_label(is_active, title_text);
                                if tab_btn.clicked() {
                                    tab_to_select = Some(tab.id);
                                }

                                // Badges
                                if tab.blocked_ads > 0 {
                                    ui.label(RichText::new(format!("{}", tab.blocked_ads)).size(10.0).color(Color32::from_rgb(56, 189, 248)));
                                }
                                if tab.detected_videos > 0 {
                                    ui.label(RichText::new("🎬").size(10.0));
                                }

                                // Close button
                                if ui.small_button("×").clicked() {
                                    tab_to_close = Some(tab.id);
                                }
                            });
                        });
                    }

                    // '+' New Tab button
                    if ui.button(" ➕ ").clicked() {
                        tab_manager.new_tab("browsy://welcome");
                    }
                });
            });

            // Layout switcher button on the right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("📑 Vertical Tabs").on_hover_text("Switch to Vertical Tabs (Ctrl+Shift+V)").clicked() {
                    tab_manager.toggle_layout();
                }
            });

            if let Some(id) = tab_to_select {
                tab_manager.select_tab(id);
            }
            if let Some(id) = tab_to_close {
                tab_manager.close_tab(id);
            }
        });
    }

    /// Render modern Vertical Sidebar Tab Strip (Arc / Zen / Edge style)
    fn render_vertical_tab_strip(&mut self, ui: &mut egui::Ui, tab_manager: &mut TabManager) {
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.heading(RichText::new("Browsy Tabs").size(15.0).strong().color(Color32::from_rgb(56, 189, 248)));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("📑 Horizontal").on_hover_text("Switch to Horizontal Tabs").clicked() {
                    tab_manager.toggle_layout();
                }
            });
        });

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            if ui.button("➕ New Tab").clicked() {
                tab_manager.new_tab("browsy://welcome");
            }
        });

        ui.separator();

        let mut tab_to_close: Option<usize> = None;
        let mut tab_to_select: Option<usize> = None;

        egui::ScrollArea::vertical().show(ui, |ui| {
            for tab in &tab_manager.tabs {
                let is_active = tab.id == tab_manager.active_tab_id;
                let bg = if is_active {
                    Color32::from_rgb(30, 41, 59)
                } else {
                    Color32::TRANSPARENT
                };

                let frame = Frame::new()
                    .fill(bg)
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(Margin::symmetric(8, 6));

                frame.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if tab.is_pinned {
                            ui.label("📌");
                        } else {
                            ui.label("🌐");
                        }

                        let title: String = tab.title.chars().take(22).collect();
                        let btn = ui.selectable_label(is_active, RichText::new(title).strong());
                        if btn.clicked() {
                            tab_to_select = Some(tab.id);
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("×").clicked() {
                                tab_to_close = Some(tab.id);
                            }
                            if tab.detected_videos > 0 {
                                ui.label(RichText::new("🎬").size(11.0));
                            }
                            if tab.blocked_ads > 0 {
                                ui.label(RichText::new(format!("{}", tab.blocked_ads)).size(10.0).color(Color32::from_rgb(56, 189, 248)));
                            }
                        });
                    });
                });
            }
        });

        if let Some(id) = tab_to_select {
            tab_manager.select_tab(id);
        }
        if let Some(id) = tab_to_close {
            tab_manager.close_tab(id);
        }
    }

    /// Render Navigation Toolbar and Omnibox
    fn render_toolbar(
        &mut self,
        ui: &mut egui::Ui,
        tab_manager: &mut TabManager,
        adblock: &AdblockEngine,
        video_downloader: &VideoDownloader,
        text_expander: &TextExpander,
        extensions: &ExtensionManager,
        engine: &BrowserEngine,
    ) {
        ui.horizontal(|ui| {
            ui.add_space(8.0);

            // Back button
            let can_back = tab_manager.active_tab().map(|t| t.can_go_back).unwrap_or(false);
            if ui.add_enabled(can_back, egui::Button::new("⮜")).clicked() {
                if let Some(tab) = tab_manager.active_tab_mut() {
                    if let Some(url) = tab.go_back() {
                        self.omnibox_input = url.to_string();
                        engine.prepare_page_content(url);
                    }
                }
            }

            // Forward button
            let can_fwd = tab_manager.active_tab().map(|t| t.can_go_forward).unwrap_or(false);
            if ui.add_enabled(can_fwd, egui::Button::new("⮞")).clicked() {
                if let Some(tab) = tab_manager.active_tab_mut() {
                    if let Some(url) = tab.go_forward() {
                        self.omnibox_input = url.to_string();
                        engine.prepare_page_content(url);
                    }
                }
            }

            // Reload button
            if ui.button("⟳").on_hover_text("Reload page").clicked() {
                if let Some(tab) = tab_manager.active_tab() {
                    engine.prepare_page_content(&tab.url);
                }
            }

            // Home button
            if ui.button("🏠").on_hover_text("Open Welcome page").clicked() {
                if let Some(tab) = tab_manager.active_tab_mut() {
                    tab.navigate("browsy://welcome");
                    self.omnibox_input = "browsy://welcome".to_string();
                }
            }

            // Omnibox
            ui.label("🔒");
            let omnibox_response = ui.add_sized(
                [ui.available_width() - 250.0, 24.0],
                egui::TextEdit::singleline(&mut self.omnibox_input).hint_text("Search or enter web address..."),
            );

            // Text Expander hook in Omnibox
            if omnibox_response.changed() {
                if let Some(expanded) = text_expander.try_expand_input(&self.omnibox_input) {
                    self.omnibox_input = expanded;
                }
            }

            // Navigate when user presses Enter
            if omnibox_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if let Some(tab) = tab_manager.active_tab_mut() {
                    tab.navigate(&self.omnibox_input);
                    engine.prepare_page_content(&tab.url);
                }
            }

            // Action toolbar buttons on right
            let active_url = tab_manager.active_tab().map(|t| t.url.clone()).unwrap_or_default();
            let blocked_count = adblock.blocked_for_url(&active_url);
            let streams_count = video_downloader.streams_for_page(&active_url).len();
            let ext_count = extensions.list_extensions().len();

            // 1. Adblock Shield button
            let shield_label = if blocked_count > 0 {
                format!("🛡️ {}", blocked_count)
            } else {
                "🛡️".to_string()
            };
            if ui.button(shield_label).on_hover_text("uBlock Origin Shield Protection").clicked() {
                self.modal_state.show_adblock = !self.modal_state.show_adblock;
            }

            // 2. Video Downloader button
            let video_label = if streams_count > 0 {
                format!("🎬 {}", streams_count)
            } else {
                "🎬".to_string()
            };
            if ui.button(video_label).on_hover_text("Video & Stream Downloader").clicked() {
                self.modal_state.show_video_downloader = !self.modal_state.show_video_downloader;
            }

            // 3. Text Expander button
            if ui.button("⚡").on_hover_text("Text Expander Snippets").clicked() {
                self.modal_state.show_text_expander = !self.modal_state.show_text_expander;
            }

            // 4. WebExtensions button
            if ui.button(format!("🧩 {}", ext_count)).on_hover_text("Firefox & Chrome Extensions").clicked() {
                self.modal_state.show_extensions = !self.modal_state.show_extensions;
            }
        });
    }

    /// Render Central Viewport (displays internal pages or Servo webview)
    fn render_content_view(
        &mut self,
        ui: &mut egui::Ui,
        tab_manager: &TabManager,
        engine: &BrowserEngine,
    ) {
        let active_tab = match tab_manager.active_tab() {
            Some(t) => t,
            None => return,
        };

        if active_tab.url.starts_with("browsy://") {
            let page_name = active_tab.url.trim_start_matches("browsy://");

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(20.0);
                ui.horizontal(|ui| {
                    ui.add_space(30.0);
                    ui.vertical(|ui| {
                        match page_name {
                            "welcome" => self.render_welcome_cards(ui),
                            "adblock" => {
                                ui.heading(RichText::new("🛡️ uBlock Origin Shield").size(24.0).color(Color32::from_rgb(56, 189, 248)));
                                ui.label("Inspect active filter lists, cosmetic rules, and domain protections.");
                                if ui.button("Open Adblock Control Panel").clicked() {
                                    self.modal_state.show_adblock = true;
                                }
                            }
                            "extensions" => {
                                ui.heading(RichText::new("🧩 Firefox & Chrome Extensions").size(24.0).color(Color32::from_rgb(56, 189, 248)));
                                ui.label("Manage installed WebExtensions.");
                                if ui.button("Open Extensions Manager").clicked() {
                                    self.modal_state.show_extensions = true;
                                }
                            }
                            "expander" => {
                                ui.heading(RichText::new("⚡ Text Expander").size(24.0).color(Color32::from_rgb(56, 189, 248)));
                                ui.label("Manage snippet shortcuts and expansion rules.");
                                if ui.button("Open Text Expander Manager").clicked() {
                                    self.modal_state.show_text_expander = true;
                                }
                            }
                            "downloads" => {
                                ui.heading(RichText::new("🎬 Video & Stream Downloads").size(24.0).color(Color32::from_rgb(56, 189, 248)));
                                ui.label("View captured video streams and active chunked downloads.");
                                if ui.button("Open Video Downloader Drawer").clicked() {
                                    self.modal_state.show_video_downloader = true;
                                }
                            }
                            _ => {
                                ui.label(format!("Browsy page: {}", page_name));
                            }
                        }
                    });
                });
            });
        } else {
            // Live Web Rendering Context
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                ui.heading(RichText::new(format!("🌐 Loading: {}", active_tab.url)).size(20.0).color(Color32::from_rgb(56, 189, 248)));
                ui.add_space(10.0);
                ui.label(RichText::new("Servo Web Engine render surface").color(Color32::GRAY));

                ui.add_space(20.0);
                ui.group(|ui| {
                    ui.label(format!("URL: {}", active_tab.url));
                    ui.label(format!("Engine: Servo (servo v0.7.0) | Status: {}", if engine.is_servo_active { "Active" } else { "Standby" }));
                    ui.label(format!("uBlock Origin: Active | Blocked on site: {}", active_tab.blocked_ads));
                    ui.label(format!("Pop-up Blocker: Active | Blocked pop-ups: {}", active_tab.blocked_popups));
                    ui.label(format!("Video Streams Sniffed: {}", active_tab.detected_videos));
                });
            });
        }
    }

    fn render_welcome_cards(&mut self, ui: &mut egui::Ui) {
        ui.heading(RichText::new("🌐 Welcome to Browsy").size(32.0).strong().color(Color32::from_rgb(56, 189, 248)));
        ui.label(RichText::new("Fast, private, customizable browser written in Rust on the Servo web engine.").size(16.0).color(Color32::LIGHT_GRAY));
        ui.add_space(16.0);

        ui.columns(3, |columns| {
            // Card 1
            columns[0].group(|ui| {
                ui.heading(RichText::new("🛡️ uBlock Origin").size(16.0).color(Color32::from_rgb(56, 189, 248)));
                ui.label("Integrated ad & tracker blocker with ABP syntax, EasyList, and cosmetic CSS hiding.");
                if ui.button("Shield Panel").clicked() {
                    self.modal_state.show_adblock = true;
                }
            });

            // Card 2
            columns[1].group(|ui| {
                ui.heading(RichText::new("🎬 Video Downloader").size(16.0).color(Color32::from_rgb(56, 189, 248)));
                ui.label("Automatically intercepts HLS (m3u8), DASH, and MP4 media with direct downloader.");
                if ui.button("Downloader Drawer").clicked() {
                    self.modal_state.show_video_downloader = true;
                }
            });

            // Card 3
            columns[2].group(|ui| {
                ui.heading(RichText::new("⚡ Text Expander").size(16.0).color(Color32::from_rgb(56, 189, 248)));
                ui.label("Expands shortcuts like :email, :date, :time everywhere in web inputs.");
                if ui.button("Snippets Manager").clicked() {
                    self.modal_state.show_text_expander = true;
                }
            });
        });

        ui.add_space(16.0);
        ui.columns(2, |columns| {
            // Card 4
            columns[0].group(|ui| {
                ui.heading(RichText::new("🧩 Chrome & Firefox Extensions").size(16.0).color(Color32::from_rgb(56, 189, 248)));
                ui.label("Supports Manifest V2 & V3 WebExtensions API (`chrome.*`, `browser.*`).");
                if ui.button("Extensions Manager").clicked() {
                    self.modal_state.show_extensions = true;
                }
            });

            // Card 5
            columns[1].group(|ui| {
                ui.heading(RichText::new("📑 Dual Tab Layouts").size(16.0).color(Color32::from_rgb(56, 189, 248)));
                ui.label("Switch anytime between Horizontal tabs and Vertical sidebar tree tabs (Ctrl+Shift+V).");
            });
        });
    }
}

impl Default for BrowserUi {
    fn default() -> Self {
        Self::new()
    }
}
