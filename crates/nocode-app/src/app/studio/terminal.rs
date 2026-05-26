use egui::{Context, Frame, RichText, ScrollArea, TopBottomPanel};

use crate::app::state::PipelineJobKind;
use crate::app::NoCodeApp;

impl NoCodeApp {
    pub(crate) fn ui_studio_terminal(&mut self, ctx: &Context, _id: &str) {
        TopBottomPanel::bottom("terminal")
            .resizable(true)
            .default_height(160.0)
            .frame(
                Frame::NONE
                    .fill(egui::Color32::from_rgb(10, 12, 16))
                    .inner_margin(10.0),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Terminal").strong().color(self.palette.muted));
                    if self.pipeline_job.is_busy() {
                        ui.spinner();
                        ui.label(match self.pipeline_job {
                            PipelineJobKind::Check => "Run (IR)…",
                            PipelineJobKind::Build => "Build…",
                            PipelineJobKind::BuildRun => "Build & Run…",
                            PipelineJobKind::Idle => "",
                        });
                    }
                });
                ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                    for line in &self.terminal {
                        ui.label(
                            RichText::new(line)
                                .family(egui::FontFamily::Monospace)
                                .size(12.0),
                        );
                    }
                });
            });
    }
}
