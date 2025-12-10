use crate::game_obj::*;

use bevy::prelude::*;
use rand::*;
use serde::Deserialize;

#[derive(Debug, Resource)]
pub enum Action {
    Move,
    Shoot,
}

#[derive(Debug, Deserialize, Resource)]
pub struct AIConfig {
    pub move_prob: f32,
    pub keep_direction_duration: f32,
    pub move_duration: f32,
    pub shoot_duration: f32,
}

#[derive(Component)]
pub struct AIComponent {
    pub action: Option<Action>,
    pub collision_happened: bool,
    pub keep_direction_timer: Timer,
    pub move_timer: Timer,
    pub shoot_timer: Timer,
}

impl AIComponent {
    pub fn new(ai_config: &AIConfig) -> Self {
        Self {
            action: None,
            collision_happened: false,
            keep_direction_timer: Timer::from_seconds(
                ai_config.keep_direction_duration,
                TimerMode::Repeating,
            ),
            move_timer: Timer::from_seconds(ai_config.move_duration, TimerMode::Repeating),
            shoot_timer: Timer::from_seconds(ai_config.shoot_duration, TimerMode::Repeating),
        }
    }
}

pub fn update_ai_for_obj(
    obj: &mut GameObjInfo,
    ai_comp: &mut AIComponent,
    ai_config: &AIConfig,
    player_pos: &Vec2,
    player_collide_span: f32,
    time: &Time,
) {
    ai_comp.keep_direction_timer.tick(time.delta());
    match ai_comp.action {
        None => new_action(obj, ai_comp, ai_config, player_pos, player_collide_span),
        Some(Action::Move) => {
            ai_comp.move_timer.tick(time.delta());
            if ai_comp.move_timer.is_finished() {
                new_action(obj, ai_comp, ai_config, player_pos, player_collide_span);
            } else if ai_comp.keep_direction_timer.is_finished() {
                choose_new_direction(obj, ai_comp, player_pos, player_collide_span);
            } else if ai_comp.collision_happened {
                choose_alt_direction(obj, ai_comp);
            }
        }
        Some(Action::Shoot) => {
            ai_comp.shoot_timer.tick(time.delta());
            if ai_comp.shoot_timer.is_finished() {
                new_action(obj, ai_comp, ai_config, player_pos, player_collide_span);
            } else if ai_comp.keep_direction_timer.is_finished() {
                choose_new_direction(obj, ai_comp, player_pos, player_collide_span);
            }
        }
    }
}

fn new_action(
    obj: &mut GameObjInfo,
    ai_comp: &mut AIComponent,
    ai_config: &AIConfig,
    player_pos: &Vec2,
    player_collide_span: f32,
) {
    let mut rng = rand::rng();

    let dice = rng.random_range(0.0..1.0) as f32;
    if dice < ai_config.move_prob {
        ai_comp.action = Some(Action::Move);
        ai_comp.move_timer.reset();
        ai_comp.collision_happened = false;
    } else {
        ai_comp.action = Some(Action::Shoot);
        ai_comp.shoot_timer.reset();
    }

    choose_new_direction(obj, ai_comp, player_pos, player_collide_span);
}

fn choose_alt_direction(obj: &mut GameObjInfo, ai_comp: &mut AIComponent) {
    let c = if rand::rng().random::<bool>() {
        1.0
    } else {
        -1.0
    };

    obj.direction = if obj.direction.x != 0.0 {
        Vec2::new(0.0, c)
    } else {
        Vec2::new(c, 0.0)
    };

    ai_comp.move_timer.reset();
    ai_comp.keep_direction_timer.reset();
    ai_comp.collision_happened = false;
}

fn choose_new_direction(
    obj: &mut GameObjInfo,
    ai_comp: &mut AIComponent,
    player_pos: &Vec2,
    player_collide_span: f32,
) {
    obj.direction = if (obj.pos.x - player_pos.x).abs() < player_collide_span {
        Vec2::new(0.0, (player_pos.y - obj.pos.y).signum())
    } else if (obj.pos.y - player_pos.y).abs() < player_collide_span {
        Vec2::new((player_pos.x - obj.pos.x).signum(), 0.0)
    } else if rand::rng().random::<bool>() {
        Vec2::new(0.0, (player_pos.y - obj.pos.y).signum())
    } else {
        Vec2::new((player_pos.x - obj.pos.x).signum(), 0.0)
    };

    ai_comp.keep_direction_timer.reset();
}
