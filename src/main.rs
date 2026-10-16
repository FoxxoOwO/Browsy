use browsy::app::BrowsyApp;

fn main() -> eframe::Result {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([720.0, 480.0])
            .with_title("Browsy - Servo Web Browser"),
        ..Default::default()
    };

    eframe::run_native(
        "Browsy",
        native_options,
        Box::new(|cc| Ok(Box::new(BrowsyApp::new(cc)))),
    )
}
