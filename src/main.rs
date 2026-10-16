use browsy::app::BrowsyApp;

fn main() -> eframe::Result {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Install process-level default Rustls CryptoProvider (aws-lc-rs) for network resource requests
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([720.0, 480.0])
            .with_title("Browsy - Servo Web Browser"),
        system_font_fallback: false,
        ..Default::default()
    };

    eframe::run_native(
        "Browsy",
        native_options,
        Box::new(|cc| Ok(Box::new(BrowsyApp::new(cc)))),
    )
}
