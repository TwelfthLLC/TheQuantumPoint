//! Studio UI — toolbar, properties, terminal, graph canvas.

mod center;
mod new_graph_popup;
mod properties;
mod terminal;
mod toolbar;

use egui::Context;

use super::NoCodeApp;

impl NoCodeApp {
    pub(crate) fn ui_studio(&mut self, ctx: &Context, id: &str) {
        self.poll_pipeline();
        self.ui_content_browser(ctx);
        self.ui_studio_toolbar(ctx, id);
        self.ui_studio_properties(ctx, id);
        self.ui_studio_terminal(ctx, id);
        self.ui_studio_new_graph_popup(ctx, id);
        self.ui_studio_center(ctx, id);
    }
}
