use crate::server::note::{pitch_bend, play_note, send_cc, stop_note};
use actix_web::{
    get, post,
    web::{self, Json},
    App, HttpResponse, HttpServer, Responder,
};
use async_std::sync::RwLock;
use crossbeam::channel::Sender;
use midi_daw_types::{MidiMsg, MidiReqBody, NoteDuration, UDS_SERVER_PATH};
use midir::MidiOutput;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

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
        .filter_map(|port| midi_out.port_name(&port).ok())
        .collect();

    serde_json::to_string(&midi_devs_names).map(|tempo| HttpResponse::Ok().body(tempo))
}

pub async fn run(tempo: RwLock<f64>, midi_out: MidiOut) -> std::io::Result<()> {
    let tempo = web::Data::new(tempo);
    let midi_out = web::Data::new(midi_out);

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

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(tempo.clone())
            .app_data(midi_out.clone())
            .service(midi)
            .service(get_devs)
            .service(get_tempo)
            .service(set_tempo)
            .service(rest)
    })
    .bind(("127.0.0.1", 8888))?
    .bind_uds(UDS_SERVER_PATH)?
    .run()
    .await
}
