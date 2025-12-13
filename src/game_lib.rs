use crate::ai::*;
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
    pub explosion_configs: HashMap<String, ExplosionConfig>,
    pub shoot_configs: HashMap<String, ShootConfig>,
    pub ai_configs: Vec<AIConfig>,
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
    pub shoot_config: Option<String>,
    pub explosion_name: Option<String>,
    pub max_hp: Option<f32>,
    pub ai_config: Option<String>,
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
    pub damage: f32,
    pub explode_span: f32,
    pub image: String,
    pub size: [u32; 2],
    pub frame_count: u32,
    pub frames_per_second: usize,
    pub z: f32,
}

#[derive(Debug, Resource)]
pub struct GameLib {
    config: GameConfig,
    origin: Vec2,
    images: HashMap<String, Handle<Image>>,
    game_obj_config_map: HashMap<String, usize>,
    texture_atlas_layout_map: HashMap<String, Handle<TextureAtlasLayout>>,
    ai_config_map: HashMap<String, usize>,
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
        let origin = Vec2::new(-config.window_width() / 2.0, -config.window_height() / 2.0);

        let mut game_lib = Self {
            config,
            origin,
            images: HashMap::new(),
            game_obj_config_map: HashMap::new(),
            texture_atlas_layout_map: HashMap::new(),
            ai_config_map: HashMap::new(),
        };

        game_lib.load_images(asset_server);
        game_lib.load_configs(texture_atlas_layouts);

        info!("GameLib initialized successfully");

        Ok(game_lib)
    }

    #[inline]
    pub fn get_game_config(&self) -> &GameConfig {
        &self.config
    }

    #[inline]
    pub fn get_screen_pos(&self, pos: &Vec2) -> Vec2 {
        self.origin + pos
    }

    #[inline]
    pub fn get_obj_config(&self, config_index: usize) -> &GameObjConfig {
        &self.config.game_obj_configs[config_index]
    }

    #[inline]
    pub fn get_obj_config_index(&self, name: &String) -> Option<usize> {
        self.game_obj_config_map.get(name).copied()
    }

    #[inline]
    pub fn get_image(&self, name: &String) -> Option<Handle<Image>> {
        match self.images.get(name) {
            Some(image) => Some(image.clone()),
            None => {
                error!("Failed to find image {}", name);
                None
            }
        }
    }

    #[inline]
    pub fn get_shoot_config(&self, name: &String) -> Option<&ShootConfig> {
        self.config.shoot_configs.get(name)
    }

    #[inline]
    pub fn get_explosion_config(&self, name: &String) -> Option<&ExplosionConfig> {
        self.config.explosion_configs.get(name)
    }

    #[inline]
    pub fn get_texture_atlas_layout(
        &self,
        explosion_name: &String,
    ) -> Option<Handle<TextureAtlasLayout>> {
        match self.texture_atlas_layout_map.get(explosion_name) {
            Some(layout) => Some(layout.clone()),
            None => {
                error!("Failed to find TextureAtlasLayout {}", explosion_name);
                None
            }
        }
    }

    fn load_images(&mut self, asset_server: &AssetServer) {
        for (name, file_path) in self.config.image_files.iter() {
            self.images
                .insert(name.clone(), asset_server.load(file_path));
        }
    }

    fn load_configs(&mut self, texture_atlas_layouts: &mut Assets<TextureAtlasLayout>) {
        for i in 0..self.config.game_obj_configs.len() {
            self.game_obj_config_map
                .insert(self.get_obj_config(i).name.clone(), i);

            if let Some(explosion_name) = self.get_obj_config(i).explosion_name.clone() {
                self.add_texture_layout(&explosion_name, texture_atlas_layouts);
            }
        }
    }

    fn add_texture_layout(
        &mut self,
        explosion_name: &String,
        texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    ) {
        let Some(explosion_config) = self.get_explosion_config(explosion_name) else {
            return;
        };

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
        self.texture_atlas_layout_map
            .insert(explosion_name.clone(), layout);
    }
}
