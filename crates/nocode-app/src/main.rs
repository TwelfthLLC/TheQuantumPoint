mod app;
mod branding;
mod graph;
mod theme;

use app::NoCodeApp;
use branding::window_icon;

fn main() -> eframe::Result<()> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1280.0, 800.0])
        .with_min_inner_size([900.0, 600.0])
        .with_title("Quantum Point");
    if let Some(icon) = window_icon() {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "Quantum Point",
        options,
        Box::new(|cc| Ok(Box::new(NoCodeApp::new(cc)))),
    )
}
