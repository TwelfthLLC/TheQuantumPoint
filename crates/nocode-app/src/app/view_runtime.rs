use graph_model::GraphLayer;
use qp_view_runtime::ViewRuntimeTheme;

use super::NoCodeApp;

impl NoCodeApp {
    pub(crate) fn sync_view_runtime(&mut self) {
        let Some(p) = self.project.as_ref() else {
            self.view_runtime = None;
            return;
        };
        if p.layer != GraphLayer::View {
            self.view_runtime = None;
            return;
        }
        self.view_runtime = Some(qp_view_runtime::build_from_project(p));
    }

    pub(crate) fn view_runtime_theme(&self) -> ViewRuntimeTheme {
        ViewRuntimeTheme {
            accent: self.palette.accent,
            muted: self.palette.muted,
            panel: self.palette.surface,
        }
    }

    pub(crate) fn show_view_runtime_panel(&mut self, ui: &mut egui::Ui) {
        let Some(rt) = self.view_runtime.clone() else {
            ui.label(
                egui::RichText::new("View graf oching yoki nod qo‘shing")
                    .small()
                    .color(self.palette.muted),
            );
            return;
        };
        let theme = self.view_runtime_theme();
        qp_view_runtime::show_runtime(ui, &rt, &mut self.view_runtime_state, &theme);
    }
}
