use crate::game_lib::*;
use crate::game_map::*;
use crate::game_obj::*;
use crate::my_error::*;
use crate::utils::*;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use std::path::Path;

pub fn setup_game(
    args: Res<Args>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut exit_app: MessageWriter<AppExit>,
    mut window: Single<&mut Window>,
) {
    let Some(mut game_lib) = load_game_lib(
        args.config_path.as_path(),
        asset_server.as_ref(),
        &mut exit_app,
    ) else {
        return;
    };

    init_window(&game_lib.config, window.as_mut());
    commands.spawn(Camera2d);

    let mut game_obj_lib = GameObjInfoLib(HashMap::new());
    let despawn_pool = DespawnPool(HashSet::new());

    let map = match load_map(
        args.map_path.as_path(),
        &mut game_lib,
        &mut commands,
        &mut game_obj_lib,
    ) {
        Ok(m) => m,
        Err(err) => {
            error!("Failed to load map from {:?}: {}", args.map_path, err);
            exit_app.write(AppExit::error());
            return;
        }
    };

    commands.insert_resource(game_lib);
    commands.insert_resource(game_obj_lib);
    commands.insert_resource(map);
    commands.insert_resource(despawn_pool);

    info!("Setup finished");
}

pub fn process_input(
    keys: Res<ButtonInput<KeyCode>>,
    game_lib: Res<GameLib>,
    mut player: Single<(Entity, &mut Transform, &mut ShootComponent), With<PlayerComponent>>,
    mut map: ResMut<GameMap>,
    mut game_obj_lib: ResMut<GameObjInfoLib>,
    mut despawn_pool: ResMut<DespawnPool>,
    time: Res<Time>,
) {
    if keys.just_pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::ArrowRight) {
        steer_player(
            Direction::Right,
            game_lib.as_ref(),
            &mut player,
            map.as_mut(),
            game_obj_lib.as_mut(),
            despawn_pool.as_mut(),
            time.as_ref(),
        );
    } else if keys.just_pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::ArrowLeft) {
        steer_player(
            Direction::Left,
            game_lib.as_ref(),
            &mut player,
            map.as_mut(),
            game_obj_lib.as_mut(),
            despawn_pool.as_mut(),
            time.as_ref(),
        );
    } else if keys.just_pressed(KeyCode::ArrowUp) || keys.pressed(KeyCode::ArrowUp) {
        steer_player(
            Direction::Up,
            game_lib.as_ref(),
            &mut player,
            map.as_mut(),
            game_obj_lib.as_mut(),
            despawn_pool.as_mut(),
            time.as_ref(),
        );
    } else if keys.just_pressed(KeyCode::ArrowDown) || keys.pressed(KeyCode::ArrowDown) {
        steer_player(
            Direction::Down,
            game_lib.as_ref(),
            &mut player,
            map.as_mut(),
            game_obj_lib.as_mut(),
            despawn_pool.as_mut(),
            time.as_ref(),
        );
    }
}

/*fn update_missile(
    mut missile_query: Query<(Entity, &mut Transform), With<MissileComponent>>,
    game_lib: Res<GameLib>,
    mut game_map: ResMut<GameMap>,
    time: Res<Time>,
) {
    let time_delta = time.delta_secs();
    for mut missile in missile_query.iter_mut() {
        let Some((collide, new_pos)) = game_map.move_obj(&missile.0, game_lib.as_ref(), time_delta)
        else {
            continue;
        };

        let new_screen_pos = game_lib.get_screen_pos(&new_pos);
        missile.1.translation.x = new_screen_pos.x;
        missile.1.translation.y = new_screen_pos.y;
    }
}*/

fn load_game_lib<P: AsRef<Path>>(
    config_path: P,
    asset_server: &AssetServer,
    exit_app: &mut MessageWriter<AppExit>,
) -> Option<GameLib> {
    let game_lib = match GameLib::new(config_path, asset_server) {
        Ok(lib) => lib,
        Err(err) => {
            error!("Failed to initialize GameLib: {}", err);
            exit_app.write(AppExit::error());
            return None;
        }
    };

    Some(game_lib)
}

fn init_window(config: &GameConfig, window: &mut Window) {
    window
        .resolution
        .set(config.window_width(), config.window_height());
}

fn load_map<P: AsRef<Path>>(
    map_path: P,
    game_lib: &mut GameLib,
    commands: &mut Commands,
    game_obj_lib: &mut GameObjInfoLib,
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
        let obj_config = game_lib.get_obj_config(config_index);
        let pos = arr_to_vec2(&map_obj_config.pos);

        if !map.is_inside(&pos, obj_config.collide_span) {
            error!("Position {:?} is outside map", pos);
            continue;
        }

        let map_pos = map.get_map_pos(&pos);

        if let Some((obj, entity)) = GameObjInfo::new(
            config_index,
            pos,
            map_pos,
            map_obj_config.direction,
            game_lib,
            commands,
        ) {
            map.add_obj(&map_pos, entity);
            if game_lib.max_collide_span < obj_config.collide_span {
                game_lib.max_collide_span = obj_config.collide_span;
            }

            game_obj_lib.0.insert(entity, obj);
        }
    }

    Ok(map)
}

fn steer_player(
    d: Direction,
    game_lib: &GameLib,
    player: &mut Single<(Entity, &mut Transform, &mut ShootComponent), With<PlayerComponent>>,
    map: &mut GameMap,
    game_obj_lib: &mut GameObjInfoLib,
    despawn_pool: &mut DespawnPool,
    time: &Time,
) {
    if despawn_pool.0.contains(&player.0) {
        return;
    }

    let new_direction: Vec2 = d.into();
    let Some(obj) = game_obj_lib.0.get(&player.0).cloned() else {
        warn!("Cannot find player in map");
        return;
    };
    let mut new_pos = obj.pos;
    let obj_config = game_lib.get_obj_config(obj.config_index);

    if new_direction != obj.direction {
        player.1.rotation = get_rotation(new_direction);
    } else {
        let (_, pos) = get_tank_new_pos(
            &player.0,
            &obj,
            obj_config,
            map,
            game_lib,
            game_obj_lib,
            despawn_pool,
            time,
        );
        new_pos = pos;

        let new_screen_pos = game_lib.get_screen_pos(&new_pos);
        player.1.translation.x = new_screen_pos.x;
        player.1.translation.y = new_screen_pos.y;
    }

    update_obj_pos_direction(&player.0, &new_pos, &new_direction, game_obj_lib, map);

    capture_collide_missiles(
        &new_pos,
        obj_config,
        map,
        game_lib,
        game_obj_lib,
        despawn_pool,
    );

    if let Some(shoot_config) = obj_config.shoot_config.as_ref() {
        let init_pos = arr_to_vec2(&shoot_config.shoot_position);
        player.2.shoot_pos = obj.pos + new_direction.rotate(init_pos);
    }
}

fn get_tank_new_pos(
    entity: &Entity,
    obj: &GameObjInfo,
    obj_config: &GameObjConfig,
    map: &GameMap,
    game_lib: &GameLib,
    game_obj_lib: &GameObjInfoLib,
    despawn_pool: &DespawnPool,
    time: &Time,
) -> (bool, Vec2) {
    let time_delta = time.delta_secs();
    let pos = obj.pos + obj.direction * obj_config.speed * time_delta;

    let (collide_bounds, pos) = collide_bounds_nonpass(
        &pos,
        obj_config.collide_span,
        &obj.direction,
        map.width,
        map.height,
    );

    let (collide_objs, pos) = check_tank_collide_nonpass(
        entity,
        &pos,
        obj,
        obj_config,
        map,
        game_lib,
        game_obj_lib,
        despawn_pool,
    );

    (collide_bounds || collide_objs, pos)
}

fn update_obj_pos_direction(
    entity: &Entity,
    new_pos: &Vec2,
    new_direction: &Vec2,
    game_obj_lib: &mut GameObjInfoLib,
    map: &mut GameMap,
) {
    let Some(obj) = game_obj_lib.0.get_mut(entity) else {
        error!("Failed to find entity {} in GameObjInfoLib", entity);
        return;
    };

    obj.pos = new_pos.clone();
    let new_map_pos = map.get_map_pos(new_pos);
    map.relocate(entity, &obj.map_pos, &new_map_pos);
    obj.direction = new_direction.clone();
    obj.map_pos = new_map_pos;
}

fn capture_collide_missiles(
    pos: &Vec2,
    obj_config: &GameObjConfig,
    map: &GameMap,
    game_lib: &GameLib,
    game_obj_lib: &GameObjInfoLib,
    despawn_pool: &mut DespawnPool,
) {
    let (start_map_pos, end_map_pos) =
        map.get_collide_region_pass(pos, obj_config.collide_span, game_lib.max_collide_span);

    for row in start_map_pos.row..=end_map_pos.row {
        for col in start_map_pos.col..=end_map_pos.col {
            for e in map.map[row][col].iter() {
                let Some(obj2) = game_obj_lib.0.get(e) else {
                    warn!("Cannot find entity {e} in map");
                    continue;
                };
                let obj_config2 = game_lib.get_obj_config(obj2.config_index);

                if obj_config2.obj_type != GameObjType::Missile
                    || obj_config2.side == obj_config.side
                {
                    continue;
                }

                if collide_obj_pass(
                    pos,
                    obj_config.collide_span,
                    &obj2.pos,
                    obj_config2.collide_span,
                ) {
                    despawn_pool.0.insert(e.clone());
                }
            }
        }
    }
}

fn check_tank_collide_nonpass(
    entity: &Entity,
    new_pos: &Vec2,
    obj: &GameObjInfo,
    obj_config: &GameObjConfig,
    map: &GameMap,
    game_lib: &GameLib,
    game_obj_lib: &GameObjInfoLib,
    despawn_pool: &DespawnPool,
) -> (bool, Vec2) {
    let mut collide = false;
    let (start_map_pos, end_map_pos) = map.get_collide_region_nonpass(
        &obj.pos,
        &new_pos,
        obj_config.collide_span,
        game_lib.max_collide_span,
    );
    let mut pos = new_pos.clone();

    for row in start_map_pos.row..=end_map_pos.row {
        for col in start_map_pos.col..=end_map_pos.col {
            for e in map.map[row][col].iter() {
                if e == entity || despawn_pool.0.contains(e) {
                    continue;
                }

                let Some(obj2) = game_obj_lib.0.get(e) else {
                    warn!("Cannot find entity {e} in map");
                    continue;
                };
                let obj_config2 = game_lib.get_obj_config(obj2.config_index);

                if (obj_config2.obj_type != GameObjType::Tank
                    && obj_config2.obj_type != GameObjType::Tile)
                    || obj_config2.collide_span == 0.0
                {
                    continue;
                }

                let (collide_obj, corrected_pos) = collide_obj_nonpass(
                    &pos,
                    obj_config.collide_span,
                    &obj.direction,
                    &obj2.pos,
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
