use crate::{App, ecs::*};

pub mod movement;

pub fn player_plugin(app: &mut App) {
    app.add_systems(Startup, movement::setup)
        .add_systems(Update, movement::handle_movement);
}
