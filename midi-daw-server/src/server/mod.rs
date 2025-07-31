// the server should keep track of what notes are being played on each output and should panic then
// kill all notes on client disconnect.

use actix_web::{web, App, HttpServer};
use crossbeam::channel::Sender;
use midi_daw_types::UDS_SERVER_PATH;
use midi_msg::MidiMsg;
use std::sync::RwLock;

#[actix::main]
pub async fn run(tempo: RwLock<f64>, midi_out: Sender<(String, MidiMsg)>) -> std::io::Result<()> {
    let tempo = web::Data::new(tempo);
    let midi_out = web::Data::new(midi_out);

    HttpServer::new(|| {
        App::new()
            .app_data(tempo)
            .app_data(midi_out)
            // .service(hello)
            // .service(echo)
            // .route("/hey", web::get().to(manual_hello))
            .service(note)
    })
    .bind(("127.0.0.1", 8080))?
    .bind_uds(UDS_SERVER_PATH)?
    .run()
    .await
}
