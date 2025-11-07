use crate::my_error::*;
use crate::utils::*;

use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Resource, Deserialize)]
pub struct GameConfig {
    map_size: [usize; 2],
    pub map_cell_size: f32,
    image_files: HashMap<String, String>,
    pub game_obj_configs: Vec<GameObjConfig>,
}

#[derive(Debug, Resource, Deserialize)]
pub struct GameObjConfig {
    pub name: String,
    pub image: String,
    pub size: [f32; 2],
    pub z: f32,
    pub obj_type: GameObjType,
    pub side: GameObjSide,
    speed: Option<f32>,
    collide_span: Option<f32>,
}

#[derive(Debug, Resource, Deserialize, PartialEq, Eq)]
pub enum GameObjType {
    Tile,
    Tank,
    Missile,
    Effect,
}

#[derive(Debug, Resource, Deserialize, PartialEq, Eq)]
pub enum GameObjSide {
    Player,
    AI,
    Neutral,
}

#[derive(Debug, Resource)]
pub struct GameLib {
    pub config: GameConfig,
    pub origin: Vec2,
    pub images: HashMap<String, Handle<Image>>,
    pub game_obj_config_map: HashMap<String, usize>,
}

impl GameConfig {
    pub fn map_row_count(&self) -> usize {
        self.map_size[0]
    }

    pub fn map_col_count(&self) -> usize {
        self.map_size[1]
    }

    pub fn window_width(&self) -> f32 {
        self.map_col_count() as f32 * self.map_cell_size
    }

    pub fn window_height(&self) -> f32 {
        self.map_row_count() as f32 * self.map_cell_size
    }
}

impl GameLib {
    pub fn new<P: AsRef<Path>>(
        config_path: P,
        asset_server: &AssetServer,
    ) -> Result<Self, MyError> {
        let config: GameConfig = read_json(config_path)?;
        let images = Self::load_images(&config.image_files, asset_server);
        let game_obj_config_map = Self::load_game_obj_config_map(&config.game_obj_configs);
        let origin = Vec2::new(-config.window_width() / 2.0, -config.window_height() / 2.0);

        let result = Self {
            config,
            origin,
            images,
            game_obj_config_map,
        };

        info!("GameLib initialized successfully");

        Ok(result)
    }

    #[inline]
    pub fn get_screen_pos(&self, pos: Vec2) -> Vec2 {
        self.origin + pos
    }

    #[inline]
    pub fn get_obj_config(&self, config_index: usize) -> &GameObjConfig {
        &self.config.game_obj_configs[config_index]
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

    fn load_game_obj_config_map(game_obj_configs: &[GameObjConfig]) -> HashMap<String, usize> {
        let mut result: HashMap<String, usize> = HashMap::new();

        for i in 0..game_obj_configs.len() {
            result.insert(game_obj_configs[i].name.clone(), i);
        }

        result
    }
}

impl GameObjConfig {
    #[inline]
    pub fn speed(&self) -> f32 {
        self.speed.map_or(0.0, |s| s)
    }

    #[inline]
    pub fn collide_span(&self) -> f32 {
        self.collide_span.map_or(0.0, |c| c)
    }
}
