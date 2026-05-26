use egui::{Align, CentralPanel, Context, Frame, Layout, RichText, ScrollArea};
use nocode_core::{resolve_project_directory, ProjectMeta};
use std::path::Path;

use crate::branding::logo_image;

use super::state::Template;
use super::NoCodeApp;

impl NoCodeApp {
    pub(crate) fn ui_launcher(&mut self, ctx: &Context) {
        CentralPanel::default()
            .frame(Frame::NONE.fill(self.palette.bg))
            .show(ctx, |ui| {
                ui.add_space(28.0);
                ui.horizontal(|ui| {
                    ui.add_space(36.0);
                    ui.add(logo_image(72.0));
                    ui.add_space(16.0);
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("Quantum Point")
                                .size(36.0)
                                .strong()
                                .color(self.palette.text),
                        );
                        ui.label(
                            RichText::new("Visual Logic Engine")
                                .size(14.0)
                                .color(self.palette.accent),
                        );
                        ui.label(
                            RichText::new("Vizual yig'ish → universal IR → native kod")
                                .color(self.palette.muted),
                        );
                    });
                });
                ui.add_space(24.0);
                ui.horizontal(|ui| {
                    ui.add_space(36.0);
                    ui.label(RichText::new("Loyihalar").size(18.0).strong());
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .button(
                                RichText::new("+ Yangi loyiha")
                                    .strong()
                                    .color(egui::Color32::WHITE),
                            )
                            .clicked()
                        {
                            self.show_create = true;
                            self.suggest_folder_from_name();
                        }
                    });
                });
                ui.add_space(12.0);

                if let Some(err) = &self.store_error {
                    ui.horizontal(|ui| {
                        ui.add_space(36.0);
                        ui.colored_label(self.palette.danger, err);
                    });
                }

                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.add_space(32.0);
                            if self.projects.is_empty() {
                                ui.vertical(|ui| {
                                    ui.label(
                                        RichText::new("Hali loyiha yoâ€˜q.")
                                            .color(self.palette.muted),
                                    );
                                    if ui.button("Birinchi loyihani yarating").clicked() {
                                        self.show_create = true;
                                        self.suggest_folder_from_name();
                                    }
                                });
                            } else {
                                for meta in self.projects.clone() {
                                    self.project_card(ui, &meta);
                                    ui.add_space(12.0);
                                }
                            }
                        });
                    });
            });

        if self.show_create {
            egui::Window::new("Yangi loyiha")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("Loyiha nomi");
                    if ui.text_edit_singleline(&mut self.new_name).changed() {
                        self.suggest_folder_from_name();
                    }
                    ui.add_space(8.0);
                    ui.label("Joylashuv");
                    ui.label(
                        RichText::new(
                            "Documents yoki boshqa papka; ichida «Loyiha nomi» papkasi yaratiladi",
                        )
                        .small()
                        .color(self.palette.muted),
                    );
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.new_folder)
                                .hint_text("masalan: C:\\Users\\…\\Documents")
                                .desired_width(300.0),
                        );
                        if ui.button("Tanlash…").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_title("Loyiha joyi (Documents va hokazo)")
                                .pick_folder()
                            {
                                let name = self.new_name.trim();
                                if name.is_empty() {
                                    self.new_folder = path.to_string_lossy().to_string();
                                } else {
                                    self.new_folder = resolve_project_directory(&path, name)
                                        .to_string_lossy()
                                        .to_string();
                                }
                            }
                        }
                    });
                    if !self.new_name.trim().is_empty() && !self.new_folder.trim().is_empty() {
                        let resolved = resolve_project_directory(
                            Path::new(self.new_folder.trim()),
                            self.new_name.trim(),
                        );
                        ui.label(
                            RichText::new(format!("Yaratiladi: {}", resolved.display()))
                                .small()
                                .color(self.palette.accent),
                        );
                    }
                    ui.add_space(8.0);
                    ui.label("Shablon");
                    egui::ComboBox::from_id_salt("tpl")
                        .selected_text(match self.template {
                            Template::Empty => "Bo'sh (Start)",
                            Template::Hello => "Namuna (Log + Assign)",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.template,
                                Template::Empty,
                                "Bo'sh (Start)",
                            );
                            ui.selectable_value(
                                &mut self.template,
                                Template::Hello,
                                "Namuna (Log + Assign)",
                            );
                        });
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if ui.button("Bekor").clicked() {
                            self.show_create = false;
                        }
                        let can_create =
                            !self.new_name.trim().is_empty() && !self.new_folder.trim().is_empty();
                        let create = ui.add_enabled(
                            can_create,
                            egui::Button::new("Yaratish").fill(self.palette.accent),
                        );
                        if create.clicked() {
                            self.create_project();
                        }
                    });
                });
        }

        if let Some(id) = self.delete_confirm.clone() {
            egui::Window::new("O'chirish")
                .collapsible(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("Loyihani o'chirasizmi?");
                    ui.horizontal(|ui| {
                        if ui.button("Bekor").clicked() {
                            self.delete_confirm = None;
                        }
                        if ui
                            .button(RichText::new("O'chirish").color(self.palette.danger))
                            .clicked()
                        {
                            let id = id.clone();
                            match self.store.delete(&id) {
                                Ok(()) => {
                                    if self.project_id.as_deref() == Some(id.as_str()) {
                                        self.project_id = None;
                                        self.project_folder = None;
                                        self.project = None;
                                        self.screen = super::state::Screen::Launcher;
                                    }
                                    self.delete_confirm = None;
                                    self.store_error = None;
                                    self.reload_projects();
                                }
                                Err(e) => {
                                    self.store_error = Some(e.to_string());
                                    self.delete_confirm = None;
                                }
                            }
                        }
                    });
                });
        }
    }

    fn project_card(&mut self, ui: &mut egui::Ui, meta: &ProjectMeta) {
        let w = 260.0;
        Frame::new()
            .fill(self.palette.surface)
            .stroke(egui::Stroke::new(1.0, self.palette.border))
            .corner_radius(12.0)
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.set_width(w);
                ui.label(RichText::new(&meta.name).strong().size(15.0));
                ui.label(
                    RichText::new(format!("{} nod · {}", meta.node_count, meta.id))
                        .small()
                        .color(self.palette.muted),
                );
                ui.label(
                    RichText::new(meta.folder.display().to_string())
                        .small()
                        .color(self.palette.muted),
                );
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("Ochish →").color(egui::Color32::WHITE),
                            )
                            .fill(self.palette.success),
                        )
                        .clicked()
                    {
                        self.open_project(&meta.id);
                    }
                    if ui.button("O'chirish").clicked() {
                        self.delete_confirm = Some(meta.id.clone());
                    }
                });
            });
    }
}
