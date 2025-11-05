use crate::game_lib::*;
use crate::game_obj::*;
use crate::utils::*;

use bevy::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GameMapObjConfig {
    pub config_name: String,
    pub pos: [f32; 2],
    pub direction: Option<[f32; 2]>,
}

#[derive(Deserialize)]
pub struct GameMapConfig {
    pub game_objs: Vec<GameMapObjConfig>,
}

#[derive(Resource, Clone)]
pub struct GameMapCell(Vec<GameObj>);

impl GameMapCell {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, obj: GameObj) {
        self.0.push(obj);
    }

    pub fn remove(&mut self, entity: Entity) -> Option<GameObj> {
        let mut index: Option<usize> = None;

        for i in 0..self.0.len() {
            if self.0[i].entity == entity {
                index = Some(i);
                break;
            }
        }

        let mut result: Option<GameObj> = None;

        if let Some(i) = index {
            if i < self.0.len() - 1 {
                result = Some(self.0[i].clone());
                self.0[i] = self.0.pop().unwrap();
            } else {
                result = Some(self.0.pop().unwrap());
            }
        }

        result
    }
}

#[derive(Resource)]
pub struct GameMap {
    cell_size: f32,
    width: f32,
    height: f32,
    map: Vec<Vec<GameMapCell>>,
}

impl GameMap {
    pub fn new(cell_size: f32, row_count: usize, col_count: usize) -> Self {
        Self {
            cell_size,
            width: col_count as f32 * cell_size,
            height: row_count as f32 * cell_size,
            map: vec![vec![GameMapCell::new(); col_count]; row_count],
        }
    }

    pub fn load(map_config: &GameMapConfig, game_lib: &GameLib, commands: &mut Commands) -> Self {
        let config = &game_lib.config;
        let mut map = Self::new(
            config.map_cell_size,
            config.map_row_count(),
            config.map_col_count(),
        );

        map.add_objs(&map_config.game_objs, game_lib, commands);

        info!("Map loaded successfully");

        map
    }

    #[inline]
    pub fn row_count(&self) -> usize {
        self.map.len()
    }

    #[inline]
    pub fn col_count(&self) -> usize {
        self.map[0].len()
    }

    #[inline]
    pub fn is_inside(&self, pos: &Vec2) -> bool {
        pos.x >= 0.0 && pos.x < self.width && pos.y >= 0.0 && pos.y < self.height
    }

    #[inline]
    pub fn get_row(&self, y: f32) -> i32 {
        (y / self.cell_size).floor() as i32
    }

    #[inline]
    pub fn get_col(&self, x: f32) -> i32 {
        (x / self.cell_size).floor() as i32
    }

    fn add_objs(
        &mut self,
        game_objs: &[GameMapObjConfig],
        game_lib: &GameLib,
        commands: &mut Commands,
    ) {
        for obj_config in game_objs {
            if let Some(config_index) = game_lib.game_obj_config_map.get(&obj_config.config_name) {
                let pos = arr_to_vec2(&obj_config.pos);
                if !self.is_inside(&pos) {
                    error!("Position {:?} is outside map", pos);
                    continue;
                }

                if let Some(obj) = GameObj::new(
                    *config_index,
                    &game_lib.get_screen_pos(&pos),
                    game_lib,
                    commands,
                ) {
                    let row = self.get_row(pos.y) as usize;
                    let col = self.get_col(pos.x) as usize;
                    self.map[row][col].add(obj);
                }
            }
        }
    }
}
