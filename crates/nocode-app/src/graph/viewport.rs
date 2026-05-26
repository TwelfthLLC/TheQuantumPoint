use egui::{Pos2, Rect, Ui, Vec2};

use super::editor::GraphEditor;

struct AutoPanAxisParams {
    pos: f32,
    min: f32,
    max: f32,
    edge: f32,
    speed: f32,
    dt: f32,
    boost: f32,
}

impl GraphEditor {
    fn should_auto_pan(&self) -> bool {
        self.drag_node.is_some() || self.connecting.is_some()
    }

    fn auto_pan_axis(pan: &mut f32, p: AutoPanAxisParams) {
        let AutoPanAxisParams {
            pos,
            min,
            max,
            edge,
            speed,
            dt,
            boost,
        } = p;
        const OUTSIDE_RAMP: f32 = 140.0;
        const OUTSIDE_MAX: f32 = 2.75;

        if pos < min {
            let overshoot = (min - pos).min(280.0);
            let extra = 1.0 + (overshoot / OUTSIDE_RAMP).min(OUTSIDE_MAX);
            *pan += speed * dt * boost * extra;
        } else if pos < min + edge {
            let t = 1.0 - (pos - min) / edge;
            *pan += speed * dt * boost * t * t;
        }

        if pos > max {
            let overshoot = (pos - max).min(280.0);
            let extra = 1.0 + (overshoot / OUTSIDE_RAMP).min(OUTSIDE_MAX);
            *pan -= speed * dt * boost * extra;
        } else if pos > max - edge {
            let t = (pos - (max - edge)) / edge;
            *pan -= speed * dt * boost * t * t;
        }
    }

    fn apply_canvas_auto_pan(&mut self, canvas: Rect, points: &[Pos2], dt: f32, boost: f32) {
        if points.is_empty() {
            return;
        }

        const EDGE: f32 = 80.0;
        const SPEED: f32 = 560.0;

        let mut pan = Vec2::ZERO;
        for pos in points {
            let mut dx = 0.0;
            let mut dy = 0.0;
            Self::auto_pan_axis(
                &mut dx,
                AutoPanAxisParams {
                    pos: pos.x,
                    min: canvas.left(),
                    max: canvas.right(),
                    edge: EDGE,
                    speed: SPEED,
                    dt,
                    boost,
                },
            );
            Self::auto_pan_axis(
                &mut dy,
                AutoPanAxisParams {
                    pos: pos.y,
                    min: canvas.top(),
                    max: canvas.bottom(),
                    edge: EDGE,
                    speed: SPEED,
                    dt,
                    boost,
                },
            );
            if dx.abs() > pan.x.abs() {
                pan.x = dx;
            }
            if dy.abs() > pan.y.abs() {
                pan.y = dy;
            }
        }
        self.pan += pan;
    }

    fn auto_pan_points(canvas: Rect, pointer: Option<Pos2>, hover: Option<Pos2>) -> Vec<Pos2> {
        let Some(p) = pointer.or(hover) else {
            return Vec::new();
        };
        let near_edge = p.x < canvas.left() + 120.0
            || p.x > canvas.right() - 120.0
            || p.y < canvas.top() + 120.0
            || p.y > canvas.bottom() - 120.0
            || !canvas.contains(p);
        if near_edge {
            vec![p]
        } else {
            Vec::new()
        }
    }

    pub(crate) fn update_canvas_auto_pan(
        &mut self,
        ui: &Ui,
        canvas: Rect,
        pointer: Option<Pos2>,
        hover: Option<Pos2>,
    ) {
        if !self.should_auto_pan() {
            return;
        }
        let boost = if self.drag_node.is_some() {
            1.15
        } else if self.connecting.is_some() {
            1.25
        } else {
            1.0
        };
        let points = Self::auto_pan_points(canvas, pointer, hover);
        if points.is_empty() {
            return;
        }
        let dt = ui.input(|i| i.unstable_dt).clamp(0.001, 0.05);
        self.apply_canvas_auto_pan(canvas, &points, dt, boost);
    }
}
