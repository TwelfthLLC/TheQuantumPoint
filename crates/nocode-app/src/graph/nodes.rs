use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use graph_model::{
    Node, NODE_API_QUERY, NODE_API_ROUTE, NODE_ASSIGN, NODE_ASYNC, NODE_AWAIT, NODE_BREAK,
    NODE_CALL, NODE_CONST, NODE_CONTINUE, NODE_DB_READ, NODE_EMIT_UI, NODE_ENUM, NODE_EXPR,
    NODE_FOR, NODE_FOREACH, NODE_FUNCTION, NODE_IF, NODE_IMPORT, NODE_LIST, NODE_LOG, NODE_RETURN,
    NODE_START, NODE_STRUCT, NODE_SUBGRAPH, NODE_SWITCH, NODE_THROW, NODE_TRY, NODE_UI_BUTTON,
    NODE_UI_EVENT, NODE_UI_INPUT, NODE_UI_LABEL, NODE_UI_PAGE, NODE_WHILE,
};

use super::edges::handle_color;
use super::editor::GraphEditor;
use super::types::{HandleKind, NodeAction, NODE_W};
use crate::theme::Palette;

pub(crate) fn scaled_font(size: f32, z: f32) -> egui::FontId {
    egui::FontId::proportional((size * z).clamp(8.0, 22.0))
}

pub(crate) fn paint_node(
    ui: &mut Ui,
    painter: &egui::Painter,
    editor: &GraphEditor,
    node: &Node,
    rect: Rect,
    selected: bool,
    palette: &Palette,
) -> NodeAction {
    let z = rect.width() / NODE_W;
    let color = node_color(node, palette);
    let corner = (10.0 * z).max(2.0);
    let stroke = if selected {
        Stroke::new((2.5 * z).max(1.0), palette.accent)
    } else {
        Stroke::new((1.5 * z).max(1.0), color)
    };
    painter.rect_filled(rect, corner, palette.surface);
    painter.rect_stroke(rect, corner, stroke, egui::StrokeKind::Outside);

    let title_h = 24.0 * z;
    let title_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), title_h));
    painter.rect_filled(title_rect, corner, color);
    let pad = 10.0 * z;
    painter.text(
        title_rect.left_top() + Vec2::new(pad, 5.0 * z),
        egui::Align2::LEFT_TOP,
        node_title(node),
        scaled_font(13.0, z),
        Color32::BLACK,
    );

    let body = node_summary(node);
    painter.text(
        rect.left_top() + Vec2::new(pad, title_h + 6.0 * z),
        egui::Align2::LEFT_TOP,
        body,
        scaled_font(12.0, z),
        palette.muted,
    );

    let mut action = NodeAction::None;
    let drag_rect = rect.shrink2(Vec2::splat((3.0 * z).max(1.0)));
    let drag_resp = ui.allocate_rect(drag_rect, Sense::click_and_drag());
    if drag_resp.clicked() && !drag_resp.dragged() {
        let additive = ui.input(|i| i.modifiers.shift);
        action = NodeAction::Select { additive };
    }
    if drag_resp.drag_started_by(egui::PointerButton::Primary) {
        if let Some(pos) = drag_resp.interact_pointer_pos() {
            let w = editor.screen_to_world(pos);
            action = NodeAction::DragStart {
                offset: Vec2::new(w.x - node.position.x as f32, w.y - node.position.y as f32),
            };
        }
    }

    for (kind, hit) in handle_hit_zones(node, rect, z) {
        let center = hit.center();
        paint_port_stub(painter, rect, center, kind, color, z);
        let r = ui.allocate_rect(hit, Sense::click_and_drag());
        paint_handle(painter, center, kind, color, palette, z);
        if r.secondary_clicked() {
            action = NodeAction::HandleDisconnect(kind);
        } else if r.clicked() || r.drag_started_by(egui::PointerButton::Primary) {
            action = NodeAction::HandleClick(kind);
        }
    }

    action
}

pub(crate) fn handle_hit_zones(node: &Node, rect: Rect, z: f32) -> Vec<(HandleKind, Rect)> {
    let s = (16.0 * z).max(8.0);
    let mut out = Vec::new();
    match node.kind.as_str() {
        NODE_START => {
            out.push((
                HandleKind::ExecOut,
                handle_rect(Pos2::new(rect.right(), rect.center().y), s),
            ));
        }
        NODE_IF => {
            push_if_ports(&mut out, rect, s);
        }
        NODE_SWITCH => {
            out.push((
                HandleKind::ExecIn,
                handle_rect(Pos2::new(rect.left(), rect.center().y), s),
            ));
            let n = switch_case_port_count(node);
            for i in 0..n {
                let y = rect.top() + rect.height() * ((i as f32 + 1.0) / (n as f32 + 2.0));
                out.push((
                    super::edges::case_handle_from_index(i),
                    handle_rect(Pos2::new(rect.right(), y), s),
                ));
            }
            out.push((
                HandleKind::DefaultOut,
                handle_rect(
                    Pos2::new(
                        rect.right(),
                        rect.top() + rect.height() * (n as f32 + 1.0) / (n as f32 + 2.0),
                    ),
                    s,
                ),
            ));
            out.push((
                HandleKind::DoneOut,
                handle_rect(Pos2::new(rect.center().x, rect.bottom()), s),
            ));
        }
        NODE_TRY => {
            out.push((
                HandleKind::ExecIn,
                handle_rect(Pos2::new(rect.left(), rect.center().y), s),
            ));
            out.push((
                HandleKind::TryOut,
                handle_rect(Pos2::new(rect.right(), rect.top() + rect.height() * 0.3), s),
            ));
            out.push((
                HandleKind::CatchOut,
                handle_rect(Pos2::new(rect.right(), rect.top() + rect.height() * 0.7), s),
            ));
            out.push((
                HandleKind::DoneOut,
                handle_rect(Pos2::new(rect.center().x, rect.bottom()), s),
            ));
        }
        NODE_WHILE | NODE_FOR | NODE_FOREACH | NODE_ASYNC | NODE_FUNCTION => {
            out.push((
                HandleKind::ExecIn,
                handle_rect(Pos2::new(rect.left(), rect.center().y), s),
            ));
            out.push((
                HandleKind::BodyOut,
                handle_rect(
                    Pos2::new(rect.right(), rect.top() + rect.height() * 0.35),
                    s,
                ),
            ));
            out.push((
                HandleKind::DoneOut,
                handle_rect(Pos2::new(rect.center().x, rect.bottom()), s),
            ));
        }
        NODE_RETURN => {
            out.push((
                HandleKind::ExecIn,
                handle_rect(Pos2::new(rect.left(), rect.center().y), s),
            ));
        }
        _ => {
            out.push((
                HandleKind::ExecIn,
                handle_rect(Pos2::new(rect.left(), rect.center().y), s),
            ));
            out.push((
                HandleKind::ExecOut,
                handle_rect(Pos2::new(rect.right(), rect.center().y), s),
            ));
        }
    }
    out
}

fn handle_rect(center: Pos2, size: f32) -> Rect {
    Rect::from_center_size(center, Vec2::splat(size))
}

fn push_if_ports(out: &mut Vec<(HandleKind, Rect)>, rect: Rect, s: f32) {
    out.push((
        HandleKind::ExecIn,
        handle_rect(Pos2::new(rect.left(), rect.center().y), s),
    ));
    out.push((
        HandleKind::TrueOut,
        handle_rect(Pos2::new(rect.right(), rect.top() + rect.height() * 0.3), s),
    ));
    out.push((
        HandleKind::FalseOut,
        handle_rect(Pos2::new(rect.right(), rect.top() + rect.height() * 0.7), s),
    ));
    out.push((
        HandleKind::DoneOut,
        handle_rect(Pos2::new(rect.center().x, rect.bottom()), s),
    ));
}

fn paint_port_stub(
    painter: &egui::Painter,
    node_rect: Rect,
    center: Pos2,
    kind: HandleKind,
    color: Color32,
    z: f32,
) {
    let stub = match kind {
        HandleKind::ExecIn => [Pos2::new(node_rect.left(), center.y), center],
        HandleKind::ExecOut
        | HandleKind::TrueOut
        | HandleKind::FalseOut
        | HandleKind::BodyOut
        | HandleKind::TryOut
        | HandleKind::Case1Out
        | HandleKind::Case2Out
        | HandleKind::Case3Out
        | HandleKind::Case4Out
        | HandleKind::Case5Out
        | HandleKind::Case6Out
        | HandleKind::DefaultOut
        | HandleKind::CatchOut => [center, Pos2::new(node_rect.right(), center.y)],
        HandleKind::DoneOut => [center, Pos2::new(center.x, node_rect.bottom())],
    };
    painter.line_segment(
        stub,
        Stroke::new((2.0 * z).max(1.0), color.gamma_multiply(0.55)),
    );
}

fn paint_handle(
    painter: &egui::Painter,
    center: Pos2,
    kind: HandleKind,
    node_color: Color32,
    palette: &Palette,
    z: f32,
) {
    let ring = handle_color(kind, palette);
    let r = (7.0 * z).max(3.0);
    let inner = (3.0 * z).max(1.5);
    painter.circle_filled(center, r, ring);
    painter.circle_stroke(center, r, Stroke::new((2.0 * z).max(1.0), Color32::WHITE));
    painter.circle_filled(center, inner, node_color);
}

fn node_color(node: &Node, p: &Palette) -> Color32 {
    match node.kind.as_str() {
        NODE_START => p.start,
        NODE_LOG => p.log,
        NODE_ASSIGN => p.assign,
        NODE_IF | NODE_WHILE | NODE_FOR | NODE_FOREACH | NODE_SWITCH => p.if_node,
        NODE_RETURN | NODE_BREAK => p.danger,
        NODE_CONTINUE | NODE_ASYNC | NODE_AWAIT => p.accent,
        NODE_TRY | NODE_EXPR | NODE_LIST => p.warn,
        NODE_FUNCTION | NODE_CALL | NODE_CONST => p.assign,
        NODE_THROW => p.danger,
        NODE_STRUCT | NODE_ENUM | NODE_IMPORT => p.success,
        NODE_UI_PAGE => p.accent,
        NODE_UI_BUTTON => p.success,
        NODE_UI_LABEL => p.log,
        NODE_UI_INPUT => p.warn,
        NODE_UI_EVENT => p.if_node,
        NODE_API_ROUTE => p.accent,
        NODE_API_QUERY => p.warn,
        NODE_DB_READ => p.success,
        NODE_EMIT_UI => p.danger,
        _ => p.muted,
    }
}

fn node_title(node: &Node) -> &'static str {
    match node.kind.as_str() {
        NODE_START => "Start",
        NODE_LOG => "Log",
        NODE_ASSIGN => "Assign",
        NODE_IF => "If",
        NODE_WHILE => "While",
        NODE_FOR => "For",
        NODE_FOREACH => "Foreach",
        NODE_RETURN => "Return",
        NODE_SWITCH => "Switch",
        NODE_BREAK => "Break",
        NODE_CONTINUE => "Continue",
        NODE_TRY => "Try",
        NODE_EXPR => "Expr",
        NODE_ASYNC => "Async",
        NODE_FUNCTION => "Function",
        NODE_CALL => "Call",
        NODE_CONST => "Const",
        NODE_LIST => "List",
        NODE_THROW => "Throw",
        NODE_AWAIT => "Await",
        NODE_IMPORT => "Import",
        NODE_STRUCT => "Struct",
        NODE_ENUM => "Enum",
        NODE_UI_PAGE => "Page",
        NODE_UI_BUTTON => "Button",
        NODE_UI_LABEL => "Label",
        NODE_UI_INPUT => "Input",
        NODE_UI_EVENT => "Event",
        NODE_API_ROUTE => "API Route",
        NODE_API_QUERY => "API Query",
        NODE_DB_READ => "DB Read",
        NODE_SUBGRAPH => "Subgraph",
        NODE_EMIT_UI => "Emit UI",
        _ => "Node",
    }
}

fn node_summary(node: &Node) -> String {
    match node.kind.as_str() {
        NODE_LOG => data_str(node, "message", "…"),
        NODE_ASSIGN => format!("{} = {}", data_str(node, "name", "x"), data_i64(node)),
        NODE_IF | NODE_WHILE => data_str(node, "condition", "true"),
        NODE_FOR => format!(
            "{} = {}..{}",
            data_str(node, "var", "i"),
            graph_model::data_get_i64(&node.data, "from").unwrap_or(0),
            graph_model::data_get_i64(&node.data, "to").unwrap_or(0)
        ),
        NODE_FOREACH => format!(
            "{} in {}",
            data_str(node, "item_var", "item"),
            data_str(node, "collection", "users")
        ),
        NODE_RETURN => data_str(node, "value", ""),
        NODE_SWITCH => {
            let cases = graph_model::data_get_str(&node.data, "cases")
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| {
                    format!(
                        "{},{}",
                        data_str(node, "case1", "1"),
                        data_str(node, "case2", "2")
                    )
                });
            format!("{} match [{}]", data_str(node, "variable", "x"), cases)
        }
        NODE_EXPR => data_str(node, "expression", "a + b"),
        NODE_FUNCTION => format!(
            "fn {}({})",
            data_str(node, "name", "f"),
            data_str(node, "params", "")
        ),
        NODE_CALL => format!("call {}", data_str(node, "name", "f")),
        NODE_CONST => format!("const {} = {}", data_str(node, "name", "x"), data_i64(node)),
        NODE_LIST => format!(
            "{} = [{}]",
            data_str(node, "name", "xs"),
            data_str(node, "items", "")
        ),
        NODE_THROW => data_str(node, "message", "…"),
        NODE_AWAIT => "await".to_string(),
        NODE_IMPORT => data_str(node, "module", "…"),
        NODE_STRUCT => format!(
            "struct {} {{ {} }}",
            data_str(node, "name", "T"),
            data_str(node, "fields", "")
        ),
        NODE_ENUM => format!(
            "enum {} {{ {} }}",
            data_str(node, "name", "E"),
            data_str(node, "variants", "")
        ),
        NODE_START => "Entry".to_string(),
        NODE_UI_PAGE | NODE_UI_BUTTON => data_str(node, "title", "…"),
        NODE_UI_LABEL => data_str(node, "text", "…"),
        NODE_UI_INPUT => data_str(node, "placeholder", "…"),
        NODE_UI_EVENT => data_str(node, "event", "…"),
        NODE_API_ROUTE => data_str(node, "path", "/…"),
        NODE_API_QUERY => data_str(node, "url", "…"),
        NODE_DB_READ => data_str(node, "table", "…"),
        NODE_SUBGRAPH => data_str(node, "module", "…"),
        NODE_EMIT_UI => data_str(node, "signal", "…"),
        _ => String::new(),
    }
}

fn data_str(node: &Node, key: &str, default: &str) -> String {
    node.data
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or(default)
        .to_string()
}

fn data_i64(node: &Node) -> i64 {
    graph_model::data_get_i64(&node.data, "value").unwrap_or(0)
}

pub(crate) fn switch_case_port_count(node: &Node) -> usize {
    if let Some(cases) = graph_model::data_get_str(&node.data, "cases") {
        let n = cases.split(',').filter(|s| !s.trim().is_empty()).count();
        if n >= 2 {
            return n.min(6);
        }
    }
    2
}
