use crate::config::Config;
use crate::gui::geometry::SlotGeometry;
use crate::gui::ui::ThemeColors;
use crate::gui::{
    ANGLE_STEP, CENTER_CIRCLE_RADIUS, ICON_INACTIVE_ALPHA, ICON_SIZE, INNER_RADIUS, OUTER_RADIUS,
    REFERENCE_HEIGHT, SLOT_COUNT, START_OFFSET,
};
use crate::sys::desktop::AppInfo;
use crate::sys::wm::{Point, WindowClass};
use cairo::Context;
use gdk_pixbuf::Pixbuf;
use gdk4::prelude::*;
use std::f64::consts::PI;

pub struct Slot {
    pub app: Option<AppInfo>,
    pub pixbuf: Option<Pixbuf>,
}

impl Slot {
    pub fn new(app: Option<AppInfo>) -> Self {
        let pixbuf = app.as_ref().and_then(|a| {
            (!a.icon.as_os_str().is_empty())
                .then(|| Pixbuf::from_file_at_scale(&a.icon, ICON_SIZE, ICON_SIZE, true).ok())
                .flatten()
        });
        Self { app, pixbuf }
    }

    pub fn is_running(&self, active_classes: &[WindowClass]) -> bool {
        self.app.as_ref().is_some_and(|app| {
            active_classes
                .iter()
                .any(|c| c.to_lowercase() == app.class.to_lowercase())
        })
    }

    pub fn is_broken(&self) -> bool {
        self.app
            .as_ref()
            .map(|a| a.exec.as_str().is_empty())
            .unwrap_or(false)
    }
}

pub struct State {
    pub center: Point,
    pub slots: Vec<Slot>,
    pub hover_index: Option<usize>,
    pub active_classes: Vec<WindowClass>,
    pub scale_factor: f64,
}

impl State {
    pub fn new(
        slots: Vec<Slot>,
        center: Point,
        active_classes: Vec<WindowClass>,
        scale_factor: f64,
    ) -> Self {
        Self {
            center,
            slots,
            hover_index: None,
            active_classes,
            scale_factor,
        }
    }

    pub fn init_slots(config: &Config) -> Vec<Slot> {
        let mut slots = vec![None; SLOT_COUNT];
        config.slots.iter().for_each(|cfg| {
            if let (Some(dir), Some(query)) = (cfg.direction, &cfg.app) {
                let idx = dir.as_index();
                slots[idx] = Some(AppInfo::new(query, cfg.class.clone(), cfg.exec.clone()));
            }
        });
        slots.into_iter().map(Slot::new).collect()
    }

    pub fn update_cursor(&mut self, cursor: Point) -> CursorAction {
        let (cx, cy) = (self.center.x, self.center.y);
        let (dx, dy) = (cursor.x - cx, cursor.y - cy);
        let dist = dx.hypot(dy);

        if dist <= INNER_RADIUS * self.scale_factor {
            let changed = self.hover_index.is_some();
            self.hover_index = None;
            return CursorAction {
                should_redraw: changed,
                should_activate: false,
            };
        }

        let cursor_angle = dy.atan2(dx);
        let new_idx = (0..SLOT_COUNT)
            .filter(|&i| self.slots[i].app.is_some())
            .min_by(|&a, &b| {
                let diff = |i| {
                    let slot_angle = START_OFFSET + (i as f64 * ANGLE_STEP);
                    (cursor_angle - slot_angle + PI).rem_euclid(2.0 * PI) - PI
                };
                diff(a)
                    .abs()
                    .partial_cmp(&diff(b).abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        let changed = self.hover_index != new_idx;
        self.hover_index = new_idx;
        let activate = dist > OUTER_RADIUS * self.scale_factor && self.hover_index.is_some();
        CursorAction {
            should_redraw: changed || activate,
            should_activate: activate,
        }
    }

    pub fn get_hovered_app(&self) -> Option<&AppInfo> {
        self.hover_index
            .and_then(|idx| self.slots[idx].app.as_ref())
    }

    pub fn refresh(
        &mut self,
        center: Point,
        active_classes: Vec<WindowClass>,
        monitor_height: f64,
    ) {
        self.active_classes = active_classes;
        self.center = center;
        self.hover_index = None;
        self.scale_factor = monitor_height / REFERENCE_HEIGHT;
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CursorAction {
    pub should_redraw: bool,
    pub should_activate: bool,
}

pub fn draw(cr: &Context, state: &State, colors: &ThemeColors) -> Result<(), cairo::Error> {
    let (cx, cy) = (state.center.x, state.center.y);
    let (r, g, b, a) = colors.center_circle.into_components();
    cr.set_source_rgba(r, g, b, a);
    cr.arc(
        cx,
        cy,
        CENTER_CIRCLE_RADIUS * state.scale_factor,
        0.0,
        2.0 * PI,
    );
    cr.fill()?;

    let filled_indices: Vec<usize> = (0..SLOT_COUNT)
        .filter(|&i| state.slots[i].app.is_some())
        .collect();

    for &i in &filled_indices {
        let slot = &state.slots[i];
        let geometry =
            SlotGeometry::calculate(i, &filled_indices, state.center, state.scale_factor);

        let hovered = state.hover_index == Some(i);
        let running = slot.is_running(&state.active_classes);
        let broken = slot.is_broken();

        let color = if broken {
            colors.broken
        } else if hovered {
            colors.hovered
        } else if running {
            colors.running
        } else {
            colors.default
        };
        let (r, g, b, a) = color.into_components();

        cr.set_source_rgba(r, g, b, a);
        cr.arc(
            geometry.center.x,
            geometry.center.y,
            geometry.radius,
            0.0,
            2.0 * PI,
        );
        cr.fill()?;

        if let Some(pixbuf) = &slot.pixbuf {
            let icon_scale = (geometry.radius * 2.0 * 0.75) / ICON_SIZE as f64;
            let (iw, ih) = (
                pixbuf.width() as f64 * icon_scale,
                pixbuf.height() as f64 * icon_scale,
            );
            let (ix, iy) = (geometry.center.x - iw / 2.0, geometry.center.y - ih / 2.0);

            cr.save()?;
            cr.translate(ix, iy);
            cr.scale(icon_scale, icon_scale);

            if !running && !hovered {
                cr.push_group();
                cr.set_source_pixbuf(pixbuf, 0.0, 0.0);
                cr.paint()?;
                cr.pop_group_to_source()?;
                cr.paint_with_alpha(ICON_INACTIVE_ALPHA)?;
            } else {
                cr.set_source_pixbuf(pixbuf, 0.0, 0.0);
                cr.paint()?;
            }
            cr.restore()?;
        } else if let Some(app) = &slot.app {
            cr.set_source_rgb(1.0, 1.0, 1.0);
            cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
            cr.set_font_size(12.0 * geometry.scale);
            if let Ok(ext) = cr.text_extents(&app.name) {
                cr.move_to(
                    geometry.center.x - ext.width() / 2.0,
                    geometry.center.y + ext.height() / 2.0,
                );
                cr.show_text(&app.name)?;
            }
        }
    }
    Ok(())
}
