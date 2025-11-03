use crate::my_error::*;
use crate::utils::*;

use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Resource, Deserialize)]
pub struct GameConfig {
    map_size: [u32; 2],
    pub map_cell_size: f32,
    image_files: HashMap<String, String>,
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
    pub images: HashMap<String, Handle<Image>>,
}

impl GameLib {
    pub fn new<P: AsRef<Path>>(
        config_path: P,
        asset_server: &AssetServer,
    ) -> Result<Self, MyError> {
        let config: GameConfig = read_json(config_path)?;
        let images = Self::load_images(&config.image_files, asset_server);

        let result = Self { config, images };

        info!("GameLib initialized successfully");

        Ok(result)
    }

    fn load_images(
        image_files: &HashMap<String, String>,
        asset_server: &AssetServer,
    ) -> HashMap<String, Handle<Image>> {
        let mut result: HashMap<String, Handle<Image>> = HashMap::new();

        for (name, file_path) in image_files.iter() {
            result.insert(name.clone(), asset_server.load(file_path));
        }

        result
    }
}
