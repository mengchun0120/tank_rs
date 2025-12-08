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
    pub phasing_duration: f32,
}

#[derive(Debug, Resource, Deserialize)]
pub struct GameObjConfig {
    pub name: String,
    pub image: String,
    pub size: [f32; 2],
    pub z: f32,
    pub obj_type: GameObjType,
    pub side: GameObjSide,
    pub speed: f32,
    pub collide_span: f32,
    pub shoot_config: Option<ShootConfig>,
    pub explosion_config: Option<ExplosionConfig>,
    pub damage_config: Option<DamageConfig>,
    pub max_hp: Option<u32>,
}

#[derive(Debug, Resource, Deserialize, PartialEq, Eq, Copy, Clone)]
pub enum GameObjType {
    Tile,
    Tank,
    Missile,
    Effect,
}

#[derive(Debug, Resource, Deserialize, PartialEq, Eq, Copy, Clone)]
pub enum GameObjSide {
    Player,
    AI,
    Neutral,
}

#[derive(Debug, Resource, Deserialize)]
pub struct ShootConfig {
    pub missile: String,
    pub shoot_duration: f32,
    pub shoot_position: [f32; 2],
}

#[derive(Debug, Resource, Deserialize)]
pub struct ExplosionConfig {
    pub image: String,
    pub size: [u32; 2],
    pub frame_count: u32,
    pub frames_per_second: usize,
    pub z: f32,
}

#[derive(Debug, Resource, Deserialize)]
pub struct DamageConfig {
    pub damage: u32,
    pub explode_span: f32,
}

#[derive(Debug, Resource)]
pub struct GameLib {
    pub config: GameConfig,
    pub origin: Vec2,
    pub images: HashMap<String, Handle<Image>>,
    pub game_obj_config_map: HashMap<String, usize>,
    pub texture_atlas_layout_map: HashMap<String, Handle<TextureAtlasLayout>>,
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
        texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    ) -> Result<Self, MyError> {
        let config: GameConfig = read_json(config_path)?;
        let images = Self::load_images(&config.image_files, asset_server);
        let (game_obj_config_map, texture_atlas_layout_map) =
            Self::load_configs(&config.game_obj_configs, texture_atlas_layouts);
        let origin = Vec2::new(-config.window_width() / 2.0, -config.window_height() / 2.0);

        let result = Self {
            config,
            origin,
            images,
            game_obj_config_map,
            texture_atlas_layout_map,
        };

        info!("GameLib initialized successfully");

        Ok(result)
    }

    #[inline]
    pub fn get_screen_pos(&self, pos: &Vec2) -> Vec2 {
        self.origin + pos
    }

    #[inline]
    pub fn get_obj_config(&self, config_index: usize) -> &GameObjConfig {
        &self.config.game_obj_configs[config_index]
    }

    pub fn get_obj_config_index(&self, name: &String) -> Option<usize> {
        self.game_obj_config_map.get(name).copied()
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

    fn load_configs(
        game_obj_configs: &[GameObjConfig],
        texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    ) -> (
        HashMap<String, usize>,
        HashMap<String, Handle<TextureAtlasLayout>>,
    ) {
        let mut game_obj_config_map: HashMap<String, usize> = HashMap::new();
        let mut texture_atlas_layout_map: HashMap<String, Handle<TextureAtlasLayout>> =
            HashMap::new();

        for i in 0..game_obj_configs.len() {
            game_obj_config_map.insert(game_obj_configs[i].name.clone(), i);

            if let Some(explosion_config) = &game_obj_configs[i].explosion_config {
                let tile_size = UVec2 {
                    x: explosion_config.size[0],
                    y: explosion_config.size[1],
                };
                let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                    tile_size,
                    explosion_config.frame_count,
                    1,
                    None,
                    None,
                ));
                texture_atlas_layout_map.insert(game_obj_configs[i].name.clone(), layout);
            }
        }

        (game_obj_config_map, texture_atlas_layout_map)
    }
}
