use egui::{Color32, Pos2, Stroke, Vec2};
use graph_model::{Edge, Project};

use super::editor::GraphEditor;
use super::nodes::scaled_font;
use super::types::HandleKind;

pub(crate) fn hit_handle(
    editor: &GraphEditor,
    node: &graph_model::Node,
    pos: Pos2,
    from_id: &str,
    from_kind: HandleKind,
) -> Option<(String, HandleKind)> {
    if node.id == from_id {
        return None;
    }
    let z = editor.zoom;
    let nr = editor.node_rect(node);
    for (kind, rect) in super::nodes::handle_hit_zones(node, nr, z) {
        if rect.contains(pos) && handles_can_connect(from_kind, kind) {
            return Some((node.id.clone(), kind));
        }
    }
    None
}

pub(crate) fn is_output_port(kind: HandleKind) -> bool {
    matches!(
        kind,
        HandleKind::ExecOut
            | HandleKind::TrueOut
            | HandleKind::FalseOut
            | HandleKind::DoneOut
            | HandleKind::BodyOut
            | HandleKind::TryOut
            | HandleKind::Case1Out
            | HandleKind::Case2Out
            | HandleKind::Case3Out
            | HandleKind::Case4Out
            | HandleKind::Case5Out
            | HandleKind::Case6Out
            | HandleKind::DefaultOut
            | HandleKind::CatchOut
    )
}

pub(crate) fn is_input_port(kind: HandleKind) -> bool {
    matches!(kind, HandleKind::ExecIn)
}

fn handles_can_connect(a: HandleKind, b: HandleKind) -> bool {
    (is_output_port(a) && is_input_port(b)) || (is_input_port(a) && is_output_port(b))
}

fn normalize_edge_endpoints(
    a_id: &str,
    a_kind: HandleKind,
    b_id: &str,
    b_kind: HandleKind,
) -> Option<(String, HandleKind, String, HandleKind)> {
    if is_output_port(a_kind) && is_input_port(b_kind) {
        Some((a_id.to_string(), a_kind, b_id.to_string(), b_kind))
    } else if is_input_port(a_kind) && is_output_port(b_kind) {
        Some((b_id.to_string(), b_kind, a_id.to_string(), a_kind))
    } else {
        None
    }
}

pub(crate) fn connect_handles(
    project: &mut Project,
    a_id: &str,
    a_kind: HandleKind,
    b_id: &str,
    b_kind: HandleKind,
) -> bool {
    let Some((src, sk, tgt, tk)) = normalize_edge_endpoints(a_id, a_kind, b_id, b_kind) else {
        return false;
    };
    push_edge(project, &src, &tgt, sk, tk);
    true
}

pub(crate) fn parse_source_handle(h: &str) -> HandleKind {
    match h.to_ascii_lowercase().as_str() {
        "true" => HandleKind::TrueOut,
        "false" => HandleKind::FalseOut,
        "done" => HandleKind::DoneOut,
        "body" => HandleKind::BodyOut,
        "case1" => HandleKind::Case1Out,
        "case2" => HandleKind::Case2Out,
        "case3" => HandleKind::Case3Out,
        "case4" => HandleKind::Case4Out,
        "case5" => HandleKind::Case5Out,
        "case6" => HandleKind::Case6Out,
        h if h.starts_with("case") => {
            if let Ok(n) = h[4..].parse::<usize>() {
                case_handle_from_index(n.saturating_sub(1))
            } else {
                HandleKind::Case1Out
            }
        }
        "default" => HandleKind::DefaultOut,
        "try" => HandleKind::TryOut,
        "catch" => HandleKind::CatchOut,
        _ => HandleKind::ExecOut,
    }
}

pub(crate) fn handle_label(k: HandleKind) -> &'static str {
    match k {
        HandleKind::TrueOut => "true",
        HandleKind::FalseOut => "false",
        HandleKind::DoneOut => "done",
        HandleKind::BodyOut => "body",
        HandleKind::TryOut => "try",
        HandleKind::Case1Out => "case1",
        HandleKind::Case2Out => "case2",
        HandleKind::Case3Out => "case3",
        HandleKind::Case4Out => "case4",
        HandleKind::Case5Out => "case5",
        HandleKind::Case6Out => "case6",
        HandleKind::DefaultOut => "default",
        HandleKind::CatchOut => "catch",
        _ => "exec",
    }
}

fn handle_target_label(_k: HandleKind) -> &'static str {
    "exec"
}

pub(crate) fn case_handle_from_index(i: usize) -> HandleKind {
    match i {
        0 => HandleKind::Case1Out,
        1 => HandleKind::Case2Out,
        2 => HandleKind::Case3Out,
        3 => HandleKind::Case4Out,
        4 => HandleKind::Case5Out,
        _ => HandleKind::Case6Out,
    }
}

pub(crate) fn edge_color(handle: &str, palette: &crate::theme::Palette) -> Color32 {
    match handle.to_ascii_lowercase().as_str() {
        "true" => palette.success,
        "false" => palette.danger,
        "done" => palette.assign,
        "body" => palette.success,
        "case1" => palette.accent,
        "case2" => palette.warn,
        "default" => palette.muted,
        "try" => palette.success,
        "catch" => palette.danger,
        _ => palette.accent,
    }
}

pub(crate) fn edge_label(handle: &str) -> Option<&'static str> {
    match handle.to_ascii_lowercase().as_str() {
        "true" => Some("true"),
        "false" => Some("false"),
        "done" => Some("done"),
        "body" => Some("body"),
        "case1" => Some("case1"),
        "case2" => Some("case2"),
        "case3" => Some("case3"),
        "case4" => Some("case4"),
        "case5" => Some("case5"),
        "case6" => Some("case6"),
        "default" => Some("default"),
        "try" => Some("try"),
        "catch" => Some("catch"),
        _ => None,
    }
}

pub(crate) fn handle_color(kind: HandleKind, palette: &crate::theme::Palette) -> Color32 {
    match kind {
        HandleKind::TrueOut => palette.success,
        HandleKind::FalseOut => palette.danger,
        HandleKind::DoneOut => palette.assign,
        HandleKind::BodyOut => palette.success,
        HandleKind::TryOut => palette.success,
        HandleKind::Case1Out
        | HandleKind::Case2Out
        | HandleKind::Case3Out
        | HandleKind::Case4Out
        | HandleKind::Case5Out
        | HandleKind::Case6Out => palette.accent,
        HandleKind::DefaultOut => palette.muted,
        HandleKind::CatchOut => palette.danger,
        HandleKind::ExecIn => palette.accent,
        HandleKind::ExecOut => palette.accent,
    }
}

pub(crate) fn draw_wire(
    painter: &egui::Painter,
    from: Pos2,
    to: Pos2,
    color: Color32,
    label: Option<&str>,
    dashed: bool,
    zoom: f32,
) {
    let z = zoom.max(0.35);
    let (c1, c2) = wire_control_points(from, to);
    let points: Vec<Pos2> = (0..=24)
        .map(|i| {
            let t = i as f32 / 24.0;
            bezier(from, c1, c2, to, t)
        })
        .collect();

    painter.add(egui::Shape::line(
        points.clone(),
        Stroke::new((6.0 * z).max(1.5), color.gamma_multiply(0.2)),
    ));

    if dashed {
        for w in points.windows(2).step_by(2) {
            if w.len() == 2 {
                painter.line_segment([w[0], w[1]], Stroke::new((2.5 * z).max(1.0), color));
            }
        }
    } else {
        painter.add(egui::Shape::line(
            points.clone(),
            Stroke::new((3.0 * z).max(1.0), color),
        ));
    }

    draw_arrow_head(
        painter,
        points[points.len().saturating_sub(2)],
        to,
        color,
        z,
    );

    if let Some(lbl) = label {
        let mid = points.get(points.len() / 2).copied().unwrap_or(from);
        painter.text(
            mid + Vec2::new(0.0, -10.0 * z),
            egui::Align2::CENTER_BOTTOM,
            lbl,
            scaled_font(11.0, z),
            color,
        );
    }
}

fn wire_control_points(from: Pos2, to: Pos2) -> (Pos2, Pos2) {
    let dx = (to.x - from.x).abs().max(48.0) * 0.55;
    let dy = (to.y - from.y).abs().max(48.0) * 0.55;
    if (to.x - from.x).abs() < 32.0 {
        let sy = if to.y >= from.y { 1.0 } else { -1.0 };
        return (
            Pos2::new(from.x, from.y + dy * sy),
            Pos2::new(to.x, to.y - dy * sy),
        );
    }
    let sx = if to.x >= from.x { 1.0 } else { -1.0 };
    (
        Pos2::new(from.x + dx * sx, from.y),
        Pos2::new(to.x - dx * sx, to.y),
    )
}

fn draw_arrow_head(painter: &egui::Painter, prev: Pos2, tip: Pos2, color: Color32, z: f32) {
    let dir = (tip - prev).normalized();
    if dir.length_sq() < 0.01 {
        return;
    }
    let side = Vec2::new(-dir.y, dir.x) * (7.0 * z).max(2.0);
    let back = tip - dir * (12.0 * z).max(4.0);
    painter.add(egui::Shape::convex_polygon(
        vec![tip, back + side, back - side],
        color,
        Stroke::NONE,
    ));
}

fn bezier(p0: Pos2, p1: Pos2, p2: Pos2, p3: Pos2, t: f32) -> Pos2 {
    let u = 1.0 - t;
    Pos2::new(
        u.powi(3) * p0.x
            + 3.0 * u.powi(2) * t * p1.x
            + 3.0 * u * t.powi(2) * p2.x
            + t.powi(3) * p3.x,
        u.powi(3) * p0.y
            + 3.0 * u.powi(2) * t * p1.y
            + 3.0 * u * t.powi(2) * p2.y
            + t.powi(3) * p3.y,
    )
}

pub(crate) fn disconnect_edges_at_handle(
    project: &mut Project,
    node_id: &str,
    kind: HandleKind,
) -> bool {
    let before = project.edges.len();
    match kind {
        HandleKind::ExecIn => {
            project.edges.retain(|e| e.target != node_id);
        }
        _ => {
            let label = handle_label(kind);
            project
                .edges
                .retain(|e| !(e.source == node_id && e.source_handle.eq_ignore_ascii_case(label)));
        }
    }
    before != project.edges.len()
}

fn push_edge(
    project: &mut Project,
    source: &str,
    target: &str,
    from_kind: HandleKind,
    to_kind: HandleKind,
) {
    let edge = Edge {
        id: format!("e_{}", uuid_simple()),
        source: source.to_string(),
        target: target.to_string(),
        source_handle: handle_label(from_kind).to_string(),
        target_handle: handle_target_label(to_kind).to_string(),
    };
    if !project.edges.iter().any(|e| {
        e.source == edge.source && e.target == edge.target && e.source_handle == edge.source_handle
    }) {
        project.edges.push(edge);
    }
}

pub(crate) fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{t:x}")
}
