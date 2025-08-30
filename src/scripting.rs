use std::fs::File;

use crate::{App, ecs::*};
use hlua::Lua;

pub fn scripting_plugin(app: &mut App) {
    let mut lua = Lua::new();
    // lua.set("x", 2);

    lua.set("print", hlua::function1(|text: String| println!("{text}")));
    lua.execute_from_reader::<(), _>(File::open("scripts/entry.lua").unwrap())
        .unwrap();

    app.world.insert_non_send_resource(lua);

    app.add_systems(Startup, lua_startup)
        .add_systems(Update, lua_update);
}

fn lua_startup(mut lua: NonSendMut<Lua>) {
    let mut lua_startup: hlua::LuaFunction<_> = lua.get("OnStartup").unwrap();
    let _: () = lua_startup.call().unwrap();
}

fn lua_update(mut lua: NonSendMut<Lua>) {
    let mut lua_update: hlua::LuaFunction<_> = lua.get("OnUpdate").unwrap();
    let _: () = lua_update.call().unwrap();
}
