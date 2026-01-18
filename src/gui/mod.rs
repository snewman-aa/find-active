use std::f64::consts::PI;

pub mod app;
pub mod geometry;
pub mod menu;
pub mod ui;

pub const SLOT_COUNT: usize = 8;
pub const REFERENCE_HEIGHT: f64 = 1440.0;
pub const ICON_SIZE: i32 = 256;
pub const INNER_RADIUS: f64 = 50.0;
pub const OUTER_RADIUS: f64 = 160.0;
pub const MENU_RADIUS: f64 = 150.0;
pub const SLOT_RADIUS: f64 = 55.0;
pub const CENTER_CIRCLE_RADIUS: f64 = 40.0;
pub const ANGLE_STEP: f64 = 2.0 * PI / SLOT_COUNT as f64;
pub const START_OFFSET: f64 = -PI / 2.0;
pub const ICON_INACTIVE_ALPHA: f64 = 0.6;
