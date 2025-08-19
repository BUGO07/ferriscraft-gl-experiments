use crate::{Application, ecs::*};

pub mod movement;

pub fn player_plugin(app: &mut Application) {
    app.add_systems(Startup, movement::setup)
        .add_systems(Update, movement::handle_movement);
}
