use std::{
    io::stdin,
    ops::Deref,
    sync::{Arc, RwLock},
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
    // println!("tempo = {:?}", tempo_time);
    tempo_time
        .read()
        .map(|bpm| {
            // info!("{bpm}");
            *bpm.deref()
        })
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

    let tempo = Arc::new(RwLock::new(tempo_from_bpm()));

    let mut time_to_send = 0;
    let mut counter = 0;
    let mut messages: Vec<(u32, Vec<u8>)> = Vec::new();
    debug!("tempo: {:?}", tempo.clone().read());
    debug!("micros: {}", bpq_time(tempo.clone()));
    debug!(
        "time: {}",
        client.time_to_frames(bpq_time(tempo.clone()) as u64)
    );
    debug!("time_to_send: {}", time_to_send);

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
                *tempo = (1_000_000 * 60) / new_tempo;
            }
        }

        let cycle_times = ps.cycle_times().unwrap();

        if time_to_send == 0 {
            // time_to_send = cycle_times.next_usecs;
            time_to_send = c.time_to_frames(c.time());
            debug!("time_to_send: {time_to_send}");
        }
        let mut sender = sync_pulse_sender.writer(ps);

        messages.retain_mut(|(time, _msg)| *time >= ps.n_frames());

        while c.time_to_frames(cycle_times.next_usecs) > time_to_send {
            let last_time_send = time_to_send;
            time_to_send = time_to_send.wrapping_add(bpq_time(tempo.clone()));

            let time = c
                .time_to_frames(time_to_send as u64)
                .wrapping_sub(c.time_to_frames(last_time_send as u64));
            let mut bytes = vec![
                MidiMsg::SystemRealTime {
                    msg: SystemRealTimeMsg::TimingClock,
                }
                .to_midi(),
            ];

            trace!("time: {time}");
            if (counter % (BPQ * 4)) == 0 {
                bytes.push(
                    MidiMsg::SystemRealTime {
                        msg: SystemRealTimeMsg::Start,
                    }
                    .to_midi(),
                );
                debug!("bar");
            }

            counter += 1;

            for bytes in bytes {
                if let Err(e) = sender.write(&jack::RawMidi {
                    time,
                    bytes: &bytes,
                }) {
                    error!("attempt to send sync pulse failed with error: {e}");
                }
            }
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
