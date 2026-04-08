#![feature(thread_sleep_until)]
use std::{
    io::stdin,
    ops::Deref,
    sync::{Arc, RwLock},
    thread::{sleep_until, spawn},
    time::{Duration, Instant},
};

use midi_daw::{
    tempo_from_bpm,
    v2::{BPQ, DEFAULT_BPM, SYNC_DEV_NAME, SYNC_DEV_PORT_NAME, TEMPO_SET_PORT},
};
use midi_msg::{Meta, MidiMsg, SystemRealTimeMsg};
use tracing::*;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

// Returns the time of a single pulse in microseconds
fn bpq_time(tempo_time: Arc<RwLock<u32>>) -> u32 {
    tempo_time
        .read()
        .map(|bpm| *bpm.deref())
        .unwrap_or(DEFAULT_BPM)
        / BPQ
}

fn main() {
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

    let (client, _status) = loop {
        if let Ok(midi_out) = jack::Client::new(SYNC_DEV_NAME, jack::ClientOptions::default()) {
            break midi_out;
        }
    };
    let mut sync_pulse_sender = client
        .register_port(SYNC_DEV_PORT_NAME, jack::MidiOut::default())
        .expect("failed to register sync pulse sender");
    let in_port = client
        .register_port(TEMPO_SET_PORT, jack::MidiIn::default())
        .expect("failed to register tempo setter");

    let tempo = Arc::new(RwLock::new(tempo_from_bpm(DEFAULT_BPM)));

    let (mut tx, recv) = spmc::channel();
    let _jh = spawn({
        let tempo = tempo.clone();

        move || {
            loop {
                let now = Instant::now();
                _ = tx.send(());

                sleep_until(now + Duration::from_micros(bpq_time(tempo.clone()).into()))
            }
        }
    });
    // let mut waiting = false;
    let mut time_to_send = 0;

    let callback = move |c: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
        let show_p = in_port.iter(ps);

        for raw_mesg in show_p {
            if let (
                Ok((
                    MidiMsg::Meta {
                        msg: Meta::SetTempo(new_tempo),
                    },
                    _,
                )),
                Ok(mut tempo),
            ) = (MidiMsg::from_midi(raw_mesg.bytes), tempo.write())
            {
                *tempo = new_tempo;
            }
        }

        // time_ellapsed += .unwrap();

        let cycle_times = ps.cycle_times().unwrap();

        if time_to_send == 0 {
            time_to_send = cycle_times.next_usecs;
        }

        while cycle_times.next_usecs > time_to_send {
            let next_time_to_send = time_to_send
                + Duration::from_micros(bpq_time(tempo.clone()).into()).as_micros() as u64;

            // println!("next_time_to_send: {next_time_to_send}");
            // println!(
            //     "{} - {}",
            //     c.time_to_frames(time_to_send),
            //     cycle_times.current_frames // time_to_send,
            //                                // cycle_times.current_usecs
            // );

            // let time = c.time_to_frames(time_to_send) - cycle_times.current_frames;
            // let time = c.time_to_frames(time_to_send - cycle_times.current_usecs);
            let time = c.time_to_frames(time_to_send) - cycle_times.current_frames;
            time_to_send = next_time_to_send;

            // if recv.try_recv().is_ok() {
            let mut sender = sync_pulse_sender.writer(ps);
            let bytes = MidiMsg::SystemRealTime {
                msg: SystemRealTimeMsg::TimingClock,
            }
            .to_midi();

            // info!("time: {time}");

            if let Err(e) = sender.write(&jack::RawMidi {
                time,
                bytes: &bytes,
            }) {
                error!("attempt to send sync pulse failed with error: {e}");
            } else {
                // info!("beat");
            }
            // }
            // waiting = true;
        }

        jack::Control::Continue
    };

    // Activate
    let active_client = client
        .activate_async((), jack::contrib::ClosureProcessHandler::new(callback))
        .unwrap();

    // Wait
    println!("Press any key to quit");
    let mut user_input = String::new();
    stdin().read_line(&mut user_input).ok();

    // Optional deactivation.
    if let Err(err) = active_client.deactivate() {
        eprintln!("JACK exited with error: {err}");
    };
}
