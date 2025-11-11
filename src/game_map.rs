use crate::game_lib::*;
use crate::game_obj::*;
use crate::utils::*;

use bevy::prelude::*;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

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
    pub game_objs: Vec<GameMapObjConfig>,
}

#[derive(Resource)]
pub struct GameMap {
    cell_size: f32,
    pub width: f32,
    pub height: f32,
    map: Vec<Vec<HashSet<Entity>>>,
    entities: HashMap<Entity, GameObj>,
    max_collide_span: f32,
}

impl GameMap {
    pub fn new(cell_size: f32, row_count: usize, col_count: usize) -> Self {
        Self {
            cell_size,
            width: col_count as f32 * cell_size,
            height: row_count as f32 * cell_size,
            map: vec![vec![HashSet::new(); col_count]; row_count],
            entities: HashMap::new(),
            max_collide_span: 0.0,
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

    #[inline]
    pub fn get_map_index(&self, s: f32) -> i32 {
        (s / self.cell_size).floor() as i32
    }

    #[inline]
    pub fn bound_row(&self, row: i32) -> usize {
        if row < 0 {
            0
        } else if row as usize >= self.row_count() {
            self.row_count() - 1
        } else {
            row as usize
        }
    }

    #[inline]
    pub fn bound_col(&self, col: i32) -> usize {
        if col < 0 {
            0
        } else if col as usize >= self.col_count() {
            self.col_count() - 1
        } else {
            col as usize
        }
    }

    #[inline]
    pub fn get_bounded_row(&self, y: f32) -> usize {
        self.bound_row(self.get_map_index(y))
    }

    #[inline]
    pub fn get_bounded_col(&self, x: f32) -> usize {
        self.bound_col(self.get_map_index(x))
    }

    #[inline]
    pub fn get_obj(&self, e: &Entity) -> Option<&GameObj> {
        self.entities.get(e)
    }

    pub fn move_obj(
        &mut self,
        entity: Entity,
        direction: Vec2,
        game_lib: &GameLib,
        time_delta: f32,
    ) -> Option<Vec2> {
        let Some((pos, config_index)) = self
            .entities
            .get(&entity)
            .map(|obj| (obj.pos, obj.config_index))
        else {
            warn!("Cannot find entity {entity} in map");
            return None;
        };

        let obj_config = game_lib.get_obj_config(config_index);
        if obj_config.speed == 0.0 {
            return None;
        }

        let velocity = obj_config.speed * direction;
        let (_, time_delta) = self.check_collide(
            &entity,
            &pos,
            &velocity,
            obj_config.collide_span,
            game_lib,
            time_delta,
        );

        let new_pos = pos + velocity * time_delta;
        self.update_pos_direction(&entity, new_pos, direction);

        Some(new_pos)
    }

    fn add_objs(
        &mut self,
        game_objs: &[GameMapObjConfig],
        game_lib: &GameLib,
        commands: &mut Commands,
    ) {
        for map_obj_config in game_objs {
            let Some(config_index) = game_lib.get_obj_config_index(&map_obj_config.config_name)
            else {
                continue;
            };
            let obj_config = game_lib.get_obj_config(config_index);
            let pos = arr_to_vec2(&map_obj_config.pos);

            if !self.is_inside(&pos, obj_config.collide_span) {
                error!("Position {:?} is outside map", pos);
                continue;
            }

            let map_pos = self.get_map_pos(&pos);

            if let Some((obj, entity)) = GameObj::new(
                config_index,
                pos,
                map_pos,
                map_obj_config.direction,
                game_lib,
                commands,
            ) {
                self.map[map_pos.row][map_pos.col].insert(entity);
                self.entities.insert(entity, obj);

                if self.max_collide_span < obj_config.collide_span {
                    self.max_collide_span = obj_config.collide_span;
                }
            }
        }
    }

    pub fn check_collide(
        &mut self,
        entity: &Entity,
        pos: &Vec2,
        velocity: &Vec2,
        collide_span: f32,
        game_lib: &GameLib,
        time_delta: f32,
    ) -> (bool, f32) {
        let (collide_bounds, time_delta) = check_obj_collide_bounds(
            &pos,
            &velocity,
            collide_span,
            time_delta,
            self.width,
            self.height,
        );

        let (start_map_pos, end_map_pos) =
            self.get_collide_region(pos, velocity, collide_span, time_delta);

        let (collide_obj, time_delta) = self.check_collide_nonpass(
            entity,
            pos,
            velocity,
            collide_span,
            &start_map_pos,
            &end_map_pos,
            time_delta,
            game_lib,
        );

        (collide_bounds || collide_obj, time_delta)
    }

    fn check_collide_nonpass(
        &mut self,
        entity: &Entity,
        pos: &Vec2,
        velocity: &Vec2,
        collide_span: f32,
        start_map_pos: &MapPos,
        end_map_pos: &MapPos,
        time_delta: f32,
        game_lib: &GameLib,
    ) -> (bool, f32) {
        let mut collide = false;
        let mut time_delta = time_delta;

        for row in start_map_pos.row..=end_map_pos.row {
            for col in start_map_pos.col..=end_map_pos.col {
                for e in self.map[row][col].iter() {
                    if e == entity {
                        continue;
                    }

                    let Some((pos2, config_index)) =
                        self.get_obj(e).map(|obj| (obj.pos, obj.config_index))
                    else {
                        warn!("Cannot find entity {e} in map");
                        continue;
                    };
                    let obj_config = game_lib.get_obj_config(config_index);

                    let (collide1, time_delta1) = check_obj_collide_nonpass(
                        pos,
                        velocity,
                        collide_span,
                        &pos2,
                        obj_config.collide_span,
                        time_delta,
                    );

                    if collide1 {
                        collide = true;
                        if time_delta1 < time_delta {
                            time_delta = time_delta1;
                        }
                    }
                }
            }
        }

        (collide, time_delta)
    }

    fn get_collide_region(
        &self,
        pos: &Vec2,
        velocity: &Vec2,
        collide_span: f32,
        time_delta: f32,
    ) -> (MapPos, MapPos) {
        let end_pos = pos + velocity * time_delta;
        let span = collide_span + self.max_collide_span;
        let start_x = pos.x.min(end_pos.x) - span;
        let start_y = pos.y.min(end_pos.y) - span;
        let start_map_pos = MapPos {
            row: self.get_bounded_row(start_y),
            col: self.get_bounded_col(start_x),
        };

        let end_x = pos.x.max(end_pos.x) + span;
        let end_y = pos.y.max(end_pos.y) + span;
        let end_map_pos = MapPos {
            row: self.get_bounded_row(end_y),
            col: self.get_bounded_col(end_x),
        };

        (start_map_pos, end_map_pos)
    }

    pub fn update_pos_direction(&mut self, e: &Entity, new_pos: Vec2, new_direction: Vec2) {
        let Some(old_map_pos) = self.entities.get(e).map(|obj| obj.map_pos) else {
            error!("Cannot find obj {e} in map");
            return;
        };

        let new_map_pos = self.get_map_pos(&new_pos);
        if new_map_pos != old_map_pos {
            self.map[old_map_pos.row][old_map_pos.col].remove(&e);
            self.map[new_map_pos.row][new_map_pos.col].insert(e.clone());
        }

        let obj = self.entities.get_mut(&e).unwrap();
        obj.pos = new_pos;
        obj.direction = new_direction;
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
