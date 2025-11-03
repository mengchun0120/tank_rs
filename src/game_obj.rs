use crate::game_lib::*;

use bevy::prelude::*;

#[derive(Clone, Resource)]
pub struct GameObj {
    pub config_index: usize,
    pub entity: Entity,
}

impl GameObj {
    pub fn new(commands: &mut Commands) -> Self {
        todo!()
    }
}
