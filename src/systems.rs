use crate::game_lib::*;
use crate::utils::*;
use bevy::prelude::*;

pub fn setup_game(
    args: Res<Args>,
    mut commands: Commands,
    mut exit_app: MessageWriter<AppExit>,
    mut window: Single<&mut Window>,
) {
    let game_lib = match GameLib::new(
        args.config_path.as_path(),
    ) {
        Ok(lib) => lib,
        Err(err) => {
            error!("Failed to initialize GameLib {}", err);
            exit_app.write(AppExit::error());
            return;
        }
    };

    let config = &game_lib.config;

    window
        .resolution
        .set(config.window_width(), config.window_height());


    commands.insert_resource(game_lib);

    info!("Setup finished");
}
