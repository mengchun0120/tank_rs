use crate::game_lib::*;
use crate::game_obj::*;
use crate::utils::*;

use bevy::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GameMapObjConfig {
    pub config_name: String,
    pub pos: [f32; 2],
    pub direction: Direction,
}

#[derive(Deserialize, Clone, Copy)]
pub enum Direction {
    Right,
    Left,
    Up,
    Down,
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

    pub fn push(&mut self, obj: GameObj) {
        self.0.push(obj);
    }

    pub fn pop(&mut self, entity: Entity) -> Option<GameObj> {
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
    pub width: f32,
    pub height: f32,
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
    pub fn is_inside(&self, pos: &Vec2, collide_span: f32) -> bool {
        pos.x >= collide_span
            && pos.x + collide_span < self.width
            && pos.y >= collide_span
            && pos.y + collide_span < self.height
    }

    #[inline]
    pub fn get_map_pos(&self, pos: &Vec2) -> MapPos {
        MapPos {
            row: (pos.y / self.cell_size).floor() as usize,
            col: (pos.x / self.cell_size).floor() as usize,
        }
    }

    fn add_objs(
        &mut self,
        game_objs: &[GameMapObjConfig],
        game_lib: &GameLib,
        commands: &mut Commands,
    ) {
        for map_obj_config in game_objs {
            if let Some(config_index) = game_lib
                .game_obj_config_map
                .get(&map_obj_config.config_name)
            {
                let obj_config = game_lib.get_obj_config(*config_index);
                let pos = arr_to_vec2(&map_obj_config.pos);

                if !self.is_inside(&pos, obj_config.collide_span()) {
                    error!("Position {:?} is outside map", pos);
                    continue;
                }

                let map_pos = self.get_map_pos(&pos);

                if let Some(obj) = GameObj::new(
                    *config_index,
                    pos,
                    map_pos,
                    map_obj_config.direction,
                    game_lib,
                    commands,
                ) {
                    self.map[map_pos.row][map_pos.col].push(obj);
                }
            }
        }
    }
}

impl From<Direction> for Vec2 {
    fn from(direction: Direction) -> Self {
        match direction {
            Direction::Left => Vec2::new(-1.0, 0.0),
            Direction::Right => Vec2::new(1.0, 0.0),
            Direction::Up => Vec2::new(0.0, 1.0),
            Direction::Down => Vec2::new(0.0, -1.0),
        }
    }
}
