use crate::{
    midi::{MidiDev, dev::fmt_dev_name, out::unwrap_rw_lock},
    sequencer::SequencerControlCmd,
    server::{
        message_bus::{MbServer, MbServerHandle},
        note::{pitch_bend, play_note, send_cc, stop_note},
    },
};
use actix::spawn;
use actix_web::{
    App, HttpResponse, HttpResponseBuilder, HttpServer, get, post,
    web::{self, Json},
};
use crossbeam::channel::Sender;
use futures::future::join_all;
use fx_hash::FxHashSet;
use midi_daw_types::{
    AddNoteBody, ChangeLenByBody, GetSequenceQuery, MidiMsg, MidiReqBody, NoteDuration,
    RenameSequenceBody, RmNoteBody, SetChannelBody, SetDevBody, UDS_SERVER_PATH,
};
pub use midi_daw_types::{BPQ, Tempo};
use midir::MidiOutput;
use std::time::Duration;
use tokio::{
    sync::{Mutex, oneshot},
    task::spawn_local,
};
use tracing::log::*;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

mod message_bus;
mod note;

pub type MidiOut = Sender<(String, midi_msg::MidiMsg)>;

#[post("/midi")]
async fn midi(
    tempo: web::Data<Tempo>,
    midi_out: web::Data<MidiOut>,
    req_body: Json<MidiReqBody>,
) -> HttpResponseBuilder {
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
            }
        }
        MidiMsg::StopNote { note } => stop_note(midi_out, dev, channel, note).await,
        MidiMsg::CC { control, value } => send_cc(midi_out, dev, channel, control, value).await,
        MidiMsg::PitchBend { bend } => pitch_bend(midi_out, dev, channel, bend).await,
    }

    HttpResponse::Ok()
}

#[post("/batch-midi")]
async fn midi_pool_exec(
    tempo: web::Data<Tempo>,
    midi_out: web::Data<MidiOut>,
    req_body: Json<Vec<MidiReqBody>>,
) -> HttpResponseBuilder {
    join_all(req_body.clone().into_iter().map(async |msg| {
        let tempo = tempo.clone();
        let midi_out = midi_out.clone();

        spawn_local(async move {
            let dev = msg.midi_dev.clone();
            let channel = msg.channel;

            match msg.msg {
                MidiMsg::PlayNote {
                    note,
                    velocity,
                    duration,
                } => {
                    debug!("note: {note} => {dev}");

                    if let Ok(tempo) = tempo.read() {
                        play_note(
                            *tempo,
                            midi_out.clone(),
                            dev,
                            channel,
                            note,
                            velocity,
                            duration,
                        )
                        .await;
                    }
                }
                MidiMsg::StopNote { note } => stop_note(midi_out.clone(), dev, channel, note).await,
                MidiMsg::CC { control, value } => {
                    send_cc(midi_out.clone(), dev, channel, control, value).await
                }
                MidiMsg::PitchBend { bend } => {
                    pitch_bend(midi_out.clone(), dev, channel, bend).await
                }
            }
        })
    }))
    .await;

    HttpResponse::Ok()
}

#[post("/rest")]
async fn rest(tempo: web::Data<Tempo>, durration: Json<NoteDuration>) -> HttpResponseBuilder {
    // let tempo = tempo.read().await;
    if let Ok(tempo) = tempo.read() {
        // serde_json::to_string(&*tempo).map(|tempo| HttpResponse::Ok().body(tempo))
        note::rest(*tempo, *durration).await;
    }

    HttpResponse::Ok()
}

#[post("/tempo")]
async fn set_tempo(tempo: web::Data<Tempo>, req_body: Json<f64>) -> HttpResponseBuilder {
    // let mut tempo = tempo.write().await;
    if let Ok(mut tempo) = tempo.write() {
        *tempo = *req_body;
    }

    HttpResponse::Ok()
}

#[get("/tempo")]
async fn get_tempo(tempo: web::Data<Tempo>) -> HttpResponse {
    // let tempo = tempo.read().await;
    if let Ok(tempo) = tempo.read() {
        if let Ok(tempo_json) = serde_json::to_string(&*tempo) {
            return HttpResponse::Ok().body(tempo_json);
        }
    }

    HttpResponse::InternalServerError().finish()
}

#[get("/midi")]
async fn get_devs(
    virtual_devs: web::Data<Mutex<FxHashSet<String>>>,
) -> Result<HttpResponse, serde_json::Error> {
    let midi_out = MidiOutput::new("MIDI-DAW-API").unwrap();
    let mut midi_devs_names: Vec<String> = midi_out
        .ports()
        .into_iter()
        .filter_map(|port| midi_out.port_name(&port).ok().map(fmt_dev_name))
        // .map(|port| port.id())
        .collect();
    midi_devs_names.append(&mut virtual_devs.lock().await.clone().into_iter().collect());

    info!("{midi_devs_names:?}");

    serde_json::to_string(&midi_devs_names).map(|tempo| HttpResponse::Ok().body(tempo))
}

// make an end point to make new virtual midi-out
#[post("/new-dev")]
async fn new_dev(
    req_body: Json<String>,
    new_dev_tx: web::Data<Sender<MidiDev>>,
    virtual_devs: web::Data<Mutex<FxHashSet<String>>>,
) -> Result<HttpResponse, serde_json::Error> {
    let port_name = &req_body.0;

    // if let Err(e) = dev_id {
    //     error!("making new dev {e}");
    // } else {
    //     info!("made output device {port_name}");
    // }

    if let Err(e) = new_dev_tx.send(MidiDev::CreateVirtual(port_name.to_string())) {
        error!("{e}");
    }
    virtual_devs.lock().await.insert(port_name.to_string());

    serde_json::to_string(&port_name).map(|tempo| HttpResponse::Ok().body(tempo))
}

#[post("/sequence/new")]
async fn new_sequence(
    seq_coms: web::Data<Sender<SequencerControlCmd>>,
    seq_name: Json<String>,
) -> HttpResponseBuilder {
    let msg = SequencerControlCmd::NewSequence {
        name: Some(seq_name.0),
        midi_dev: None,
        channel: None,
    };
    match seq_coms.send(msg) {
        Ok(_) => HttpResponse::Ok(),
        Err(e) => {
            error!("{e}");
            HttpResponse::InternalServerError()
        }
    }
}

#[post("/sequence/rm")]
async fn rm_sequence(
    seq_coms: web::Data<Sender<SequencerControlCmd>>,
    seq_name: Json<String>,
) -> HttpResponseBuilder {
    let msg = SequencerControlCmd::RmSequence { name: seq_name.0 };

    match seq_coms.send(msg) {
        Ok(_) => HttpResponse::Ok(),
        Err(e) => {
            error!("{e}");
            HttpResponse::InternalServerError()
        }
    }
}

async fn do_get_sequences(seq_coms: web::Data<Sender<SequencerControlCmd>>) -> HttpResponse {
    let (responder, mut recv_er) = oneshot::channel();

    let msg = SequencerControlCmd::GetSequences { responder };

    match seq_coms.send(msg) {
        Ok(_) => loop {
            if let Ok(res) = recv_er.try_recv() {
                return HttpResponse::Ok().json(res);
            }
        },
        Err(e) => {
            let error_msg = format!("sending control message to sequencer failed with error, {e}");

            error!("{error_msg}");
            HttpResponse::InternalServerError().body(error_msg)
        }
    }
}

#[get("/sequence/names")]
async fn get_sequences(seq_coms: web::Data<Sender<SequencerControlCmd>>) -> HttpResponse {
    do_get_sequences(seq_coms).await
}

#[get("/sequence")]
async fn get_sequence(
    seq_coms: web::Data<Sender<SequencerControlCmd>>,
    seq_name: web::Query<GetSequenceQuery>,
) -> HttpResponse {
    let (responder, mut recv_er) = oneshot::channel();

    let msg = SequencerControlCmd::GetSequence {
        sequence: seq_name.sequence.clone(),
        responder,
    };

    match seq_coms.send(msg) {
        Ok(_) => match {
            loop {
                if let Ok(res) = recv_er.try_recv() {
                    break res;
                }
            }
        } {
            Some(sequences) => HttpResponse::Ok().json(sequences),
            None => {
                let error_msg = format!("reading reponse from sequencer failed");

                error!("{error_msg}");
                HttpResponse::InternalServerError().body(error_msg)
            }
        },
        Err(e) => {
            let error_msg = format!("sending control message to sequencer failed with error, {e}");

            error!("{error_msg}");
            HttpResponse::InternalServerError().body(error_msg)
        }
    }
}

#[post("/sequence/play-one")]
async fn play_sequence(
    seq_coms: web::Data<Sender<SequencerControlCmd>>,
    seq_name: Json<String>,
) -> HttpResponseBuilder {
    let msg = SequencerControlCmd::Play(vec![seq_name.0.clone()]);

    match seq_coms.send(msg) {
        Ok(_) => {
            info!("now playing: {}.", seq_name.0);
            HttpResponse::Ok()
        }
        Err(e) => {
            error!("{e}");
            HttpResponse::InternalServerError()
        }
    }
}

#[post("/sequence/play-these")]
async fn play_these_sequences(
    seq_coms: web::Data<Sender<SequencerControlCmd>>,
    seq_name: Json<Vec<String>>,
) -> HttpResponseBuilder {
    let msg = SequencerControlCmd::Play(seq_name.0.clone());

    match seq_coms.send(msg) {
        Ok(_) => {
            info!("playing the folowing sequences: {:?}", seq_name.0);
            HttpResponse::Ok()
        }
        Err(e) => {
            error!("{e}");
            HttpResponse::InternalServerError()
        }
    }
}

#[post("/sequence/play-all")]
async fn play_all_sequence(
    seq_coms: web::Data<Sender<SequencerControlCmd>>,
) -> HttpResponseBuilder {
    let msg = SequencerControlCmd::PlayAll;

    match seq_coms.send(msg) {
        Ok(_) => {
            info!("playing every sequence.");
            HttpResponse::Ok()
        }
        Err(e) => {
            error!("{e}");
            HttpResponse::InternalServerError()
        }
    }
}

#[post("/sequence/pause")]
async fn pause_sequence(
    seq_coms: web::Data<Sender<SequencerControlCmd>>,
    seq_name: Json<Vec<String>>,
) -> HttpResponseBuilder {
    let msg = SequencerControlCmd::Pause(seq_name.0);

    match seq_coms.send(msg) {
        Ok(_) => HttpResponse::Ok(),
        Err(e) => {
            error!("{e}");
            HttpResponse::InternalServerError()
        }
    }
}

#[post("/sequence/pause-all")]
async fn pause_all_sequence(
    seq_coms: web::Data<Sender<SequencerControlCmd>>,
) -> HttpResponseBuilder {
    let msg = SequencerControlCmd::PauseAll;

    match seq_coms.send(msg) {
        Ok(_) => HttpResponse::Ok(),
        Err(e) => {
            error!("{e}");
            HttpResponse::InternalServerError()
        }
    }
}

#[post("/sequence/stop-one")]
async fn stop_sequence(
    seq_coms: web::Data<Sender<SequencerControlCmd>>,
    seq_name: Json<String>,
) -> HttpResponseBuilder {
    let msg = SequencerControlCmd::Stop(vec![seq_name.0]);

    match seq_coms.send(msg) {
        Ok(_) => HttpResponse::Ok(),
        Err(e) => {
            error!("{e}");
            HttpResponse::InternalServerError()
        }
    }
}

#[post("/sequence/stop-these")]
async fn stop_some_sequence(
    seq_coms: web::Data<Sender<SequencerControlCmd>>,
    seq_name: Json<Vec<String>>,
) -> HttpResponseBuilder {
    let msg = SequencerControlCmd::Stop(seq_name.0);

    match seq_coms.send(msg) {
        Ok(_) => HttpResponse::Ok(),
        Err(e) => {
            error!("{e}");
            HttpResponse::InternalServerError()
        }
    }
}

#[post("/sequence/stop-all")]
async fn stop_all_sequence(
    seq_coms: web::Data<Sender<SequencerControlCmd>>,
) -> HttpResponseBuilder {
    let msg = SequencerControlCmd::StopAll;

    match seq_coms.send(msg) {
        Ok(_) => HttpResponse::Ok(),
        Err(e) => {
            error!("{e}");
            HttpResponse::InternalServerError()
        }
    }
}

#[post("/sequence/add-note")]
async fn add_note(
    seq_coms: web::Data<Sender<SequencerControlCmd>>,
    args: Json<AddNoteBody>,
) -> HttpResponseBuilder {
    let msg = SequencerControlCmd::AddNote {
        sequence: args.sequence.clone(),
        step: args.step,
        note: args.note,
        velocity: args.velocity,
        note_len: args.note_len,
    };

    match seq_coms.send(msg) {
        Ok(_) => HttpResponse::Ok(),
        Err(e) => {
            error!("{e}");
            HttpResponse::InternalServerError()
        }
    }
}

#[post("/sequence/rm-note")]
async fn rm_note(
    seq_coms: web::Data<Sender<SequencerControlCmd>>,
    args: Json<RmNoteBody>,
) -> HttpResponseBuilder {
    let msg = SequencerControlCmd::RmNote {
        sequence: args.sequence.clone(),
        step: args.step,
        note: args.note,
    };

    match seq_coms.send(msg) {
        Ok(_) => HttpResponse::Ok(),
        Err(e) => {
            error!("{e}");
            HttpResponse::InternalServerError()
        }
    }
}

#[post("/sequence/set-dev")]
async fn set_dev(
    seq_coms: web::Data<Sender<SequencerControlCmd>>,
    args: Json<SetDevBody>,
) -> HttpResponseBuilder {
    let msg = SequencerControlCmd::SetSequenceDev {
        name: args.sequence.clone(),
        midi_dev: args.midi_dev.clone(),
    };

    match seq_coms.send(msg) {
        Ok(_) => HttpResponse::Ok(),
        Err(e) => {
            error!("{e}");
            HttpResponse::InternalServerError()
        }
    }
}

#[post("/sequence/rename")]
async fn rename_sequence(
    seq_coms: web::Data<Sender<SequencerControlCmd>>,
    args: Json<RenameSequenceBody>,
) -> HttpResponse {
    let msg = SequencerControlCmd::RenameSequence {
        old_name: args.old_name.clone(),
        new_name: args.new_name.clone(),
    };

    match seq_coms.send(msg) {
        Ok(_) => do_get_sequences(seq_coms).await,
        Err(e) => {
            let error_msg = format!("sending control message to sequencer failed with error, {e}");

            error!("{error_msg}");
            HttpResponse::InternalServerError().body(error_msg)
        }
    }
}

#[post("/sequence/set-channel")]
async fn set_channel(
    seq_coms: web::Data<Sender<SequencerControlCmd>>,
    args: Json<SetChannelBody>,
) -> HttpResponse {
    let msg = SequencerControlCmd::SetSequenceChannel {
        name: args.sequence.clone(),
        channel: args.channel.clone(),
    };

    match seq_coms.send(msg) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            let error_msg = format!("sending control message to sequencer failed with error, {e}");

            error!("{error_msg}");
            HttpResponse::InternalServerError().body(error_msg)
        }
    }
}

#[post("/sequence/change-len-by")]
async fn change_len_by(
    seq_coms: web::Data<Sender<SequencerControlCmd>>,
    args: Json<ChangeLenByBody>,
) -> HttpResponse {
    let msg = SequencerControlCmd::ChangeLenBy {
        sequence: args.sequence.clone(),
        amt: args.amt.clone(),
    };

    match seq_coms.send(msg) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            let error_msg = format!("sending control message to sequencer failed with error, {e}");

            error!("{error_msg}");
            HttpResponse::InternalServerError().body(error_msg)
        }
    }
}

/// sends a message to the message bus every note
pub fn clock_notif(data: MbServerHandle, tempo: web::Data<Tempo>) -> ! {
    // TODO: make this a client running in a syncronouse std::thread
    let mut sn = 0;
    let id = uuid::Uuid::new_v4();
    let msgs: Vec<String> = vec![
        "1", "1e", "1&", "1a", "2", "2e", "2&", "2a", "3", "3e", "3&", "3a", "4", "4e", "4&", "4a",
    ]
    .into_iter()
    .map(String::from)
    .collect();

    loop {
        // start a sleep thread to sleep for sleep_time.
        let sleep_thread = std::thread::spawn({
            // calculate sleep_time based on BPQ & tempo.
            let tempo = unwrap_rw_lock(&tempo, 99.);
            let sleep_time = Duration::from_secs_f64((60.0 / tempo) / 4.);

            move || {
                std::thread::sleep(sleep_time);
            }
        });

        let message: String = msgs[sn].clone();

        data.send_message(id, &message);
        // debug!("{message}");

        sn += 1;
        sn %= 16;

        // if let Ok(tempo) = tempo.read() {
        // let sleep_time = (*tempo / 60.0) * 2.0 / 16.0;
        // sleep(Duration::from_secs_f64(sleep_time)).await
        // }
        // time sync
        if let Err(e) = sleep_thread.join() {
            error!(
                "joinning sleep thread in midi_out thread resultd in error; {e:?}. this likely means that the sending of sync pulses took longer then the sync step duration"
            );
        }
    }
}

/// sends a message to the message bus every step
pub fn sync_step_notif(data: MbServerHandle, tempo: web::Data<Tempo>, bpq: web::Data<BPQ>) -> ! {
    let conn = uuid::Uuid::new_v4();

    loop {
        // start a sleep thread to sleep for sleep_time.
        let sleep_thread = std::thread::spawn({
            // calculate sleep_time based on BPQ & tempo.
            let (tempo, beats) = (unwrap_rw_lock(&tempo, 99.), unwrap_rw_lock(&bpq, 24.));
            let sleep_time = Duration::from_secs_f64((60.0 / tempo) / 4. / beats);

            move || {
                std::thread::sleep(sleep_time);
            }
        });

        data.send_binary(conn, Vec::new().into());

        // time syncRenameSequenceBody
        if let Err(e) = sleep_thread.join() {
            error!(
                "joinning sleep thread in midi_out thread resultd in error; {e:?}. this likely means that the sending of sync pulses took longer then the sync step duration"
            );
        }
    }
}

pub async fn run(
    tempo: Tempo,
    bpq: BPQ,
    midi_out: MidiOut,
    new_dev_tx: Sender<MidiDev>,
    sequencer_tx: Sender<SequencerControlCmd>,
) -> std::io::Result<()> {
    let tempo = web::Data::new(tempo);
    let bpq = web::Data::new(bpq);
    let midi_out = web::Data::new(midi_out);
    let new_dev_tx = web::Data::new(new_dev_tx);
    let seq_tx = web::Data::new(sequencer_tx);
    let virtual_devs = web::Data::new(Mutex::new(FxHashSet::<String>::default()));
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

    let _clock_notif_jh = std::thread::spawn({
        let tempo = tempo.clone();
        let server_tx = server_tx.clone();

        move || clock_notif(server_tx, tempo)
    });
    let _sync_step_jh = std::thread::spawn({
        let tempo = tempo.clone();
        let bpq = bpq.clone();
        let server_tx = server_tx.clone();

        move || sync_step_notif(server_tx, tempo, bpq)
    });

    HttpServer::new({
        let server_tx = web::Data::new(server_tx);

        move || {
            App::new()
                .wrap(TracingLogger::default())
                .app_data(tempo.clone())
                .app_data(midi_out.clone())
                .app_data(server_tx.clone())
                .app_data(new_dev_tx.clone())
                .app_data(virtual_devs.clone())
                .app_data(seq_tx.clone())
                .service(midi)
                .service(midi_pool_exec)
                .service(get_devs)
                .service(get_tempo)
                .service(set_tempo)
                .service(rest)
                .service(new_dev)
                .service(new_sequence)
                .service(rm_sequence)
                .service(get_sequences)
                .service(get_sequence)
                .service(play_sequence)
                .service(play_these_sequences)
                .service(play_all_sequence)
                .service(pause_sequence)
                .service(pause_all_sequence)
                .service(stop_sequence)
                .service(stop_some_sequence)
                .service(stop_all_sequence)
                .service(add_note)
                .service(rm_note)
                .service(set_dev)
                .service(rename_sequence)
                .service(set_channel)
                .service(change_len_by)
                .service(message_bus::message_bus)
        }
    })
    .worker_max_blocking_threads(1)
    .workers(12)
    // .bind(("127.0.0.1", 8080))?
    // .bind(("localhost", 8080))?
    .bind(("0.0.0.0", 8080))?
    .bind_uds(UDS_SERVER_PATH)?
    .run()
    .await

    // try_join!(http_server, async move { chat_server.await.unwrap() })?;

    // Ok(())
}
