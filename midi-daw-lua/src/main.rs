use mlua::prelude::*;

/// plays a lua function on a device
fn play_on(
    lua: Lua,
    midi_dev: String,
    channel: Option<i8>,
    loop_n: Option<isize>,
    block: Option<bool>,
) {
    // store old globals table
    // set globals table
    // call function (maybe in a loop)
    // restore old globas table
}

/// plays a note
fn play() {}

/// returns a string of device names
fn get_devs() -> Vec<String> {
    return Vec::new();
}

/// fuzzy finds a device from the list of devices
fn find_dev(query: &str) -> String {
    let devs = get_devs();

    devs.get(0).unwrap_or(&String::new()).clone()
}

fn main() {
    let lua = Lua::new();
    let globals = lua.globals();

    // println!("Hello, world!");
    if let Err(e) = lua.load(r#"print("hello world from lua!")"#).exec() {
        eprintln!("lua failed: {e}");
    }
}
