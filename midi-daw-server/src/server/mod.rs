use crate::server::note::{pitch_bend, play_note, send_cc, stop_note};
use actix_web::{
    get, post,
    web::{self, Json},
    App, HttpResponse, HttpServer, Responder,
};
use async_std::sync::RwLock;
use crossbeam::channel::Sender;
use midi_daw_types::{MidiMsg, MidiReqBody, UDS_SERVER_PATH};

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
            // if let Ok(tempo) = tempo.read() {
            let tempo = tempo.read().await;
            play_note(*tempo, midi_out, dev, channel, note, velocity, duration).await;
            // } else {
            // error!("failed to read tempo");
            // }
        }
        MidiMsg::StopNote { note } => stop_note(midi_out, dev, channel, note).await,
        MidiMsg::CC { control, value } => send_cc(midi_out, dev, channel, control, value).await,
        MidiMsg::PitchBend { bend } => pitch_bend(midi_out, dev, channel, bend).await,
    }

    HttpResponse::Ok()
}

#[post("tempo")]
async fn set_tempo(
    tempo: web::Data<RwLock<f64>>,
    // midi_out: web::Data<MidiOut>,
    req_body: Json<f64>,
) -> impl Responder {
    // if let Ok(mut tempo) = tempo.write() {
    let mut tempo = tempo.write().await;
    *tempo = *req_body;
    // } else {
    //     error!("faling to set tempo bc failed to get write lock.");
    // }

    HttpResponse::Ok()
}

#[get("tempo")]
async fn get_tempo(
    tempo: web::Data<RwLock<f64>>,
    // midi_out: web::Data<MidiOut>,
    // req_body: Json<f64>,
) -> impl Responder {
    // if let Ok(mut tempo) = tempo.write() {
    let tempo = tempo.read().await;
    // *tempo = *req_body;
    // } else {
    //     error!("faling to set tempo bc failed to get write lock.");
    // }

    serde_json::to_string(&*tempo).map(|tempo| HttpResponse::Ok().body(tempo))
}

// #[actix::main]
pub async fn run(tempo: RwLock<f64>, midi_out: MidiOut) -> std::io::Result<()> {
    let tempo = web::Data::new(tempo);
    let midi_out = web::Data::new(midi_out);

    HttpServer::new(move || {
        App::new()
            .app_data(tempo.clone())
            .app_data(midi_out.clone())
            // .service(hello)
            // .service(echo)
            // .route("/hey", web::get().to(manual_hello))
            // .service(note::note)
            .service(midi)
            .service(get_tempo)
            .service(set_tempo)
    })
    .bind(("127.0.0.1", 8888))?
    .bind_uds(UDS_SERVER_PATH)?
    .run()
    .await
}
