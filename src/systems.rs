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
    mut commands: Commands,
    mut player: Single<(Entity, &mut Transform, &mut DirectionComponent), With<PlayerComponent>>,
    mut map: ResMut<GameMap>,
) {
    if keys.just_pressed(KeyCode::ArrowRight) {
        change_direction_player(Direction::Right, &mut player);
    } else if keys.just_pressed(KeyCode::ArrowLeft) {
        change_direction_player(Direction::Left, &mut player);
    } else if keys.just_pressed(KeyCode::ArrowUp) {
        change_direction_player(Direction::Up, &mut player);
    } else if keys.just_pressed(KeyCode::ArrowDown) {
        change_direction_player(Direction::Down, &mut player);
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

fn change_direction_player(
    d: Direction,
    player: &mut Single<(Entity, &mut Transform, &mut DirectionComponent), With<PlayerComponent>>,
) {
    let new_direction = d.into();
    player.1.rotation = GameObj::get_rotation(&new_direction);
    player.2.0 = new_direction;
}