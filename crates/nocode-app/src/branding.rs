//! Quantum Point logo — window icon and UI.

const LOGO_PNG: &[u8] = include_bytes!("../assets/logo.png");

/// Taskbar / window icon (all platforms supported by winit).
pub fn window_icon() -> Option<egui::IconData> {
    let image = image::load_from_memory(LOGO_PNG).ok()?;
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    Some(egui::IconData {
        rgba: rgba.into_raw(),
        width,
        height,
    })
}

/// Branded logo for egui panels (launcher, toolbar).
pub fn logo_image(max_height: f32) -> egui::Image<'static> {
    egui::Image::new(egui::include_image!("../assets/logo.png")).max_height(max_height)
}
