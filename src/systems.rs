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
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut commands: Commands,
    mut exit_app: MessageWriter<AppExit>,
    mut window: Single<&mut Window>,
) {
    let Some(mut game_lib) = load_game_lib(
        args.config_path.as_path(),
        asset_server.as_ref(),
        texture_atlas_layouts.as_mut(),
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
    mut commands: Commands,
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
    } else if keys.just_pressed(KeyCode::KeyF) || keys.pressed(KeyCode::KeyF) {
        shoot_player_missile(
            &mut player,
            &mut commands,
            game_lib.as_ref(),
            map.as_mut(),
            game_obj_lib.as_mut(),
            time.as_ref(),
        );
    }
}

fn load_game_lib<P: AsRef<Path>>(
    config_path: P,
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    exit_app: &mut MessageWriter<AppExit>,
) -> Option<GameLib> {
    let game_lib = match GameLib::new(config_path, asset_server, texture_atlas_layouts) {
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

pub fn update_missiles(
    mut missile_query: Query<(Entity, &mut Transform), With<MissileComponent>>,
    game_lib: Res<GameLib>,
    mut map: ResMut<GameMap>,
    mut game_obj_lib: ResMut<GameObjInfoLib>,
    mut despawn_pool: ResMut<DespawnPool>,
    time: Res<Time>,
) {
    for mut missile in missile_query.iter_mut() {
        if despawn_pool.contains(&missile.0) {
            continue;
        }

        let Some(obj) = game_obj_lib.get(&missile.0).cloned() else {
            error!("Failed to find entity in GameObjInfoLib");
            continue;
        };
        let obj_config = game_lib.get_obj_config(obj.config_index);

        let (collide, new_pos) = get_missile_new_pos(
            &missile.0,
            &obj,
            obj_config,
            map.as_ref(),
            game_lib.as_ref(),
            game_obj_lib.as_ref(),
            despawn_pool.as_ref(),
            time.as_ref(),
        );

        let screen_pos = game_lib.get_screen_pos(&new_pos);
        missile.1.translation.x = screen_pos.x;
        missile.1.translation.y = screen_pos.y;

        update_obj_pos_direction(
            &missile.0,
            &new_pos,
            &obj.direction,
            game_obj_lib.as_mut(),
            map.as_mut(),
        );

        if collide {
            despawn_pool.insert(missile.0);
        }
    }
}

pub fn cleanup_objs(mut commands: Commands, mut despawn_pool: ResMut<DespawnPool>) {
    for e in despawn_pool.iter() {
        commands.entity(e.clone()).despawn();
    }
    despawn_pool.clear();
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
        let pos = arr_to_vec2(&map_obj_config.pos);
        let direction: Vec2 = map_obj_config.direction.into();

        add_obj(
            config_index,
            &pos,
            &direction,
            game_lib,
            &mut map,
            game_obj_lib,
            commands,
        );
    }

    Ok(map)
}

fn add_obj(
    config_index: usize,
    pos: &Vec2,
    direction: &Vec2,
    game_lib: &GameLib,
    map: &mut GameMap,
    game_obj_lib: &mut GameObjInfoLib,
    commands: &mut Commands,
) {
    let obj_config = game_lib.get_obj_config(config_index);

    if !map.is_inside(&pos, obj_config.collide_span) {
        error!("Position {:?} is outside map", pos);
        return;
    }

    let map_pos = map.get_map_pos(&pos);

    if let Some((obj, entity)) =
        GameObjInfo::new(config_index, pos, &map_pos, direction, game_lib, commands)
    {
        map.add_obj(&map_pos, entity, obj_config.collide_span);
        game_obj_lib.insert(entity, obj);
    }
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
    if despawn_pool.contains(&player.0) {
        return;
    }

    let new_direction: Vec2 = d.into();
    let Some(obj) = game_obj_lib.get(&player.0).cloned() else {
        warn!("Cannot find player in map");
        return;
    };
    let mut new_pos = obj.pos;
    let obj_config = game_lib.get_obj_config(obj.config_index);

    if new_direction != obj.direction {
        player.1.rotation = get_rotation(&new_direction);
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

    if let Some(shoot_config) = obj_config.shoot_config.as_ref() {
        let init_pos = arr_to_vec2(&shoot_config.shoot_position);
        player.2.shoot_pos = obj.pos + new_direction.rotate(init_pos);
    }

    capture_collide_missiles(
        &new_pos,
        obj_config,
        map,
        game_lib,
        game_obj_lib,
        despawn_pool,
    );
}

fn shoot_player_missile(
    player: &mut Single<(Entity, &mut Transform, &mut ShootComponent), With<PlayerComponent>>,
    commands: &mut Commands,
    game_lib: &GameLib,
    map: &mut GameMap,
    game_obj_lib: &mut GameObjInfoLib,
    time: &Time,
) {
    player.2.timer.tick(time.delta());
    if player.2.timer.just_finished() {
        let Some(direction) = game_obj_lib.get(&player.0).map(|obj| obj.direction) else {
            error!("Failed to find player in GameObjInfoLib");
            return;
        };
        add_obj(
            player.2.missile_config_index,
            &player.2.shoot_pos,
            &direction,
            game_lib,
            map,
            game_obj_lib,
            commands,
        );

        player.2.timer.reset();
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

    let (collide_bounds, pos) = check_collide_bounds_nonpass(
        &pos,
        obj_config.collide_span,
        &obj.direction,
        map.width,
        map.height,
    );

    let (collide_objs, pos) = check_tank_collide(
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

fn get_missile_new_pos(
    entity: &Entity,
    obj: &GameObjInfo,
    obj_config: &GameObjConfig,
    map: &GameMap,
    game_lib: &GameLib,
    game_obj_lib: &GameObjInfoLib,
    despawn_pool: &DespawnPool,
    time: &Time,
) -> (bool, Vec2) {
    let pos = obj.pos + obj.direction * obj_config.speed * time.delta_secs();

    if check_collide_bounds_pass(&pos, obj_config.collide_span, map.width, map.height) {
        return (true, pos);
    }

    let collide = check_missile_collide(
        entity,
        &pos,
        obj_config,
        map,
        game_lib,
        game_obj_lib,
        despawn_pool,
    );

    (collide, pos)
}

fn update_obj_pos_direction(
    entity: &Entity,
    new_pos: &Vec2,
    new_direction: &Vec2,
    game_obj_lib: &mut GameObjInfoLib,
    map: &mut GameMap,
) {
    let Some(obj) = game_obj_lib.get_mut(entity) else {
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
        map.get_collide_region_pass(pos, obj_config.collide_span, map.max_collide_span);

    for row in start_map_pos.row..=end_map_pos.row {
        for col in start_map_pos.col..=end_map_pos.col {
            for e in map.map[row][col].iter() {
                let Some(obj2) = game_obj_lib.get(e) else {
                    warn!("Cannot find entity {e} in map");
                    continue;
                };
                let obj_config2 = game_lib.get_obj_config(obj2.config_index);

                if obj_config2.obj_type != GameObjType::Missile
                    || obj_config2.side == obj_config.side
                {
                    continue;
                }

                if check_collide_obj_pass(
                    pos,
                    obj_config.collide_span,
                    &obj2.pos,
                    obj_config2.collide_span,
                ) {
                    despawn_pool.insert(e.clone());
                }
            }
        }
    }
}

fn check_tank_collide(
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
        map.max_collide_span,
    );
    let mut pos = new_pos.clone();

    for row in start_map_pos.row..=end_map_pos.row {
        for col in start_map_pos.col..=end_map_pos.col {
            for e in map.map[row][col].iter() {
                if e == entity || despawn_pool.contains(e) {
                    continue;
                }

                let Some(obj2) = game_obj_lib.get(e) else {
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

                let (collide_obj, corrected_pos) = check_collide_obj_nonpass(
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

fn check_missile_collide(
    entity: &Entity,
    new_pos: &Vec2,
    obj_config: &GameObjConfig,
    map: &GameMap,
    game_lib: &GameLib,
    game_obj_lib: &GameObjInfoLib,
    despawn_pool: &DespawnPool,
) -> bool {
    let (start_map_pos, end_map_pos) =
        map.get_collide_region_pass(new_pos, obj_config.collide_span, map.max_collide_span);

    for row in start_map_pos.row..=end_map_pos.row {
        for col in start_map_pos.col..=end_map_pos.col {
            for e in map.map[row][col].iter() {
                if e == entity || despawn_pool.contains(e) {
                    continue;
                }

                let Some(obj2) = game_obj_lib.get(e) else {
                    warn!("Cannot find entity {e} in map");
                    continue;
                };
                let obj_config2 = game_lib.get_obj_config(obj2.config_index);

                if (obj_config2.obj_type != GameObjType::Tank
                    && obj_config2.obj_type != GameObjType::Tile)
                    || obj_config2.collide_span == 0.0
                    || obj_config.side == obj_config2.side
                {
                    continue;
                }

                if check_collide_obj_pass(
                    new_pos,
                    obj_config.collide_span,
                    &obj2.pos,
                    obj_config2.collide_span,
                ) {
                    return true;
                }
            }
        }
    }

    false
}

fn create_explosion(
    pos: Vec2,
    obj_config: &GameObjConfig,
    game_lib: &GameLib,
    commands: &mut Commands,
) {
    let Some(explosion_config) = obj_config.explosion_config.as_ref() else {
        error!("ExplosionConfig is absent in GameObjConfig");
        return;
    };
    let Some(texture) = game_lib.images.get(&explosion_config.image).cloned() else {
        error!("Failed to find image: {}", explosion_config.image);
        return;
    };
    let Some(layout) = game_lib.texture_atlas_layout_map.get(&obj_config.name).cloned() else {
        error!("Failed to find TextureAtlasLayout for {}", obj_config.name);
        return;
    };

    let screen_pos = game_lib.get_screen_pos(&pos);
    let frame_duration = 1.0 / explosion_config.frames_per_second as f32;

    commands.spawn((
        Sprite::from_atlas_image(texture, TextureAtlas { layout, index: 1 }),
        Transform::from_xyz(screen_pos.x, screen_pos.y, explosion_config.z),
        ExplosionComponent {
            timer: Timer::from_seconds(frame_duration, TimerMode::Repeating),
            last_index: explosion_config.frame_count as usize,
        },
    ));
}
