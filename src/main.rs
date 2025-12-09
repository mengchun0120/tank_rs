mod ai;
mod game_lib;
mod game_map;
mod game_obj;
mod my_error;
mod systems;
mod utils;

use crate::systems::*;
use crate::utils::*;
use bevy::{log::LogPlugin, prelude::*};
use clap::Parser;

fn main() {
    let args = Args::parse();
    let _guard = setup_log(&args.log_path);

    App::new()
        .add_plugins(DefaultPlugins.build().disable::<LogPlugin>())
        .insert_resource(args)
        .add_systems(Startup, setup_game)
        .add_systems(FixedUpdate, update_ai)
        .add_systems(
            Update,
            (
                process_input,
                update_missiles,
                update_explosions,
                update_phasing_objs,
            ),
        )
        .add_systems(PostUpdate, cleanup)
        .run();
}
