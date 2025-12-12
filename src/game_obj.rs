use crate::ai::*;
use crate::game_lib::*;
use crate::game_map::*;
use crate::utils::*;

use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Resource, Copy)]
pub struct GameObjInfo {
    pub config_index: usize,
    pub pos: Vec2,
    pub map_pos: MapPos,
    pub direction: Vec2,
    pub side: GameObjSide,
    pub obj_type: GameObjType,
    pub collide_span: f32,
    pub speed: f32,
    pub hp: Option<u32>,
}

#[derive(Component)]
pub struct TankComponent;

#[derive(Component)]
pub struct MissileComponent;

#[derive(Component)]
pub struct PlayerComponent;

#[derive(Component)]
pub struct ShootComponent {
    pub timer: Timer,
    pub shoot_pos: Vec2,
    pub missile_config_index: usize,
}

#[derive(Component)]
pub struct ExplosionComponent {
    pub timer: Timer,
    pub last_index: usize,
}

pub struct DeadGameObjInfo {
    pub map_pos: MapPos,
    pub is_phasing: bool,
}

#[derive(Component, Deref, DerefMut)]
pub struct PhasingTimer(pub Timer);

#[derive(Resource, Deref, DerefMut)]
pub struct GameObjInfoLib(pub HashMap<Entity, GameObjInfo>);

#[derive(Resource, Deref, DerefMut)]
pub struct DespawnPool(pub HashSet<Entity>);

#[derive(Resource)]
pub struct PlayerInfo(pub Option<Entity>);

impl GameObjInfo {
    pub fn new(
        config_index: usize,
        pos: &Vec2,
        map_pos: &MapPos,
        direction: &Vec2,
        game_lib: &GameLib,
        commands: &mut Commands,
    ) -> Option<(Self, Entity)> {
        let obj_config = &game_lib.config.game_obj_configs[config_index];
        let Some(entity) = Self::create_entity(pos, direction, obj_config, game_lib, commands)
        else {
            return None;
        };
        let obj = Self {
            config_index,
            pos: pos.clone(),
            map_pos: map_pos.clone(),
            direction: direction.clone(),
            side: obj_config.side,
            obj_type: obj_config.obj_type,
            collide_span: obj_config.collide_span,
            speed: obj_config.speed,
            hp: obj_config.max_hp.clone(),
        };

        Some((obj, entity))
    }

    fn create_entity(
        pos: &Vec2,
        direction: &Vec2,
        obj_config: &GameObjConfig,
        game_lib: &GameLib,
        commands: &mut Commands,
    ) -> Option<Entity> {
        let Some(image) = game_lib.get_image(&obj_config.image) else {
            return None;
        };
        let size = arr_to_vec2(&obj_config.size);
        let screen_pos = game_lib.get_screen_pos(&pos);

        let mut entity = commands.spawn((
            Sprite {
                image,
                custom_size: Some(size),
                image_mode: SpriteImageMode::Scale(ScalingMode::FillCenter),
                ..default()
            },
            Transform {
                translation: Vec3::new(screen_pos.x, screen_pos.y, obj_config.z),
                rotation: get_rotation(direction),
                ..default()
            },
        ));

        match obj_config.obj_type {
            GameObjType::Tank => {
                entity.insert(TankComponent);
            }
            GameObjType::Missile => {
                entity.insert(MissileComponent);
            }
            _ => (),
        }

        if obj_config.side == GameObjSide::AI {
            // if let Some(name) = obj_config.ai_config.as_ref() {
            //     if let Some(ai_config) = game_lib.config.ai_configs.get(name) {
            //         entity.insert(AIComponent::new(ai_config));
            //     } else {
            //         error!("Failed to find AIConfig {}", name);
            //     }
            // }
        } else if obj_config.side == GameObjSide::Player {
            entity.insert(PlayerComponent);
        }

        if let Some(shoot_config) = obj_config.shoot_config.as_ref() {
            let missile_config_index = game_lib
                .get_obj_config_index(&shoot_config.missile)
                .expect(&format!("Cannot find missile {}", shoot_config.missile));

            let shoot_pos = pos + direction.rotate(arr_to_vec2(&shoot_config.shoot_position));
            entity.insert(ShootComponent {
                timer: Timer::from_seconds(shoot_config.shoot_duration, TimerMode::Repeating),
                shoot_pos,
                missile_config_index,
            });
        }

        Some(entity.id())
    }
}

impl PhasingTimer {
    pub fn new(duration: f32) -> Self {
        Self(Timer::from_seconds(duration, TimerMode::Once))
    }
}
