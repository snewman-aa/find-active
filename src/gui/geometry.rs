use crate::gui::{ANGLE_STEP, MENU_RADIUS, SLOT_COUNT, SLOT_RADIUS, START_OFFSET};
use crate::sys::wm::Point;
use std::f64::consts::PI;

#[derive(Debug, Clone)]
pub struct SlotGeometry {
    pub center: Point,
    pub radius: f64,
    pub scale: f64,
}

impl SlotGeometry {
    pub fn calculate(
        index: usize,
        filled_indices: &[usize],
        center: Point,
        scale_factor: f64,
    ) -> Self {
        let prev_idx = filled_indices[(filled_indices.iter().position(|&x| x == index).unwrap()
            + filled_indices.len()
            - 1)
            % filled_indices.len()];
        let next_idx = filled_indices
            [(filled_indices.iter().position(|&x| x == index).unwrap() + 1) % filled_indices.len()];

        let d_l = if prev_idx == index {
            2.0 * PI
        } else {
            ((index + SLOT_COUNT - prev_idx) % SLOT_COUNT) as f64 * ANGLE_STEP
        };
        let d_r = if next_idx == index {
            2.0 * PI
        } else {
            ((next_idx + SLOT_COUNT - index) % SLOT_COUNT) as f64 * ANGLE_STEP
        };
        let width = (d_l + d_r) / 2.0;
        let scale = (width / ANGLE_STEP).sqrt().min(2.5);
        let current_slot_radius = SLOT_RADIUS * scale * scale_factor;

        let angle = START_OFFSET + (index as f64 * ANGLE_STEP);
        let (x, y) = (
            center.x + (MENU_RADIUS * scale_factor) * angle.cos(),
            center.y + (MENU_RADIUS * scale_factor) * angle.sin(),
        );

        Self {
            center: Point::new(x, y),
            radius: current_slot_radius,
            scale,
        }
    }
}
