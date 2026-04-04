use std::{
    sync::{
        // Arc, Mutex,
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
    // thread::{JoinHandle, sleep, spawn},
    // time::Duration,
};

use mlua::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};
use tokio::{
    spawn,
    sync::Mutex,
    task::{JoinHandle, yield_now},
    time::sleep,
};
use tracing::*;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

pub const MIDI_DEV_VAR_NAME: &str = "__MIDI_DEV";
pub const MIDI_CHANNEL_VAR_NAME: &str = "__MIDI_CHANNEL";

// fn do_play_on(
//     lua: &Lua,
//     args: (
//         // the functions being "decorated"
//         LuaFunction, // mlua::Fucntion
//         // midi_dev
//         String,
//         // channel number
//         Option<i8>,
//         //
//         Option<isize>,
//         Option<bool>,
//     ),
// ) {}
//
// TODO: make rust fucntions meathods on a struct and make a unique instance of that struct for
// every import of a module

/// plays a lua function on a device
async fn do_play_on(
    lua: Lua,
    func: LuaFunction, // mlua::Fucntion
    _name: String,
    _midi_dev: String,
    _channel: Option<i8>,
    loop_n: Option<isize>,
    block: Option<bool>,
) -> Option<(JoinHandle<()>, Arc<AtomicBool>)> {
    let should_exit: Arc<AtomicBool> = Arc::new(false.into());

    // call function
    let f = {
        let lua = lua.clone();
        let func = func.clone();
        let should_exit = should_exit.clone();

        async move || {
            // lua.sandbox(true);
            let loop_n = loop_n.unwrap_or(0);

            // store old globals table
            // let old_env = func.environment();

            // set globals table
            // let globals = lua.globals();

            // let set = move |func: &LuaFunction,
            //                 old_env: &Option<LuaTable>,
            //                 midi_dev: &str,
            //                 channel: &Option<i8>|
            //       -> LuaResult<()> {
            // let set = || -> LuaResult<()> {
            //     globals.set(MIDI_DEV_VAR_NAME, midi_dev)?;
            //     globals.set(MIDI_CHANNEL_VAR_NAME, channel)?;
            //     func.set_environment(globals)?;
            //
            //     // let Some(new_env) = old_env.clone() else {
            //     //     error!("failed to get the funtions env attr");
            //     //
            //     //     return Ok(());
            //     // };
            //     // // debug!("{:?}", new_en);
            //     // // let Some(new_env): Option<LuaTable> = new_env.get("_M").ok() else {
            //     // //     error!("failed to find _ENV");
            //     // //
            //     // //     return Ok(());
            //     // // };
            //     //
            //     // new_env.set(MIDI_DEV_VAR_NAME, midi_dev)?;
            //     // new_env.set(MIDI_CHANNEL_VAR_NAME, channel.unwrap_or(0))?;
            //     //
            //     // func.set_environment(new_env)?;
            //
            //     Ok(())
            // };

            // let set = move |lua: &Lua, func: &LuaFunction| -> LuaResult<()> {
            //     let globals = lua.globals();
            //     // Create a custom environment table
            //     let custom_env = lua.create_table()?;
            //     // custom_env.set("x", 100)?;
            //     custom_env.set("print", globals.get::<LuaFunction>("print")?)?;
            //     custom_env.set("play", globals.get::<LuaFunction>("play")?)?;
            //
            //     // globals.set(MIDI_DEV_VAR_NAME, midi_dev)?;
            //     // globals.set(MIDI_CHANNEL_VAR_NAME, channel)?;
            //     custom_env.set(MIDI_DEV_VAR_NAME, midi_dev)?;
            //     custom_env.set(MIDI_CHANNEL_VAR_NAME, channel)?;
            //     // lua.set_globals(globals.clone())?;
            //     lua.set_globals(custom_env.clone())?;
            //     func.set_environment(custom_env)?;
            //
            //     // let Some(new_env) = old_env.clone() else {
            //     //     error!("failed to get the funtions env attr");
            //     //
            //     //     return Ok(());
            //     // };
            //     // // debug!("{:?}", new_en);
            //     // // let Some(new_env): Option<LuaTable> = new_env.get("_M").ok() else {
            //     // //     error!("failed to find _ENV");
            //     // //
            //     // //     return Ok(());
            //     // // };
            //     //
            //     // new_env.set(MIDI_DEV_VAR_NAME, midi_dev)?;
            //     // new_env.set(MIDI_CHANNEL_VAR_NAME, channel.unwrap_or(0))?;
            //     //
            //     // func.set_environment(new_env)?;
            //
            //     Ok(())
            // };

            // let f = |set: fn(&LuaFunction, &Option<LuaTable>) -> LuaResult<()>| {
            // let f = |midi_dev: &str, channel: &Option<i8>| {
            let f = {
                let func = func.clone();

                // let set = move |lua: &Lua, func: &LuaFunction| -> LuaResult<()> {
                //     let globals = lua.globals();
                //     globals.set(MIDI_DEV_VAR_NAME, midi_dev)?;
                //     globals.set(MIDI_CHANNEL_VAR_NAME, channel)?;
                //     func.set_environment(globals)?;
                //
                //     // let Some(new_env) = old_env.clone() else {
                //     //     error!("failed to get the funtions env attr");
                //     //
                //     //     return Ok(());
                //     // };
                //     // // debug!("{:?}", new_en);
                //     // // let Some(new_env): Option<LuaTable> = new_env.get("_M").ok() else {
                //     // //     error!("failed to find _ENV");
                //     // //
                //     // //     return Ok(());
                //     // // };
                //     //
                //     // new_env.set(MIDI_DEV_VAR_NAME, midi_dev)?;
                //     // new_env.set(MIDI_CHANNEL_VAR_NAME, channel.unwrap_or(0))?;
                //     //
                //     // func.set_environment(new_env)?;
                //
                //     Ok(())
                // };

                // move |lua: &Lua| -> LuaResult<()> {
                // move || -> LuaResult<()> {
                async move || -> LuaResult<()> {
                    // let func = func.clone();

                    // if let Err(e) = set(&func, &old_env) {
                    // let old_env = func.environment();

                    // if let Err(e) = set(&func, &old_env, midi_dev, channel) {
                    // if let Err(e) = set(&lua, &func) {
                    //     error!("atempt to set globals resulted in error: {e}");
                    // }
                    // let globals = lua.globals();
                    // globals.set(MIDI_DEV_VAR_NAME, midi_dev.clone())?;
                    // globals.set(MIDI_CHANNEL_VAR_NAME, channel)?;
                    // func.set_environment(globals)?;

                    // let lua_res = lua.load(format!("coroutine.create({name})")).eval();
                    //
                    // let Ok(loc_func): LuaResult<LuaThread> = lua_res else {
                    //     error!("failed to make coroutine");
                    //     error!("{lua_res:?}",);
                    //
                    //     return Ok(());
                    // };

                    let res = func.call_async::<()>(()).await;
                    // let res = loc_func.resume::<()>(());

                    // // restore old globas table
                    // if let Some(old_env) = old_env {
                    //     if let Err(e) = func.set_environment(old_env) {
                    //         error!("failed to reset function enviornment: {e}");
                    //     }
                    // }

                    // lua.sandbox(false);

                    if let Err(ref e) = res {
                        error!("play-on callback function gave error: {e}");
                    } else {
                        debug!("ran fucntion sucesfully");
                    }

                    res
                }
            };

            // debug!("lopping {loop_n} times");

            // if let Err(e) = set() {
            //     error!("atempt to set globals resulted in error: {e}");
            // }

            // lua.sandbox(true);

            // if let Err(e) = set(&lua, &func) {
            //     error!("atempt to set globals resulted in error: {e}");
            // }

            if loop_n > 0 {
                for _ in 0..loop_n {
                    // if f.clone()(&lua).is_err() {
                    if f().await.is_err() {
                        break;
                    }
                }
            } else if loop_n < 0 {
                while !should_exit.load(Ordering::Relaxed) {
                    // if f.clone()(&lua).is_err() {
                    if f().await.is_err() {
                        break;
                    }
                }
            } else {
                _ = f();
            }
            // lua.sandbox(false);

            // // restore old globas table
            // if let Some(old_env) = old_env {
            //     if let Err(e) = func.set_environment(old_env) {
            //         error!("failed to reset function enviornment: {e}");
            //     }
            // }
        }
    };

    if !block.unwrap_or(true) {
        debug!("spawning funtion");
        Some((spawn(async move { f().await }), should_exit))
    } else {
        // jh.join();
        f().await;
        None
    }
}

#[allow(unused)]
/// plays a note
async fn do_play(
    _lua: Lua,
    note: String,
    duration: Option<String>,
    vel: Option<u8>,
    midi_dev: String,
    midi_chan: u8,
    blocking: Option<bool>,
    threads: Arc<Mutex<FxHashSet<(String, String, u8)>>>,
) -> Option<JoinHandle<()>> {
    // parse note into midi_note
    let note_key = (note.clone(), midi_dev.clone(), midi_chan);
    // let note =
    let _play = async move || {
        info!("playing note: {note} -> {midi_dev}:{midi_chan}");
        sleep(Duration::from_secs_f32(0.5)).await;
        threads.lock().await.remove(&note_key);
    };

    if !blocking.unwrap_or(true) {
        Some(spawn(async move {
            info!("non-blocking do_note call");
            _play().await
        }))
    } else {
        info!("a blocking do_note call");
        _play().await;
        None
    }
}

/// sends not off for every note
fn panic() {}

/// returns a string of device names
fn get_devs() -> Vec<String> {
    return Vec::new();
}

/// fuzzy finds a device from the list of devices
fn find_dev(query: &str) -> String {
    let devs = get_devs();

    devs.get(0).unwrap_or(&String::new()).clone()
}

#[tokio::main]
async fn main() -> LuaResult<()> {
    let lua = Lua::new();
    // lua.sandbox(true);
    let globals = lua.globals();
    // let sched = Scheduler::new(lua)?;

    let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("debug"));
    FmtSubscriber::builder()
        .with_file(true)
        .with_line_number(true)
        .with_level(true)
        .with_thread_names(false)
        .with_thread_ids(false)
        .with_env_filter(env_filter)
        .without_time()
        .init();

    let play_on_threads = Arc::new(Mutex::new(FxHashMap::default()));
    let misc_threads = Arc::new(Mutex::new(Vec::<JoinHandle<()>>::default()));
    let playing_notes = Arc::new(Mutex::new(FxHashSet::default()));

    let clear_threads = {
        let misc_threads = misc_threads.clone();
        let play_on_threads = play_on_threads.clone();

        async move || {
            misc_threads
                .lock()
                .await
                .retain(|thread| !thread.is_finished());
            play_on_threads.lock().await.retain(
                |_name, (jh, _should_exit): &mut (JoinHandle<()>, Arc<AtomicBool>)| {
                    !jh.is_finished()
                },
            );
        }
    };

    let play_on = lua.create_async_function({
        // let play_on = lua.create_function({
        let threads = play_on_threads.clone();
        let clear_threads = clear_threads.clone();

        move |lua: Lua,
              // args: (
              (func, midi_dev, channel, loop_n, block): (
            // the functions being "decorated"
            LuaFunction, // mlua::Function,
            // midi_dev
            String,
            // channel number
            Option<i8>,
            // loop_n
            Option<isize>,
            // block
            Option<bool>,
        )| {
            let threads = threads.clone();
            let clear_threads = clear_threads.clone();

            async move {
                let clear_threads = clear_threads.clone();
                // let lua = lua.clone();
                let name = func.info().name.unwrap_or({
                    let base_name = "annon";
                    let n = threads
                        .lock()
                        .await
                        .keys()
                        .clone()
                        .into_iter()
                        // .collect::<Vec<String>>()
                        .filter(|name: &&String| name.starts_with(base_name))
                        // .collect::<Vec<_>>()
                        // .len();
                        .count();
                    format!("{base_name}-{n}")
                });
                debug!("about to play the function, \"{name}\", using do_play_on");

                if let Some((jh, should_exit)) = do_play_on(
                    lua.clone(),
                    func,
                    name.clone(),
                    midi_dev,
                    channel,
                    loop_n,
                    block,
                )
                .await
                {
                    debug!("storing do_play_on thread for callback: \"{name}\"");
                    threads.lock().await.insert(name, (jh, should_exit));
                } else {
                    debug!("do_play_on thread for ccallback: \"{name}\" awaited");
                }

                clear_threads().await;

                Ok(())
            }
        }
    })?;

    globals.set("_do_play_on", play_on)?;
    lua.load_std_libs(LuaStdLib::ALL_SAFE)?;
    globals.set(
        "_do_play",
        // lua.create_function({
        lua.create_async_function({
            let playing_notes = playing_notes.clone();
            let threads = misc_threads.clone();
            let clear_threads = clear_threads.clone();

            move |lua: Lua,
                  (midi_dev, midi_chan, note, duration, vel, blocking): (
                Option<String>,
                Option<u8>,
                String,
                Option<String>,
                Option<u8>,
                Option<bool>,
            )| {
                // let playing_notes = playing_notes.clone();
                // let threads = threads.clone();
                // let clear_threads = clear_threads.clone();
                //
                // async move {
                info!("playing");

                // let g = lua.globals();
                // let Ok(midi_dev) = g.get::<String>(MIDI_DEV_VAR_NAME) else {
                //     error!("{MIDI_DEV_VAR_NAME} not set");
                //
                //     return Ok(());
                // };
                // let Ok(midi_chan) = g.get::<u8>(MIDI_CHANNEL_VAR_NAME) else {
                //     error!("{MIDI_CHANNEL_VAR_NAME} not set");
                //
                //     return Ok(());
                // };

                // put (note, midi_device, channel) in the "playing" hashset
                let midi_dev = midi_dev.unwrap_or("default-midi".into());
                let midi_chan = midi_chan.unwrap_or(0);
                let note_key = (note.clone(), midi_dev.clone(), midi_chan);
                // yield_now().await;

                if !playing_notes.lock().await.contains(&note_key) {
                    playing_notes.lock().await.insert(note_key);

                    if let Some(jh) = do_play(
                        lua.clone(),
                        note,
                        duration,
                        vel,
                        midi_dev,
                        midi_chan,
                        blocking,
                        playing_notes.clone(),
                    )
                    // .await
                    {
                        // threads.lock().unwrap().insert(
                        //     format!("{note} => {midi_dev}:{midi_chan}"),
                        //     (jh, should_exit),
                        // );
                        threads.lock().await.push(jh);
                        // yield_now().await;
                    }
                } else {
                    warn!("{note_key:?} already acounted for...");
                }

                // clear_threads().await;

                Ok(())
                // }
            }
        })?,
    )?;
    globals.set(
        "partial",
        lua.load(
            r#"
            function(f, ...)
                local args = {...} -- Capture initial arguments
                return function(...)
                    local new_args = {...} -- Capture new arguments from the call
                    -- Combine both sets of arguments
                    local final_args = {}
                    for i=1, #args do final_args[i] = args[i] end
                    for i=1, #new_args do final_args[#args + i] = new_args[i] end
                    
                    return f(table.unpack(final_args))
                end
            end
        "#,
        )
        .eval::<LuaFunction>()?,
    )?;
    globals.set(
        "play_on",
        lua.load(format!(
            r#"
            function(func, dev, channel, loop_n, blocking)
                print("inside lua-wrapper")
                
                local m_dev = dev;
                local m_chan = channel;

                print("playing on " .. m_dev)
                local _play = partial(_do_play, m_dev, m_chan)
                _G.play = function(note, dur, vel, blocking); local f = coroutine.wrap(_play); return f(note, dur, vel, blocking); end

                local loc_env = {{ {MIDI_DEV_VAR_NAME} = dev, {MIDI_CHANNEL_VAR_NAME} = channel, _G = _G}}
                api = {{ play = partial(_do_play, m_dev, m_chan), }}
                setmetatable(loc_env, {{ __index = setmetatable(api, {{ __index = _G }}) }})
                do
                    -- local _ENV = loc_env
                    print("midi_dev " .. (dev or "UNASSINED"))
                    -- play("note", "q")
                    -- func()
                    -- f = coroutine.wrap(_do_play_on)
                    -- f(func, dev, channel, loop_n, blocking)
                    _do_play_on(func, dev, channel, loop_n, blocking)

                    -- for _ = 1, (loop_n | 1) do    
                    -- co = coroutine.create(function(); _do_play_on(func, dev, channel, loop_n, blocking); end)
                    -- co()
                    -- while coroutine.status(co) ~= "dead" do
                    --     -- print("Polling... status: " .. coroutine.status(co))
                    --     local success, result = coroutine.resume(co)
                    --     
                    --     if not success then
                    --         print("Error: " .. result)
                    --         break
                    --     end
                    --     -- Optional: sleep/wait briefly here if needed for game loop
                    -- end
                    -- end
                end
            end  
        "#
        ))
        .eval::<LuaFunction>()?,
    )?;

    globals.set(
        "get_devs",
        lua.create_function(|_: &Lua, _: ()| Ok(get_devs()))?,
    )?;
    // globals.set(
    //     "play",
    //     lua.load(format!(
    //         r#"
    //         function(note, duration, vel, blocking)
    //             -- local loc_env = {{ {MIDI_DEV_VAR_NAME} = dev, {MIDI_CHANNEL_VAR_NAME} = channel }}
    //             -- setmetatable(loc_env, {{ __index = _G }})
    //             local tmp_env = getupvalue()
    //
    //             _do_play(note, {MIDI_DEV_VAR_NAME}, {MIDI_CHANNEL_VAR_NAME}, duration, vel, blocking)
    //         end
    //     "#
    //     ))
    //     .eval::<LuaFunction>()?,
    // )?;

    // println!("Hello, world!");
    // if let Err(e) = lua.load(r#"print("hello world from lua!")"#).exec() {
    //     eprintln!("lua failed: {e}");
    //     Err(e)?
    // }

    if let Err(e) = lua
        .load(
            r#"
            -- function kick(api)
            function kick()
                play("c1", "q")
                -- api.play("bd", "q")
            end

            -- function hi_hat(api)
            function hi_hat()
                play("f#1", "s")
                -- api.play("hh", "s")
            end

            -- main()
            loops = 3
            play_on(kick, "some-midi-dev", 1, loops, false)
            play_on(hi_hat, "other-midi-dev", 10, loops, false)
            print("lua: done")
            "#,
        )
        .exec()
    {
        eprintln!("lua failed to run: {e}");
        Err(e)?
    }

    sleep(Duration::from_secs_f32(6.)).await;

    info!("about to stop threads...");

    play_on_threads
        .lock()
        .await
        .iter()
        .for_each(|(name, (_jh, should_exit))| {
            info!("stopping a fucntion named, {name}");
            should_exit.store(true, Ordering::Relaxed);
            // jh.join();
        });

    Ok(())
}
