use crate::ai::*;
use crate::game_lib::*;
use crate::game_map::*;
use crate::game_obj::*;
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

    let map = match GameMap::load(
        args.map_path.as_path(),
        &game_lib,
        &mut game_obj_lib,
        &mut commands,
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
            &mut commands,
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
            &mut commands,
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
            &mut commands,
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
            &mut commands,
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

pub fn update_missiles(
    mut missile_query: Query<(Entity, &mut Transform), With<MissileComponent>>,
    game_lib: Res<GameLib>,
    mut map: ResMut<GameMap>,
    mut game_obj_lib: ResMut<GameObjInfoLib>,
    mut despawn_pool: ResMut<DespawnPool>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let mut dead_objs: HashMap<Entity, DeadGameObjInfo> = HashMap::new();

    for (entity, mut transform) in missile_query.iter_mut() {
        if despawn_pool.contains(&entity) {
            continue;
        }

        let Some(obj) = game_obj_lib.get(&entity).cloned() else {
            error!("Failed to find entity in GameObjInfoLib");
            continue;
        };

        let (collide, new_pos) = map.get_missile_new_pos(
            &entity,
            &obj,
            game_obj_lib.as_ref(),
            despawn_pool.as_ref(),
            time.as_ref(),
        );

        let screen_pos = game_lib.get_screen_pos(&new_pos);
        transform.translation.x = screen_pos.x;
        transform.translation.y = screen_pos.y;

        update_obj_pos_direction(
            &entity,
            &new_pos,
            &obj.direction,
            game_obj_lib.as_mut(),
            map.as_mut(),
        );

        if collide {
            if let Some(explosion_name) = game_lib
                .get_obj_config(obj.config_index)
                .explosion_name
                .as_ref()
            {
                explode(
                    &new_pos,
                    obj.side,
                    explosion_name,
                    &mut dead_objs,
                    map.as_ref(),
                    game_lib.as_ref(),
                    game_obj_lib.as_mut(),
                    despawn_pool.as_ref(),
                    &mut commands,
                );
            }

            dead_objs.insert(
                entity.clone(),
                DeadGameObjInfo {
                    map_pos: obj.map_pos,
                    is_phasing: false,
                },
            );
        }
    }

    process_dead_objs(
        &dead_objs,
        game_lib.config.phasing_duration,
        map.as_mut(),
        game_obj_lib.as_mut(),
        despawn_pool.as_mut(),
        &mut commands,
    );
}

pub fn update_explosions(
    mut explosion_query: Query<(Entity, &mut Sprite, &mut ExplosionComponent)>,
    mut despawn_pool: ResMut<DespawnPool>,
    time: Res<Time>,
) {
    for (entity, mut sprite, mut explosion_comp) in explosion_query.iter_mut() {
        explosion_comp.timer.tick(time.delta());
        if explosion_comp.timer.is_finished() {
            if let Some(atlas) = sprite.texture_atlas.as_mut() {
                if atlas.index < explosion_comp.last_index - 1 {
                    atlas.index += 1;
                } else {
                    despawn_pool.insert(entity);
                }
            }
        }
    }
}

pub fn update_phasing_objs(
    mut phasing_obj_query: Query<(Entity, &mut Sprite, &mut PhasingTimer)>,
    mut despawn_pool: ResMut<DespawnPool>,
    game_lib: Res<GameLib>,
    time: Res<Time>,
) {
    for (entity, mut sprite, mut phasing_timer) in phasing_obj_query.iter_mut() {
        phasing_timer.tick(time.delta());
        if !phasing_timer.is_finished() {
            let alpha =
                (1.0 - phasing_timer.elapsed_secs() / game_lib.config.phasing_duration).max(0.0);
            sprite.color.set_alpha(alpha);
        } else {
            despawn_pool.insert(entity);
        }
    }
}

// pub fn update_ai(
//     mut ai_tank_query: Query<(Entity, &mut AIComponent)>,
//     game_lib: Res<GameLib>,
//     mut game_obj_lib: ResMut<GameObjInfoLib>,
//     player_info: Res<PlayerInfo>,
//     time: Res<Time>,
// ) {
//     let Some((player_pos, player_collide_span)) = get_player_info_for_ai(
//         player_info.as_ref(),
//         game_obj_lib.as_ref(),
//         game_lib.as_ref(),
//     ) else {
//         return;
//     };

//     for (entity, mut ai_comp) in ai_tank_query.iter_mut() {
//         let Some((obj, ai_config)) =
//             get_obj_for_ai(&entity, game_obj_lib.as_mut(), game_lib.as_ref())
//         else {
//             continue;
//         };

//         update_ai_for_obj(
//             obj,
//             ai_comp.as_mut(),
//             ai_config,
//             &player_pos,
//             player_collide_span,
//             time.as_ref(),
//         );
//     }
// }

pub fn cleanup(mut commands: Commands, mut despawn_pool: ResMut<DespawnPool>) {
    for e in despawn_pool.iter() {
        commands.entity(e.clone()).despawn();
    }
    despawn_pool.clear();
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

fn steer_player(
    d: Direction,
    game_lib: &GameLib,
    player: &mut Single<(Entity, &mut Transform, &mut ShootComponent), With<PlayerComponent>>,
    map: &mut GameMap,
    game_obj_lib: &mut GameObjInfoLib,
    despawn_pool: &mut DespawnPool,
    commands: &mut Commands,
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
        let (_, pos) = map.get_tank_new_pos(&player.0, &obj, game_obj_lib, despawn_pool, time);
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
        commands,
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
        map.add_obj(
            player.2.missile_config_index,
            &player.2.shoot_pos,
            &direction,
            game_lib,
            game_obj_lib,
            commands,
        );

        player.2.timer.reset();
    }
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
    map: &mut GameMap,
    game_lib: &GameLib,
    game_obj_lib: &mut GameObjInfoLib,
    despawn_pool: &mut DespawnPool,
    commands: &mut Commands,
) {
    let (start_map_pos, end_map_pos) = map.get_collide_region_pass(pos, obj_config.collide_span);
    let mut dead_objs: HashMap<Entity, DeadGameObjInfo> = HashMap::new();

    for row in start_map_pos.row..=end_map_pos.row {
        for col in start_map_pos.col..=end_map_pos.col {
            for e in map.map[row][col].iter() {
                if despawn_pool.contains(e) {
                    continue;
                }
                let Some(obj2) = game_obj_lib.get(e).cloned() else {
                    warn!("Cannot find entity {e} in map");
                    continue;
                };
                let obj_config2 = game_lib.get_obj_config(obj2.config_index);

                if obj_config2.obj_type == GameObjType::Missile
                    && obj_config2.side != obj_config.side
                {
                    if check_collide_obj_pass(
                        pos,
                        obj_config.collide_span,
                        &obj2.pos,
                        obj_config2.collide_span,
                    ) {
                        if let Some(explosion_name) = obj_config2.explosion_name.as_ref() {
                            explode(
                                &obj2.pos,
                                obj2.side,
                                explosion_name,
                                &mut dead_objs,
                                map,
                                game_lib,
                                game_obj_lib,
                                despawn_pool,
                                commands,
                            );
                        }

                        dead_objs.insert(
                            e.clone(),
                            DeadGameObjInfo {
                                map_pos: obj2.map_pos,
                                is_phasing: false,
                            },
                        );
                    }
                }
            }
        }
    }

    process_dead_objs(
        &dead_objs,
        game_lib.config.phasing_duration,
        map,
        game_obj_lib,
        despawn_pool,
        commands,
    );
}

fn explode(
    pos: &Vec2,
    side: GameObjSide,
    explosion_name: &String,
    dead_objs: &mut HashMap<Entity, DeadGameObjInfo>,
    map: &GameMap,
    game_lib: &GameLib,
    game_obj_lib: &mut GameObjInfoLib,
    despawn_pool: &DespawnPool,
    commands: &mut Commands,
) {
    let Some(explosion_config) = game_lib.get_explosion_config(explosion_name) else {
        error!("Failed to find ExplosionConfig {}", explosion_name);
        return;
    };

    do_damage(
        pos,
        side,
        explosion_config.damage,
        explosion_config.explode_span,
        dead_objs,
        map,
        game_obj_lib,
        despawn_pool,
        game_lib,
    );

    create_explosion(pos, explosion_name, explosion_config, game_lib, commands);
}

fn do_damage(
    pos: &Vec2,
    side: GameObjSide,
    damage: f32,
    explode_span: f32,
    dead_objs: &mut HashMap<Entity, DeadGameObjInfo>,
    map: &GameMap,
    game_obj_lib: &mut GameObjInfoLib,
    despawn_pool: &DespawnPool,
    game_lib: &GameLib,
) {
    let (start_pos, end_pos) = map.get_collide_region_pass(pos, explode_span);

    for row in start_pos.row..=end_pos.row {
        for col in start_pos.col..=end_pos.col {
            for e in map.map[row][col].iter() {
                if dead_objs.contains_key(e) || despawn_pool.contains(e) {
                    continue;
                }
                let Some(obj) = game_obj_lib.get_mut(e) else {
                    error!("Failed to find entity {} in GameObjLib", e);
                    continue;
                };

                if obj.obj_type == GameObjType::Tank
                    && obj.side != side
                    && check_collide_obj_pass(pos, explode_span, &obj.pos, obj.collide_span)
                {
                    if let Some(hp) = obj.hp.as_mut() {
                        *hp = (*hp - damage).max(0.0);
                        if *hp == 0.0 {
                            dead_objs.insert(
                                e.clone(),
                                DeadGameObjInfo {
                                    map_pos: obj.map_pos,
                                    is_phasing: true,
                                },
                            );
                        }
                    }
                }
            }
        }
    }
}

fn create_explosion(
    pos: &Vec2,
    explosion_name: &String,
    explosion_config: &ExplosionConfig,
    game_lib: &GameLib,
    commands: &mut Commands,
) {
    let Some(texture) = game_lib.get_image(&explosion_config.image) else {
        return;
    };
    let Some(layout) = game_lib.get_texture_atlas_layout(explosion_name) else {
        return;
    };

    let screen_pos = game_lib.get_screen_pos(pos);
    let frame_duration = 1.0 / explosion_config.frames_per_second as f32;

    commands.spawn((
        Sprite::from_atlas_image(texture, TextureAtlas { layout, index: 0 }),
        Transform::from_xyz(screen_pos.x, screen_pos.y, explosion_config.z),
        ExplosionComponent {
            timer: Timer::from_seconds(frame_duration, TimerMode::Repeating),
            last_index: explosion_config.frame_count as usize,
        },
    ));
}

fn process_dead_objs(
    dead_objs: &HashMap<Entity, DeadGameObjInfo>,
    phasing_duration: f32,
    map: &mut GameMap,
    game_obj_lib: &mut GameObjInfoLib,
    despawn_pool: &mut DespawnPool,
    commands: &mut Commands,
) {
    for (e, dead_obj) in dead_objs.iter() {
        map.remove_obj(&dead_obj.map_pos, e);
        game_obj_lib.remove(e);
        if !dead_obj.is_phasing {
            despawn_pool.insert(e.clone());
        } else {
            commands
                .entity(e.clone())
                .remove::<AIComponent>()
                .insert(PhasingTimer::new(phasing_duration));
        }
    }
}

fn get_player_info_for_ai(
    player_info: &PlayerInfo,
    game_obj_lib: &GameObjInfoLib,
    game_lib: &GameLib,
) -> Option<(Vec2, f32)> {
    let Some(player_entity) = player_info.0.as_ref() else {
        return None;
    };
    let Some((config_index, player_pos)) = game_obj_lib
        .get(player_entity)
        .map(|obj| (obj.config_index, obj.pos))
    else {
        error!("Failed to find player in GameObjInfoLib");
        return None;
    };
    let player_collide_span = game_lib.get_obj_config(config_index).collide_span;

    Some((player_pos, player_collide_span))
}

// fn get_obj_for_ai<'a, 'b>(
//     entity: &Entity,
//     game_obj_lib: &'a mut GameObjInfoLib,
//     game_lib: &'b GameLib,
// ) -> Option<(&'a mut GameObjInfo, &'b AIConfig)> {
//     let Some(obj) = game_obj_lib.get_mut(&entity) else {
//         error!("Failed to find tank {} in GameObjInfoLib", entity);
//         return None;
//     };
//     let Some(ai_config_name) = game_lib.get_obj_config(obj.config_index).ai_config.as_ref() else {
//         return None;
//     };
//     let Some(ai_config) = game_lib.config.ai_configs.get(ai_config_name) else {
//         error!("Failed to find AIConfig {}", ai_config_name);
//         return None;
//     };

//     Some((obj, ai_config))
// }
