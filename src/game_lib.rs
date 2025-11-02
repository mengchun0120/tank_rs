use crate::my_error::*;
use crate::utils::*;
use bevy::prelude::*;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Resource, Deserialize)]
pub struct GameConfig {
    map_size: [u32; 2],
    pub map_cell_size: f32,
}

impl GameConfig {
    pub fn window_width(&self) -> f32 {
        self.map_size[0] as f32 * self.map_cell_size
    }

    pub fn window_height(&self) -> f32 {
        self.map_size[1] as f32 * self.map_cell_size
    }
}

#[derive(Debug, Resource)]
pub struct GameLib {
    pub config: GameConfig,
}

impl GameLib {
    pub fn new<P: AsRef<Path>>(
        config_path: P,
    ) -> Result<Self, MyError> {
        let config: GameConfig = read_json(config_path)?;
        let result = Self {
            config,
        };

        info!("GameLib initialized successfully");

        Ok(result)
    }
}