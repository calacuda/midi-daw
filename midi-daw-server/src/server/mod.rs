use crate::{
    midi::dev::fmt_dev_name,
    server::{
        message_bus::{MbServer, MbServerHandle},
        note::{pitch_bend, play_note, send_cc, stop_note},
    },
};
use actix::{clock::sleep, spawn};
use actix_web::{
    get, post,
    web::{self, Json},
    App, HttpResponse, HttpServer, Responder,
};
use async_std::sync::RwLock;
use crossbeam::channel::Sender;
use midi_daw_types::{MidiMsg, MidiReqBody, NoteDuration, UDS_SERVER_PATH};
use midir::MidiOutput;
use std::time::Duration;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

mod message_bus;
mod note;

pub type MidiOut = Sender<(String, midi_msg::MidiMsg)>;

#[post("/midi")]
async fn midi(
    tempo: web::Data<RwLock<f64>>,
    midi_out: web::Data<MidiOut>,
    req_body: Json<MidiReqBody>,
) -> impl Responder {
    let dev = req_body.midi_dev.clone();
    let channel = req_body.channel;

    match req_body.msg {
        MidiMsg::PlayNote {
            note,
            velocity,
            duration,
        } => {
            let tempo = tempo.read().await;
            play_note(*tempo, midi_out, dev, channel, note, velocity, duration).await;
        }
        MidiMsg::StopNote { note } => stop_note(midi_out, dev, channel, note).await,
        MidiMsg::CC { control, value } => send_cc(midi_out, dev, channel, control, value).await,
        MidiMsg::PitchBend { bend } => pitch_bend(midi_out, dev, channel, bend).await,
    }

    HttpResponse::Ok()
}

#[post("/rest")]
async fn rest(tempo: web::Data<RwLock<f64>>, durration: Json<NoteDuration>) -> impl Responder {
    let tempo = tempo.read().await;

    // serde_json::to_string(&*tempo).map(|tempo| HttpResponse::Ok().body(tempo))
    note::rest(*tempo, *durration).await;

    HttpResponse::Ok()
}

#[post("/tempo")]
async fn set_tempo(tempo: web::Data<RwLock<f64>>, req_body: Json<f64>) -> impl Responder {
    let mut tempo = tempo.write().await;
    *tempo = *req_body;

    HttpResponse::Ok()
}

#[get("/tempo")]
async fn get_tempo(tempo: web::Data<RwLock<f64>>) -> impl Responder {
    let tempo = tempo.read().await;
    serde_json::to_string(&*tempo).map(|tempo| HttpResponse::Ok().body(tempo))
}

#[get("/midi")]
async fn get_devs() -> impl Responder {
    let midi_out = MidiOutput::new("MIDI-DAW-API").unwrap();
    let midi_devs_names: Vec<String> = midi_out
        .ports()
        .into_iter()
        .filter_map(|port| midi_out.port_name(&port).ok().map(fmt_dev_name))
        // .map(|port| port.id())
        .collect();

    serde_json::to_string(&midi_devs_names).map(|tempo| HttpResponse::Ok().body(tempo))
}

/// sends a message to the message bus every note
pub async fn clock_notif(data: MbServerHandle, tempo: web::Data<RwLock<f64>>) -> ! {
    let mut sn = 0;
    let id = uuid::Uuid::new_v4();
    let msgs: Vec<String> = vec![
        "1", "1e", "1&", "1a", "2", "2e", "2&", "2a", "3", "3e", "3&", "3a", "4", "4e", "4&", "4a",
    ]
    .into_iter()
    .map(String::from)
    .collect();

    loop {
        let message: String = msgs[sn].clone();

        // let msg = MbMessageWrapper { id, message };

        // data.do_send(msg);
        data.send_message(id, message).await;

        sn += 1;
        sn %= 16;

        let sleep_time = (*tempo.read().await / 60.0) * 2.0 / 16.0;
        sleep(Duration::from_secs_f64(sleep_time)).await
    }
}

pub async fn run(tempo: RwLock<f64>, midi_out: MidiOut) -> std::io::Result<()> {
    let tempo = web::Data::new(tempo);
    let midi_out = web::Data::new(midi_out);
    // let msg_event_addr = web::Data::new(MbMessageEvent.start());
    let (mb_server, server_tx) = MbServer::new();

    let _chat_server = spawn(mb_server.run());

    // Filter based on level - trace, debug, info, warn, error
    // Tunable via `RUST_LOG` env variable
    let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info"));
    FmtSubscriber::builder()
        .with_file(true)
        .with_line_number(true)
        .with_level(true)
        .with_thread_names(false)
        .with_thread_ids(false)
        .with_env_filter(env_filter)
        .without_time()
        .init();

    let _jh = spawn(clock_notif(server_tx.clone(), tempo.clone()));

    HttpServer::new({
        let server_tx = web::Data::new(server_tx);
        move || {
            App::new()
                .wrap(TracingLogger::default())
                .app_data(tempo.clone())
                .app_data(midi_out.clone())
                .app_data(server_tx.clone())
                .service(midi)
                .service(get_devs)
                .service(get_tempo)
                .service(set_tempo)
                .service(rest)
                .service(message_bus::message_bus)
        }
    })
    .worker_max_blocking_threads(1)
    .workers(12)
    .bind(("127.0.0.1", 8888))?
    .bind_uds(UDS_SERVER_PATH)?
    .run()
    .await

    // try_join!(http_server, async move { chat_server.await.unwrap() })?;

    // Ok(())
}
