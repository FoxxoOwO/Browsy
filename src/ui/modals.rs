use eframe::egui::{self, Color32, RichText};

use crate::adblock::AdblockEngine;
use crate::extensions::ExtensionManager;
use crate::text_expander::TextExpander;
use crate::video_downloader::{DownloadState, VideoDownloader};

/// State for open dialogs and modals
#[derive(Default)]
pub struct ModalState {
    pub show_adblock: bool,
    pub show_video_downloader: bool,
    pub show_text_expander: bool,
    pub show_extensions: bool,
    pub show_settings: bool,

    // Form inputs for modals
    pub new_snippet_trigger: String,
    pub new_snippet_replacement: String,
    pub new_snippet_desc: String,
    pub expander_test_input: String,

    pub custom_adblock_rule: String,
    pub unpack_ext_path: String,
}

/// Render the uBlock Origin Adblock Shield modal
pub fn show_adblock_modal(
    ctx: &egui::Context,
    modal_state: &mut ModalState,
    adblock: &AdblockEngine,
    current_url: &str,
) {
    if !modal_state.show_adblock {
        return;
    }

    egui::Window::new("uBlock Origin Shield")
        .open(&mut modal_state.show_adblock)
        .resizable(true)
        .default_width(550.0)
        .default_height(480.0)
        .show(ctx, |ui| {
            let mut enabled = adblock.is_enabled();
            let domain = url::Url::parse(current_url)
                .ok()
                .and_then(|u| u.host_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "current site".to_string());

            let is_allowlisted = adblock.is_allowlisted(&domain);

            ui.horizontal(|ui| {
                ui.heading(RichText::new("Shield Protection").strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let label = if enabled { "ENABLED" } else { "DISABLED" };
                    if ui.add(egui::Checkbox::new(&mut enabled, label)).changed() {
                        adblock.set_enabled(enabled);
                    }
                });
            });

            ui.separator();

            // Stats row
            let stats = adblock.stats();
            let page_blocked = adblock.blocked_for_url(current_url);

            ui.horizontal(|ui| {
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new(format!("{}", stats.total_blocked)).size(24.0).strong().color(Color32::from_rgb(56, 189, 248)));
                        ui.label("Total Ads & Trackers Blocked");
                    });
                });
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new(format!("{}", page_blocked)).size(24.0).strong().color(Color32::from_rgb(34, 197, 94)));
                        ui.label(format!("Blocked on {}", domain));
                    });
                });
            });

            ui.add_space(8.0);

            // Per-site toggle
            ui.horizontal(|ui| {
                ui.label(format!("Protection for {}: ", domain));
                let allow_label = if is_allowlisted { "❌ Allowlisted (Shields OFF)" } else { "✅ Protected (Shields ON)" };
                if ui.button(allow_label).clicked() {
                    adblock.toggle_allowlist(&domain);
                }
            });

            ui.separator();

            // Custom rule adder
            ui.collapsing("➕ Add Custom uBlock / ABP Filter Rule", |ui| {
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut modal_state.custom_adblock_rule).hint_text("e.g. ||example.com/ads/* or ##.banner"));
                    if ui.button("Add Rule").clicked() && !modal_state.custom_adblock_rule.trim().is_empty() {
                        adblock.add_rules(&[modal_state.custom_adblock_rule.trim().to_string()]);
                        modal_state.custom_adblock_rule.clear();
                    }
                });
            });

            ui.add_space(8.0);
            ui.label(RichText::new("Recent Blocked Requests:").strong());

            egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                if stats.blocked_history.is_empty() {
                    ui.label("No blocked requests recorded yet.");
                } else {
                    for record in stats.blocked_history.iter().rev().take(30) {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(&record.resource_type).size(11.0).color(Color32::from_rgb(251, 146, 60)));
                                ui.label(RichText::new(&record.domain).strong());
                                if let Some(rule) = &record.matched_rule {
                                    ui.label(RichText::new(format!("[{}]", rule)).size(11.0).color(Color32::GRAY));
                                }
                            });
                            ui.label(RichText::new(&record.url).size(10.0).color(Color32::LIGHT_GRAY));
                        });
                    }
                }
            });
        });
}

/// Render the Video Downloader drawer / modal
pub fn show_video_downloader_modal(
    ctx: &egui::Context,
    modal_state: &mut ModalState,
    downloader: &VideoDownloader,
    current_page_url: &str,
    current_title: &str,
) {
    if !modal_state.show_video_downloader {
        return;
    }

    egui::Window::new("Video & Stream Downloader")
        .open(&mut modal_state.show_video_downloader)
        .resizable(true)
        .default_width(620.0)
        .default_height(500.0)
        .show(ctx, |ui| {
            let ytdlp_available = downloader.check_ytdlp_available();

            ui.horizontal(|ui| {
                ui.label(format!("Save folder: {}", downloader.download_dir().display()));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Open Folder").clicked() {
                        let _ = std::process::Command::new("explorer")
                            .arg(downloader.download_dir())
                            .spawn();
                    }
                });
            });

            ui.separator();

            // Detected streams on current page
            let detected = downloader.streams_for_page(current_page_url);
            ui.heading(RichText::new(format!("Detected Media on this Page ({})", detected.len())).size(16.0));

            if detected.is_empty() {
                ui.label("No video or audio streams detected on the current tab yet.");
                ui.label("Play a video on the page or inspect the network stream.");
                
                // Allow fallback yt-dlp download if page URL is http
                if current_page_url.starts_with("http") {
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("Download Page Video with yt-dlp").clicked() {
                            let _ = downloader.start_ytdlp_download(current_page_url, current_title);
                        }
                        if !ytdlp_available {
                            ui.label(RichText::new("(Note: yt-dlp was not found in PATH)").color(Color32::YELLOW).size(11.0));
                        }
                    });
                }
            } else {
                egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                    for stream in detected {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(stream.kind.label()).strong().color(Color32::from_rgb(56, 189, 248)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("⬇ Download").clicked() {
                                        let _ = downloader.start_download(stream.id);
                                    }
                                });
                            });
                            ui.label(RichText::new(&stream.stream_url).size(11.0).color(Color32::LIGHT_GRAY));
                        });
                    }
                });
            }

            ui.add_space(10.0);
            ui.separator();
            ui.heading(RichText::new("Active & Completed Downloads").size(16.0));

            let tasks = downloader.all_downloads();
            if tasks.is_empty() {
                ui.label("No active downloads.");
            } else {
                egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                    for task in tasks {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(&task.filename).strong());
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    match &task.state {
                                        DownloadState::Completed { file_path } => {
                                            ui.label(RichText::new("✅ Completed").color(Color32::GREEN));
                                            if ui.button("Open").clicked() {
                                                let _ = std::process::Command::new("explorer")
                                                    .arg("/select,")
                                                    .arg(file_path)
                                                    .spawn();
                                            }
                                        }
                                        DownloadState::Downloading { percentage, speed_kbps, .. } => {
                                            ui.label(format!("{:.1}% ({:.0} kB/s)", percentage, speed_kbps));
                                        }
                                        DownloadState::Failed { error } => {
                                            ui.label(RichText::new(format!("Failed: {}", error)).color(Color32::RED));
                                        }
                                        DownloadState::Queued => {
                                            ui.label("Queued");
                                        }
                                        DownloadState::Cancelled => {
                                            ui.label("Cancelled");
                                        }
                                    }
                                });
                            });

                            if let DownloadState::Downloading { percentage, .. } = task.state {
                                ui.add(egui::ProgressBar::new(percentage / 100.0).show_percentage());
                            }
                        });
                    }
                });
            }
        });
}

/// Render the Text Expander manager modal
pub fn show_text_expander_modal(
    ctx: &egui::Context,
    modal_state: &mut ModalState,
    expander: &TextExpander,
) {
    if !modal_state.show_text_expander {
        return;
    }

    egui::Window::new("Text Expander Snippets")
        .open(&mut modal_state.show_text_expander)
        .resizable(true)
        .default_width(650.0)
        .default_height(520.0)
        .show(ctx, |ui| {
            ui.label("Type trigger shortcuts (e.g. :email, :date) in webpage inputs or the address bar to auto-expand.");
            ui.separator();

            // Add new snippet form
            ui.group(|ui| {
                ui.label(RichText::new("Add New Snippet").strong());
                ui.horizontal(|ui| {
                    ui.label("Trigger:");
                    ui.add(egui::TextEdit::singleline(&mut modal_state.new_snippet_trigger).desired_width(80.0).hint_text(":prefix"));
                    ui.label("Replacement:");
                    ui.add(egui::TextEdit::singleline(&mut modal_state.new_snippet_replacement).desired_width(180.0).hint_text("text or {date}"));
                    ui.label("Description:");
                    ui.add(egui::TextEdit::singleline(&mut modal_state.new_snippet_desc).desired_width(120.0));
                    if ui.button("+ Add").clicked() {
                        if !modal_state.new_snippet_trigger.trim().is_empty() {
                            expander.add_snippet(
                                &modal_state.new_snippet_trigger,
                                &modal_state.new_snippet_replacement,
                                &modal_state.new_snippet_desc,
                            );
                            modal_state.new_snippet_trigger.clear();
                            modal_state.new_snippet_replacement.clear();
                            modal_state.new_snippet_desc.clear();
                        }
                    }
                });
                ui.label(RichText::new("Variables supported: {date}, {time}, {datetime}").size(11.0).color(Color32::GRAY));
            });

            ui.add_space(8.0);
            ui.heading(RichText::new("Active Snippets").size(16.0));

            let snippets = expander.all_snippets();
            egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                for s in snippets {
                    ui.horizontal(|ui| {
                        let mut enabled = s.enabled;
                        if ui.checkbox(&mut enabled, "").changed() {
                            expander.toggle_enabled(s.id);
                        }
                        ui.label(RichText::new(&s.trigger).strong().color(Color32::from_rgb(56, 189, 248)));
                        ui.label("->");
                        ui.label(RichText::new(&s.replacement).color(Color32::WHITE));
                        ui.label(format!("({})", s.description));

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Delete").clicked() {
                                expander.remove_snippet(s.id);
                            }
                        });
                    });
                }
            });

            ui.separator();
            ui.label(RichText::new("Test Snippets Here:").strong());
            ui.horizontal(|ui| {
                let edit = ui.add(
                    egui::TextEdit::singleline(&mut modal_state.expander_test_input)
                        .hint_text("Type :email, :date, :time, :shrug or your snippet here..."),
                );
                if edit.changed() {
                    if let Some(expanded) = expander.try_expand_input(&modal_state.expander_test_input) {
                        modal_state.expander_test_input = expanded;
                    }
                }
                if ui.button("Clear").clicked() {
                    modal_state.expander_test_input.clear();
                }
            });
        });
}

/// Render the WebExtensions manager modal
pub fn show_extensions_modal(
    ctx: &egui::Context,
    modal_state: &mut ModalState,
    manager: &ExtensionManager,
) {
    if !modal_state.show_extensions {
        return;
    }

    egui::Window::new("Firefox & Chrome WebExtensions")
        .open(&mut modal_state.show_extensions)
        .resizable(true)
        .default_width(600.0)
        .default_height(480.0)
        .show(ctx, |ui| {
            ui.label("Compatible with standard Manifest V2 & Manifest V3 WebExtensions from Chrome and Firefox.");
            ui.separator();

            // Load unpacked extension form
            ui.horizontal(|ui| {
                ui.label("Load Unpacked Extension:");
                ui.add(egui::TextEdit::singleline(&mut modal_state.unpack_ext_path).desired_width(280.0).hint_text("Folder path containing manifest.json"));
                if ui.button("Load").clicked() && !modal_state.unpack_ext_path.trim().is_empty() {
                    let path = std::path::PathBuf::from(&modal_state.unpack_ext_path);
                    match manager.load_unpacked(&path) {
                        Ok(id) => log::info!("Loaded extension: {}", id),
                        Err(e) => log::error!("Failed to load extension: {}", e),
                    }
                    modal_state.unpack_ext_path.clear();
                }
            });

            ui.add_space(8.0);
            ui.separator();
            ui.heading(RichText::new("Installed Extensions").size(16.0));

            let exts = manager.list_extensions();
            if exts.is_empty() {
                ui.label("No extensions installed.");
            } else {
                egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                    for ext in exts {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(&ext.name).strong().size(15.0).color(Color32::from_rgb(56, 189, 248)));
                                ui.label(RichText::new(format!("v{}", ext.version)).size(12.0).color(Color32::GRAY));

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let label = if ext.enabled { "Active" } else { "Disabled" };
                                    let mut enabled = ext.enabled;
                                    if ui.checkbox(&mut enabled, label).changed() {
                                        manager.toggle_extension(&ext.id);
                                    }
                                });
                            });

                            if !ext.description.is_empty() {
                                ui.label(&ext.description);
                            }

                            if !ext.permissions.is_empty() {
                                ui.label(RichText::new(format!("Permissions: {}", ext.permissions.join(", "))).size(11.0).color(Color32::LIGHT_GRAY));
                            }

                            ui.label(RichText::new(format!("Location: {}", ext.root_path.display())).size(10.0).color(Color32::DARK_GRAY));
                        });
                    }
                });
            }
        });
}
