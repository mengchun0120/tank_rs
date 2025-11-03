use crate::game_obj::*;

use bevy::prelude::*;

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

pub struct GameMap(Vec<Vec<GameMapCell>>);

impl GameMap {
    pub fn new(row_count: usize, col_count: usize) -> Self {
        Self(vec![vec![GameMapCell::new(); col_count]; row_count])
    }

    pub fn row_count(&self) -> usize {
        self.0.len()
    }

    pub fn col_count(&self) -> usize {
        self.0[0].len()
    }
}
