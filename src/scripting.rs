use mlua::{Function, Lua};

use crate::{App, ecs::*};

pub fn scripting_plugin(app: &mut App) {
    let lua = Lua::new();

    lua.globals()
        .set(
            "print",
            lua.create_function(|_, text: String| {
                println!("{text}");
                Ok(())
            })
            .unwrap(),
        )
        .unwrap();

    lua.load(std::fs::read_to_string("scripts/entry.lua").unwrap())
        .exec()
        .unwrap();

    app.world.insert_non_send_resource(lua);

    app.add_systems(Startup, |lua: NonSend<Lua>| lua_func(lua, "OnStartup"))
        .add_systems(Update, |lua: NonSend<Lua>| lua_func(lua, "OnUpdate"))
        .add_systems(Exiting, |lua: NonSend<Lua>| lua_func(lua, "OnExit"));
}

fn lua_func(lua: NonSend<Lua>, name: &str) {
    if let Ok(func) = lua.globals().get::<Function>(name) {
        func.call::<()>(()).unwrap();
    }
}
