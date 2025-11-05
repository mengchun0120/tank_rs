use crate::game_lib::*;
use crate::utils::*;

use bevy::prelude::*;

#[derive(Clone, Resource)]
pub struct GameObj {
    pub config_index: usize,
    pub entity: Entity,
}

#[derive(Component)]
pub struct AIComponent;

#[derive(Component)]
pub struct PlayerComponent;

impl GameObj {
    pub fn new(
        config_index: usize,
        pos: &Vec2,
        game_lib: &GameLib,
        commands: &mut Commands,
    ) -> Option<Self> {
        let obj_config = &game_lib.config.game_obj_configs[config_index];
        let Some(entity) = Self::create_entity(pos, obj_config, game_lib, commands) else {
            return None;
        };
        let obj = Self {
            config_index,
            entity,
        };

        Some(obj)
    }

    pub fn create_entity(
        pos: &Vec2,
        obj_config: &GameObjConfig,
        game_lib: &GameLib,
        commands: &mut Commands,
    ) -> Option<Entity> {
        let Some(image) = game_lib.images.get(&obj_config.name) else {
            error!("Cannot find image {}", obj_config.image);
            return None;
        };
        let size = arr_to_vec2(&obj_config.size);

        let mut entity = commands
            .spawn((
                Sprite {
                    image: image.clone(),
                    custom_size: Some(size),
                    image_mode: SpriteImageMode::Scale(ScalingMode::FillCenter),
                    ..default()
                },
                Transform::from_xyz(pos.x, pos.y, obj_config.z),
                Visibility::Visible,
            ));

        if obj_config.obj_type == GameObjType::Tank {
            if obj_config.side == GameObjSide::AI {
                entity.insert(AIComponent);
            } else if obj_config.side == GameObjSide::Player {
                entity.insert(PlayerComponent);
            }
        }

        Some(entity.id())
    }
}
