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
    width: f32,
    height: f32,
    map: Vec<Vec<HashSet<Entity>>>,
    entities: HashMap<Entity, GameObj>,
    max_collide_span: f32,
    despawn_pool: HashSet<Entity>,
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
            despawn_pool: HashSet::new(),
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
    pub fn get_obj(&self, entity: &Entity) -> Option<&GameObj> {
        self.entities.get(entity)
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

    pub fn clear_despawn_pool(&mut self) {
        self.despawn_pool.clear();
    }

    pub fn move_obj(
        &mut self,
        entity: &Entity,
        game_lib: &GameLib,
        time_delta: f32,
    ) -> Option<(bool, Vec2)> {
        let Some(obj) = self.get_obj(entity).cloned() else {
            warn!("Cannot find entity {entity} in map");
            return None;
        };

        let obj_config = game_lib.get_obj_config(obj.config_index);
        if obj_config.speed == 0.0 {
            return None;
        }

        let (collide, new_pos) = match obj_config.obj_type {
            GameObjType::Tank => {
                self.check_nonpass_collide(entity, &obj, obj_config, time_delta, game_lib)
            }
            GameObjType::Missile => {
                self.check_missile_collide(entity, &obj, obj_config, time_delta, game_lib)
            }
            _ => {
                return None;
            }
        };

        self.update_pos(&entity, new_pos);

        Some((collide, new_pos))
    }

    pub fn change_direction(&mut self, entity: Entity, new_direction: Vec2) {
        let Some(obj) = self.entities.get_mut(&entity) else {
            return;
        };
        obj.direction = new_direction.clone();
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

    fn check_nonpass_collide(
        &mut self,
        entity: &Entity,
        obj: &GameObj,
        obj_config: &GameObjConfig,
        time_delta: f32,
        game_lib: &GameLib,
    ) -> (bool, Vec2) {
        let pos = obj.pos + obj.direction * obj_config.speed * time_delta;

        let (collide_bounds, pos) = collide_bounds_nonpass(
            &pos,
            obj_config.collide_span,
            &obj.direction,
            self.width,
            self.height,
        );

        let (collide_obj, pos) = self.check_nonpass_collide_objs(
            entity,
            &pos,
            &obj.direction,
            obj_config,
            time_delta,
            game_lib,
        );

        (collide_bounds || collide_obj, pos)
    }

    fn check_missile_collide(
        &mut self,
        entity: &Entity,
        obj: &GameObj,
        obj_config: &GameObjConfig,
        time_delta: f32,
        game_lib: &GameLib,
    ) -> (bool, Vec2) {
        let pos = obj.pos + obj.direction * obj_config.speed * time_delta;

        let collide = collide_bounds_pass(&pos, obj_config.collide_span, self.width, self.height);
        if collide {
            return (true, pos);
        }

        let collide = self.check_missile_collide_objs(entity, &pos, obj_config, game_lib);

        if collide {
            self.despawn_pool.insert(*entity);
        }

        (collide, pos)
    }

    fn check_nonpass_collide_objs(
        &self,
        entity: &Entity,
        pos: &Vec2,
        direction: &Vec2,
        obj_config: &GameObjConfig,
        time_delta: f32,
        game_lib: &GameLib,
    ) -> (bool, Vec2) {
        let mut collide = false;
        let (start_map_pos, end_map_pos) = self.get_collide_region_nonpass(
            pos,
            direction,
            obj_config.speed,
            obj_config.collide_span,
            time_delta,
        );
        let mut pos = pos.clone();

        for row in start_map_pos.row..=end_map_pos.row {
            for col in start_map_pos.col..=end_map_pos.col {
                for e in self.map[row][col].iter() {
                    if e == entity {
                        continue;
                    }

                    let Some((pos2, config_index)) =
                        self.entities.get(e).map(|obj| (obj.pos, obj.config_index))
                    else {
                        warn!("Cannot find entity {e} in map");
                        continue;
                    };
                    let obj_config2 = game_lib.get_obj_config(config_index);

                    if (obj_config2.obj_type != GameObjType::Tank
                        && obj_config2.obj_type != GameObjType::Tile)
                        || obj_config2.collide_span == 0.0
                        || self.despawn_pool.contains(e)
                    {
                        continue;
                    }

                    let (collide_obj, corrected_pos) = collide_obj_nonpass(
                        &pos,
                        obj_config.collide_span,
                        direction,
                        &pos2,
                        obj_config2.collide_span,
                    );

                    if collide_obj {
                        collide = true;
                    }

                    pos = corrected_pos;
                }
            }
        }

        (collide, pos)
    }

    fn check_missile_collide_objs(
        &self,
        entity: &Entity,
        pos: &Vec2,
        obj_config: &GameObjConfig,
        game_lib: &GameLib,
    ) -> bool {
        let (start_map_pos, end_map_pos) =
            self.get_collide_region_pass(pos, obj_config.collide_span);

        for row in start_map_pos.row..=end_map_pos.row {
            for col in start_map_pos.col..=end_map_pos.col {
                for e in self.map[row][col].iter() {
                    if e == entity {
                        continue;
                    }

                    let Some((pos2, config_index)) =
                        self.entities.get(e).map(|obj| (obj.pos, obj.config_index))
                    else {
                        warn!("Cannot find entity {e} in map");
                        continue;
                    };
                    let obj_config2 = game_lib.get_obj_config(config_index);

                    if (obj_config2.obj_type != GameObjType::Tank
                        && obj_config2.obj_type != GameObjType::Tile)
                        || obj_config2.collide_span == 0.0
                        || obj_config.side == obj_config2.side
                    {
                        continue;
                    }

                    if collide_obj_pass(
                        pos,
                        obj_config.collide_span,
                        &pos2,
                        obj_config2.collide_span,
                    ) {
                        return true;
                    }
                }
            }
        }

        false
    }

    #[inline]
    fn get_collide_region_nonpass(
        &self,
        pos: &Vec2,
        direction: &Vec2,
        speed: f32,
        collide_span: f32,
        time_delta: f32,
    ) -> (MapPos, MapPos) {
        let end_pos = pos + direction * speed * time_delta;
        let span = collide_span + self.max_collide_span;
        let left = pos.x.min(end_pos.x) - span;
        let bottom = pos.y.min(end_pos.y) - span;
        let right = pos.x.max(end_pos.x) + span;
        let top = pos.y.max(end_pos.y) + span;

        self.get_map_region(left, bottom, right, top)
    }

    #[inline]
    fn get_collide_region_pass(&self, pos: &Vec2, collide_span: f32) -> (MapPos, MapPos) {
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

    fn update_pos(&mut self, e: &Entity, new_pos: Vec2) {
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
