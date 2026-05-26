use egui::{Color32, CornerRadius, Stroke, Style, Visuals};

pub struct Palette {
    pub bg: Color32,
    pub panel: Color32,
    pub surface: Color32,
    pub border: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub accent: Color32,
    pub success: Color32,
    pub warn: Color32,
    pub danger: Color32,
    pub start: Color32,
    pub log: Color32,
    pub assign: Color32,
    pub if_node: Color32,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            bg: Color32::from_rgb(15, 17, 23),
            panel: Color32::from_rgb(22, 27, 38),
            surface: Color32::from_rgb(26, 32, 48),
            border: Color32::from_rgb(42, 49, 66),
            text: Color32::from_rgb(232, 234, 237),
            muted: Color32::from_rgb(148, 163, 184),
            accent: Color32::from_rgb(37, 99, 235),
            success: Color32::from_rgb(34, 197, 94),
            warn: Color32::from_rgb(245, 158, 11),
            danger: Color32::from_rgb(239, 68, 68),
            start: Color32::from_rgb(34, 197, 94),
            log: Color32::from_rgb(59, 130, 246),
            assign: Color32::from_rgb(168, 85, 247),
            if_node: Color32::from_rgb(245, 158, 11),
        }
    }
}

pub fn apply(ctx: &egui::Context, palette: &Palette) {
    let mut visuals = Visuals::dark();
    visuals.panel_fill = palette.panel;
    visuals.window_fill = palette.surface;
    visuals.extreme_bg_color = palette.bg;
    visuals.widgets.noninteractive.bg_fill = palette.surface;
    visuals.widgets.inactive.bg_fill = palette.surface;
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(36, 44, 62);
    visuals.widgets.active.bg_fill = palette.accent;
    visuals.selection.bg_fill = palette.accent.gamma_multiply(0.35);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, palette.text);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, palette.text);
    visuals.override_text_color = Some(palette.text);
    visuals.window_corner_radius = CornerRadius::same(10);
    ctx.set_visuals(visuals);

    let mut style = Style::default();
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);
    ctx.set_style(style);
}
