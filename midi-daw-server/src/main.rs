use crate::midi::{dev::new_midi_dev, out::midi_out};
use actix_web::{web, App, HttpServer};
use crossbeam::channel::unbounded;
use midi_daw_types::UDS_SERVER_PATH;
use std::{sync::RwLock, thread::spawn};

pub mod midi;
pub mod server;

// #[actix::main]
// async fn main() -> std::io::Result<()> {
fn main() -> std::io::Result<()> {
    // tempo
    let tempo = RwLock::new(99.0);

    // prepare mpsc.
    let (midi_msg_out_tx, midi_msg_out_rx) = unbounded();

    let (_jh_1, _jh_2 /* _jh_3 */) = {
        // start midi output thread.
        let (new_midi_dev_tx, new_midi_dev_rx) = unbounded();

        let midi_out_jh = spawn(move || midi_out(midi_msg_out_rx, new_midi_dev_rx));

        // start a thread for midi device discovery.
        let midi_dev_jh = spawn(move || new_midi_dev(new_midi_dev_tx));

        // start a automation thread.
        // let (automation_tx, automation_rx) = unbounded();
        //
        // let automation_jh = spawn(move || automation(automation_rx, midi_msg_out_tx.clone()));

        (midi_out_jh, midi_dev_jh)
    };

    // run webserver.
    server::run(tempo, midi_msg_out_tx)
    // let tempo = web::Data::new(tempo);
    // let midi_out = web::Data::new(midi_msg_out_tx);
    //
    // HttpServer::new(move || {
    //     App::new()
    //         .app_data(tempo.clone())
    //         .app_data(midi_out.clone())
    //         // .service(hello)
    //         // .service(echo)
    //         // .route("/hey", web::get().to(manual_hello))
    //         // .service(note::note)
    //         .service(server::midi)
    // })
    // .bind(("127.0.0.1", 8080))?
    // .bind_uds(UDS_SERVER_PATH)?
    // .run()
    // .await
}
