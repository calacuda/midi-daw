// the server should keep track of what notes are being played on each output and should panic then
// kill all notes on client disconnect.

use crate::server::note::{pitch_bend, play_note, send_cc, stop_note};
use actix_web::{
    post,
    web::{self, Json},
    App, HttpResponse, HttpServer, Responder,
};
use crossbeam::channel::Sender;
use log::*;
use midi_daw_types::{MidiMsg, MidiReqBody, UDS_SERVER_PATH};
use std::sync::RwLock;

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
            if let Ok(tempo) = tempo.read() {
                play_note(*tempo, midi_out, dev, channel, note, velocity, duration).await;
            } else {
                error!("failed to read tempo");
            }
        }
        MidiMsg::StopNote { note } => stop_note(midi_out, dev, channel, note).await,
        MidiMsg::CC { control, value } => send_cc(midi_out, dev, channel, control, value).await,
        MidiMsg::PitchBend { bend } => pitch_bend(midi_out, dev, channel, bend).await,
    }

    HttpResponse::Ok()
}

#[actix::main]
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
    })
    .bind(("127.0.0.1", 8888))?
    .bind_uds(UDS_SERVER_PATH)?
    .run()
    .await
}
