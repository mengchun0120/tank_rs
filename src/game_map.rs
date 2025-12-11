use crate::game_lib::*;
use crate::my_error::*;
use crate::game_obj::*;
use crate::utils::*;
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::Path;

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct MapPos {
    pub row: usize,
    pub col: usize,
}

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
    pub objs: Vec<GameMapObjConfig>,
}

#[derive(Resource)]
pub struct GameMap {
    pub cell_size: f32,
    pub width: f32,
    pub height: f32,
    pub map: Vec<Vec<HashSet<Entity>>>,
    pub max_collide_span: f32,
}

impl GameMap {
    pub fn new(cell_size: f32, row_count: usize, col_count: usize) -> Self {
        Self {
            cell_size,
            width: col_count as f32 * cell_size,
            height: row_count as f32 * cell_size,
            map: vec![vec![HashSet::new(); col_count]; row_count],
            max_collide_span: 0.0,
        }
    }

    pub fn load<P: AsRef<Path>>(
        map_path: P,
        game_lib: &GameLib,
        game_obj_lib: &mut GameObjInfoLib,
        commands: &mut Commands,
    ) -> Result<GameMap, MyError> {
        let map_config: GameMapConfig = read_json(map_path.as_ref())?;
        let game_config = &game_lib.config;
        let mut map = GameMap::new(
            game_config.map_cell_size,
            game_config.map_row_count(),
            game_config.map_col_count(),
        );

        for map_obj_config in map_config.objs.iter() {
            let Some(config_index) = game_lib.get_obj_config_index(&map_obj_config.config_name) else {
                warn!(
                    "Failed to find config name {} in GameLib",
                    map_obj_config.config_name
                );
                continue;
            };
            let pos = arr_to_vec2(&map_obj_config.pos);
            let direction: Vec2 = map_obj_config.direction.into();

            map.add_obj(
                config_index,
                &pos,
                &direction,
                game_lib,
                game_obj_lib,
                commands,
            );
        }

        Ok(map)
    }

    pub fn add_obj(
        &mut self,
        config_index: usize,
        pos: &Vec2,
        direction: &Vec2,
        game_lib: &GameLib,
        game_obj_lib: &mut GameObjInfoLib,
        commands: &mut Commands,
    ) {
        let obj_config = game_lib.get_obj_config(config_index);

        if !self.is_inside(&pos, obj_config.collide_span) {
            error!("Position {:?} is outside map", pos);
            return;
        }

        let map_pos = self.get_map_pos(&pos);

        if let Some((obj, entity)) =
            GameObjInfo::new(config_index, pos, &map_pos, direction, game_lib, commands)
        {
            if obj_config.side == GameObjSide::Player {
                commands.insert_resource(PlayerInfo(Some(entity)));
            }

            self.map[map_pos.row][map_pos.col].insert(entity);
            if self.max_collide_span < obj_config.collide_span {
                self.max_collide_span = obj_config.collide_span;
            }

            game_obj_lib.insert(entity, obj);
        }
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

    #[inline]
    pub fn remove_obj(&mut self, pos: &MapPos, entity: &Entity) {
        self.map[pos.row][pos.col].remove(entity);
    }

    #[inline]
    pub fn get_map_index(&self, s: f32) -> i32 {
        (s / self.cell_size).floor() as i32
    }

    #[inline]
    pub fn clamp_row(&self, y: f32) -> usize {
        self.get_map_index(y)
            .clamp(0, (self.row_count() - 1) as i32) as usize
    }

    #[inline]
    pub fn clamp_col(&self, x: f32) -> usize {
        self.get_map_index(x)
            .clamp(0, (self.col_count() - 1) as i32) as usize
    }

    #[inline]
    pub fn get_collide_region_nonpass(
        &self,
        start_pos: &Vec2,
        end_pos: &Vec2,
        collide_span: f32,
    ) -> (MapPos, MapPos) {
        let span = collide_span + self.max_collide_span;
        let left = start_pos.x.min(end_pos.x) - span;
        let bottom = start_pos.y.min(end_pos.y) - span;
        let right = start_pos.x.max(end_pos.x) + span;
        let top = start_pos.y.max(end_pos.y) + span;

        self.get_map_region(left, bottom, right, top)
    }

    #[inline]
    pub fn get_collide_region_pass(&self, pos: &Vec2, collide_span: f32) -> (MapPos, MapPos) {
        let span = collide_span + self.max_collide_span;
        let left = pos.x - span;
        let bottom = pos.y - span;
        let right = pos.x + span;
        let top = pos.y + span;

        self.get_map_region(left, bottom, right, top)
    }

    #[inline]
    fn get_map_region(&self, left: f32, bottom: f32, right: f32, top: f32) -> (MapPos, MapPos) {
        (
            MapPos {
                row: self.clamp_row(bottom),
                col: self.clamp_col(left),
            },
            MapPos {
                row: self.clamp_row(top),
                col: self.clamp_col(right),
            },
        )
    }

    #[inline]
    pub fn relocate(&mut self, entity: &Entity, old_pos: &MapPos, new_pos: &MapPos) {
        self.map[old_pos.row][old_pos.col].remove(entity);
        self.map[new_pos.row][new_pos.col].insert(entity.clone());
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
