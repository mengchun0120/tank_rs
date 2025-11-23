use crate::my_error::*;

use bevy::prelude::*;
use clap::Parser;
use core::f32;
use serde::de::DeserializeOwned;
use serde_json;
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[derive(Parser, Resource)]
pub struct Args {
    #[arg(short, long)]
    pub log_path: PathBuf,

    #[arg(short, long)]
    pub config_path: PathBuf,

    #[arg(short, long)]
    pub map_path: PathBuf,
}

pub fn read_json<T, P>(path: P) -> Result<T, MyError>
where
    T: DeserializeOwned,
    P: AsRef<Path>,
{
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let result: T = serde_json::from_reader(reader)?;
    Ok(result)
}

pub fn setup_log<P: AsRef<Path>>(log_path: P) -> WorkerGuard {
    let log_file = File::create(log_path.as_ref()).expect("Open file");
    let (non_blocking_appender, guard) = tracing_appender::non_blocking(log_file);

    let file_layer = fmt::layer()
        .with_ansi(false) // Disable ANSI color codes for the file to keep it clean
        .with_writer(non_blocking_appender)
        .with_file(true)
        .with_level(true)
        .with_line_number(true)
        .with_thread_names(true);

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(file_layer)
        .init();

    guard
}

#[inline]
pub fn arr_to_vec2(v: &[f32; 2]) -> Vec2 {
    Vec2 { x: v[0], y: v[1] }
}

#[inline]
pub fn get_rotation(d: Vec2) -> Quat {
    let from = Vec2::new(1.0, 0.0);
    Quat::from_rotation_arc_2d(from, d)
}

pub fn collide_bounds_nonpass(
    pos: &Vec2,
    collide_span: f32,
    direction: &Vec2,
    width: f32,
    height: f32,
) -> (bool, Vec2) {
    let left = pos.x - collide_span;
    let right = pos.x + collide_span;
    let dx = if left < 0.0 {
        -left
    } else if right > width {
        width - right
    } else {
        0.0
    };

    let bottom = pos.y - collide_span;
    let top = pos.y + collide_span;
    let dy = if bottom < 0.0 {
        -bottom
    } else if top > height {
        height - top
    } else {
        0.0
    };

    let mut corrected_pos = pos.clone();
    let min_x = collide_span;
    let max_x = width - collide_span;
    let min_y = collide_span;
    let max_y = height - collide_span;

    let collide = if dx == 0.0 && dy == 0.0 {
        false
    } else {
        if dx.signum() * direction.x.signum() < 0.0 && dy.signum() * direction.y.signum() < 0.0 {
            if (dx * direction.y).abs() < (dy * direction.x).abs() {
                corrected_pos.x = corrected_pos.x.clamp(min_x, max_x);
                corrected_pos.y += dy.signum() * (dx * direction.y / direction.x).abs();
                corrected_pos.y = corrected_pos.y.clamp(min_y, max_y);
            } else {
                corrected_pos.y = corrected_pos.y.clamp(min_y, max_y);
                corrected_pos.x += dx.signum() * (dy * direction.x / direction.y).abs();
                corrected_pos.x = corrected_pos.x.clamp(min_y, max_y);
            }
        } else {
            corrected_pos.x = corrected_pos.x.clamp(min_x, max_x);
            corrected_pos.y = corrected_pos.y.clamp(min_y, max_y);
        }
        true
    };

    (collide, corrected_pos)
}

pub fn collide_bounds_pass(pos: &Vec2, collide_span: f32, width: f32, height: f32) -> bool {
    pos.x - collide_span < 0.0
        || pos.x + collide_span > width
        || pos.y - collide_span < 0.0
        || pos.y + collide_span > height
}

pub fn collide_obj_nonpass(
    pos1: &Vec2,
    collide_span1: f32,
    direction: &Vec2,
    pos2: &Vec2,
    collide_span2: f32,
) -> (bool, Vec2) {
    let total_span = collide_span1 + collide_span2;
    let dx = (pos1.x - pos2.x).abs();
    let dy = (pos1.y - pos2.y).abs();
    let mut corrected_pos = pos1.clone();

    if dx >= total_span || dy >= total_span {
        return (false, corrected_pos);
    }

    let cx = total_span - dx;
    let cy = total_span - dy;

    if cx * direction.y.abs() < cy * direction.x.abs() {
        corrected_pos.x = if direction.x > 0.0 {
            pos2.x - total_span
        } else {
            pos2.x + total_span
        };
        if direction.y != 0.0 {
            corrected_pos.y -= direction.y.signum() * cx * direction.y.abs() / direction.x;
        }
    } else {
        corrected_pos.y = if direction.y > 0.0 {
            pos2.y - total_span
        } else {
            pos2.y + total_span
        };
        if direction.x != 0.0 {
            corrected_pos.x -= direction.x.signum() * cy * direction.x.abs() / direction.y;
        }
    }

    (true, corrected_pos)
}

pub fn collide_obj_pass(pos1: &Vec2, collide_span1: f32, pos2: &Vec2, collide_span2: f32) -> bool {
    let total_span = collide_span1 + collide_span2;
    (pos1.x - pos2.x).abs() < total_span || (pos1.y - pos2.y).abs() < total_span
}
