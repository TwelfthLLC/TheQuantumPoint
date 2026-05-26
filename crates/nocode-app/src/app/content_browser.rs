use egui::{
    collapsing_header::CollapsingState, Context, Frame, NumExt, Response, RichText, ScrollArea,
    Sense, TextStyle, TextWrapMode, Ui, Widget, WidgetInfo, WidgetText, WidgetType,
};
use nocode_core::{ContentItem, ContentItemKind, ContentSection};

use super::NoCodeApp;

impl NoCodeApp {
    pub(crate) fn reload_content_browser(&mut self) {
        let Some(folder) = self.project_folder.clone() else {
            self.content_sections.clear();
            return;
        };
        self.content_sections = nocode_core::scan_project_browser(&folder, &self.content_search);
    }

    pub(crate) fn ui_content_browser(&mut self, ctx: &Context) {
        egui::SidePanel::left("nocode_content_browser")
            .resizable(true)
            .default_width(260.0)
            .min_width(160.0)
            .frame(Frame::NONE.fill(self.palette.panel).inner_margin(12.0))
            .show(ctx, |ui| {
                paint_panel_body(self, ui);
            });
    }

    pub(crate) fn on_content_item_clicked(&mut self, item: &ContentItem) {
        if !item.openable || item.path.is_empty() {
            return;
        }
        match item.kind {
            ContentItemKind::Graph => {
                self.switch_graph_file(item.path.clone());
            }
            ContentItemKind::Build => {
                self.open_generated_main_rs(&item.path);
            }
            _ => {}
        }
    }

    fn open_generated_main_rs(&mut self, relative_path: &str) {
        let Some(folder) = self.project_folder.clone() else {
            return;
        };
        let path = folder.join(relative_path.replace('/', std::path::MAIN_SEPARATOR_STR));
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                self.generated_main = text;
                self.show_generated = true;
            }
            Err(e) => {
                self.terminal.push(format!("main.rs o‘qib bo‘lmadi: {e}"));
            }
        }
    }

    pub(crate) fn on_content_item_secondary(&mut self, item: &ContentItem) {
        if item.kind != ContentItemKind::Graph || item.path.is_empty() {
            return;
        }
        if self.dirty {
            self.save_active_graph();
        }
        self.active_graph = item.path.clone();
        self.set_entry_graph_file();
        self.reload_content_browser();
    }
}

fn panel_inner_width(ui: &Ui) -> f32 {
    ui.max_rect().width()
}

fn constrain_width(ui: &mut Ui, width: f32) {
    ui.set_min_width(0.0);
    ui.set_max_width(width);
}

fn paint_panel_body(app: &mut NoCodeApp, ui: &mut Ui) {
    let content_w = panel_inner_width(ui);
    constrain_width(ui, content_w);

    ui.label(RichText::new("Fayllar").strong());
    ui.add_space(8.0);

    let refresh_w = ui.spacing().interact_size.x;
    let gap = ui.spacing().item_spacing.x;
    let search_w = (content_w - refresh_w - gap).max(48.0);
    let row_h = ui.spacing().interact_size.y;

    ui.horizontal(|ui| {
        constrain_width(ui, content_w);
        if ui
            .add_sized(
                [search_w, row_h],
                egui::TextEdit::singleline(&mut app.content_search)
                    .hint_text("Qidiruv…")
                    .clip_text(true),
            )
            .changed()
        {
            app.reload_content_browser();
        }
        if ui.button("↻").on_hover_text("Yangilash (F5)").clicked() {
            app.reload_content_browser();
        }
    });
    if ui.input(|i| i.key_pressed(egui::Key::F5)) {
        app.reload_content_browser();
    }
    if ui
        .add_sized([content_w, row_h], egui::Button::new("+ Graf fayl").small())
        .on_hover_text("Yangi graphs/*.qp")
        .clicked()
    {
        app.show_new_graph_popup = true;
    }

    ui.separator();

    let sections = app.content_sections.clone();
    let active = app.active_graph.clone();
    let entry = app
        .project_id
        .as_ref()
        .and_then(|id| app.store.entry_graph_path(id).ok())
        .unwrap_or_default();

    ScrollArea::vertical()
        .id_salt("content_browser_list")
        .max_width(content_w)
        .show(ui, |ui| {
            constrain_width(ui, content_w);
            if sections.is_empty() {
                ui.label(
                    RichText::new("Fayl topilmadi")
                        .small()
                        .color(app.palette.muted),
                );
                return;
            }
            for section in &sections {
                paint_section(ui, app, section, &active, &entry, content_w);
                ui.add_space(4.0);
            }
        });
}

fn paint_section(
    ui: &mut Ui,
    app: &mut NoCodeApp,
    section: &ContentSection,
    active_path: &str,
    entry_path: &str,
    content_w: f32,
) {
    constrain_width(ui, content_w);
    let id = ui.make_persistent_id(("content_section", section.title.as_str()));
    let header =
        CollapsingState::load_with_default_open(ui.ctx(), id, true).show_header(ui, |ui| {
            constrain_width(ui, content_w);
            ui.add(egui::Label::new(RichText::new(&section.title).strong()).truncate());
        });
    header.body_unindented(|ui| {
        constrain_width(ui, content_w);
        for item in &section.items {
            paint_item(ui, app, item, active_path, entry_path, 0, content_w);
        }
    });
}

fn paint_item(
    ui: &mut Ui,
    app: &mut NoCodeApp,
    item: &ContentItem,
    active_path: &str,
    entry_path: &str,
    depth: usize,
    content_w: f32,
) {
    let indent = 12.0 + depth as f32 * 14.0;
    let (icon, color) = item_style(item.kind, &app.palette);
    let selected = item.openable && item.path == active_path;
    let is_entry = item.path == entry_path && item.kind == ContentItemKind::Graph;

    let label = if is_entry && !item.name.starts_with('★') {
        format!("★ {}", item.name)
    } else {
        item.name.clone()
    };
    let text = RichText::new(format!("{icon} {label}")).color(color);

    let row_h = ui.spacing().interact_size.y;
    let row_w = (content_w - indent - ui.spacing().item_spacing.x).max(1.0);

    ui.horizontal(|ui| {
        constrain_width(ui, content_w);
        ui.add_space(indent);
        let resp = ui
            .allocate_ui(egui::vec2(row_w, row_h), |ui| {
                ui.set_width(row_w);
                ui.set_max_width(row_w);
                ui.add(selectable_label_truncated(selected, text, row_w))
            })
            .inner;
        if resp.clicked() {
            app.on_content_item_clicked(item);
        }
        if resp.secondary_clicked() && item.kind == ContentItemKind::Graph {
            app.on_content_item_secondary(item);
        }
        if resp.hovered() && item.openable {
            let hint = match item.kind {
                ContentItemKind::Build => format!("{}\nOchish — o‘ng panelda main.rs", item.path),
                ContentItemKind::Graph => format!("{}\nOchish · o‘ng tugma = Entry", item.path),
                _ => item.path.clone(),
            };
            resp.on_hover_text(hint);
        }
    });

    for child in &item.children {
        paint_item(
            ui,
            app,
            child,
            active_path,
            entry_path,
            depth + 1,
            content_w,
        );
    }
}

struct TruncatedSelectableLabel {
    selected: bool,
    text: WidgetText,
    max_width: f32,
}

fn selectable_label_truncated(
    selected: bool,
    text: impl Into<WidgetText>,
    max_width: f32,
) -> TruncatedSelectableLabel {
    TruncatedSelectableLabel {
        selected,
        text: text.into(),
        max_width,
    }
}

impl Widget for TruncatedSelectableLabel {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self {
            selected,
            text,
            max_width,
        } = self;
        let button_padding = ui.spacing().button_padding;
        let total_extra = button_padding + button_padding;
        let wrap_width = (max_width - total_extra.x).max(0.0);
        let galley = text.into_galley(
            ui,
            Some(TextWrapMode::Truncate),
            wrap_width,
            TextStyle::Button,
        );
        let mut desired_size = total_extra + galley.size();
        desired_size.x = desired_size.x.min(max_width);
        desired_size.y = desired_size.y.at_least(ui.spacing().interact_size.y);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());
        response.widget_info(|| {
            WidgetInfo::selected(
                WidgetType::SelectableLabel,
                ui.is_enabled(),
                selected,
                galley.text(),
            )
        });
        if ui.is_rect_visible(response.rect) {
            let text_pos = ui
                .layout()
                .align_size_within_rect(galley.size(), rect.shrink2(button_padding))
                .min;
            let visuals = ui.style().interact_selectable(&response, selected);
            if selected || response.hovered() || response.highlighted() || response.has_focus() {
                let rect = rect.expand(visuals.expansion);
                ui.painter().rect(
                    rect,
                    visuals.corner_radius,
                    visuals.weak_bg_fill,
                    visuals.bg_stroke,
                    egui::StrokeKind::Inside,
                );
            }
            ui.painter().galley(text_pos, galley, visuals.text_color());
        }
        response
    }
}

fn item_style(
    kind: ContentItemKind,
    palette: &crate::theme::Palette,
) -> (&'static str, egui::Color32) {
    match kind {
        ContentItemKind::Graph => ("◆", palette.log),
        ContentItemKind::Config => ("⚙", palette.muted),
        ContentItemKind::Doc => ("📄", palette.muted),
        ContentItemKind::Build => ("▣", palette.warn),
        ContentItemKind::Folder => ("📁", palette.muted),
        ContentItemKind::Other => ("·", palette.muted),
    }
}
