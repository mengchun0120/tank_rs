use crate::game_lib::*;
use crate::game_map::*;
use crate::game_obj::*;
use crate::utils::*;
use bevy::prelude::*;
use std::path::Path;

pub fn setup_game(
    args: Res<Args>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut exit_app: MessageWriter<AppExit>,
    mut window: Single<&mut Window>,
) {
    let Some(game_lib) = load_game_lib(
        args.config_path.as_path(),
        asset_server.as_ref(),
        &mut exit_app,
    ) else {
        return;
    };

    init_window(&game_lib.config, window.as_mut());

    commands.spawn(Camera2d);

    if !load_map(
        args.map_path.as_path(),
        &game_lib,
        &mut commands,
        &mut exit_app,
    ) {
        return;
    }

    commands.insert_resource(game_lib);

    info!("Setup finished");
}

pub fn process_input(
    keys: Res<ButtonInput<KeyCode>>,
    game_lib: Res<GameLib>,
    mut player: Single<(Entity, &mut Transform, &mut ShootComponent), With<PlayerComponent>>,
    mut map: ResMut<GameMap>,
    time: Res<Time>,
) {
    if keys.just_pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::ArrowRight) {
        move_player(
            Direction::Right,
            game_lib.as_ref(),
            &mut player,
            map.as_mut(),
            time.as_ref(),
        )
    } else if keys.just_pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::ArrowLeft) {
        move_player(
            Direction::Left,
            game_lib.as_ref(),
            &mut player,
            map.as_mut(),
            time.as_ref(),
        )
    } else if keys.just_pressed(KeyCode::ArrowUp) || keys.pressed(KeyCode::ArrowUp) {
        move_player(
            Direction::Up,
            game_lib.as_ref(),
            &mut player,
            map.as_mut(),
            time.as_ref(),
        )
    } else if keys.just_pressed(KeyCode::ArrowDown) || keys.pressed(KeyCode::ArrowDown) {
        move_player(
            Direction::Down,
            game_lib.as_ref(),
            &mut player,
            map.as_mut(),
            time.as_ref(),
        )
    }
}

fn update_missile(
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
}

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
    game_lib: &GameLib,
    commands: &mut Commands,
    exit_app: &mut MessageWriter<AppExit>,
) -> bool {
    let map_config: GameMapConfig = match read_json(map_path.as_ref()) {
        Ok(config) => config,
        Err(err) => {
            error!("Failed to load map config: {}", err);
            exit_app.write(AppExit::error());
            return false;
        }
    };

    let map = GameMap::load(&map_config, game_lib, commands);
    commands.insert_resource(map);

    true
}

fn move_player(
    d: Direction,
    game_lib: &GameLib,
    player: &mut Single<(Entity, &mut Transform, &mut ShootComponent), With<PlayerComponent>>,
    map: &mut GameMap,
    time: &Time,
) {
    let new_direction: Vec2 = d.into();
    let Some((old_pos, old_direction, obj_config)) = map.get_obj(&player.0).map(|obj| {
        (
            obj.pos,
            obj.direction,
            game_lib.get_obj_config(obj.config_index),
        )
    }) else {
        warn!("Cannot find player in map");
        return;
    };

    if new_direction != old_direction {
        map.change_direction(player.0, new_direction);
        player.1.rotation = get_rotation(new_direction);

        if let Some(shoot_config) = obj_config.shoot_config.as_ref() {
            let shoot_pos = arr_to_vec2(&shoot_config.shoot_position);
            player.2.shoot_pos = old_pos + new_direction.rotate(shoot_pos);
        }
    } else {
        let Some((_, new_pos)) = map.move_obj(&player.0, game_lib, time.delta_secs()) else {
            return;
        };

        let new_screen_pos = game_lib.get_screen_pos(&new_pos);
        player.1.translation.x = new_screen_pos.x;
        player.1.translation.y = new_screen_pos.y;
    }
}
