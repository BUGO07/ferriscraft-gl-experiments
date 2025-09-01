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

    let source = std::fs::read_to_string("scripts/entry.lua").unwrap();
    lua.load(&source).exec().unwrap();

    app.world.insert_non_send_resource(lua);

    app.add_systems(Startup, |lua: NonSend<Lua>| lua_func(lua, "OnStartup"))
        .add_systems(Update, |lua: NonSend<Lua>| lua_func(lua, "OnUpdate"));
}

fn lua_func(lua: NonSend<Lua>, name: &str) {
    if let Ok(func) = lua.globals().get::<Function>(name) {
        func.call::<()>(()).unwrap();
    }
}
