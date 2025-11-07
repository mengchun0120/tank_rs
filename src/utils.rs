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

pub fn check_collide_bounds(
    pos: Vec2,
    velocity: Vec2,
    collide_span: f32,
    time_delta: f32,
    width: f32,
    height: f32,
) -> (bool, f32) {
    let time_delta_x = if velocity.x > 0.0 {
        (width - pos.x - collide_span) / velocity.x
    } else if velocity.x < 0.0 {
        (pos.x - collide_span) / (-velocity.x)
    } else {
        f32::INFINITY
    };

    let time_delta_y = if velocity.y > 0.0 {
        (height - pos.y - collide_span) / velocity.y
    } else if velocity.y < 0.0 {
        (pos.y - collide_span) / (-velocity.y)
    } else {
        f32::INFINITY
    };

    let time_delta_min = time_delta_x.min(time_delta_y);
    if time_delta_min < time_delta {
        (true, time_delta_min)
    } else {
        (false, time_delta)
    }
}
