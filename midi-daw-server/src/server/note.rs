use crate::server::MidiOut;
use actix::clock::sleep;
use actix_web::web::{self};
use midi_daw_types::{MidiChannel, NoteDuration};
use midi_msg::ControlChange;
use std::time::Duration;
use tracing::log::*;

pub async fn rest(tempo: f64, dur: NoteDuration) {
    let (mul, denom) = match dur {
        NoteDuration::Wn(n) => (n, 1.0),
        NoteDuration::Hn(n) => (n, 2.0),
        NoteDuration::Qn(n) => (n, 4.0),
        NoteDuration::En(n) => (n, 8.0),
        NoteDuration::Sn(n) => (n, 16.0),
        NoteDuration::Tn(n) => (n, 32.0),
        NoteDuration::S4n(n) => (n, 64.0),
    };
    let mul = mul as f64;

    sleep(Duration::from_secs_f64(
        ((60.0 / tempo) * 2.0 / denom) * mul,
    ))
    .await;
}

pub async fn play_note(
    tempo: f64,
    midi_out: web::Data<MidiOut>,
    dev: String,
    channel: MidiChannel,
    note: u8,
    velocity: u8,
    dur: NoteDuration,
) {
    debug!("playing note {note} on {dev} with velocity {velocity}");

    let msg = midi_msg::MidiMsg::ChannelVoice {
        channel: channel.into(),
        msg: midi_msg::ChannelVoiceMsg::NoteOn { note, velocity },
    };

    _ = midi_out.send((dev.clone(), msg));

    rest(tempo, dur).await;

    let msg = midi_msg::MidiMsg::ChannelVoice {
        channel: channel.into(),
        msg: midi_msg::ChannelVoiceMsg::NoteOff { note, velocity },
    };

    _ = midi_out.send((dev, msg));
}

pub async fn stop_note(midi_out: web::Data<MidiOut>, dev: String, channel: MidiChannel, note: u8) {
    let msg = midi_msg::MidiMsg::ChannelVoice {
        channel: channel.into(),
        msg: midi_msg::ChannelVoiceMsg::NoteOff {
            note,
            velocity: 127,
        },
    };

    _ = midi_out.send((dev, msg));
}

pub async fn send_cc(
    midi_out: web::Data<MidiOut>,
    dev: String,
    channel: MidiChannel,
    cc: u8,
    value: u8,
) {
    let msg = midi_msg::MidiMsg::ChannelVoice {
        channel: channel.into(),
        msg: midi_msg::ChannelVoiceMsg::ControlChange {
            control: ControlChange::CC { control: cc, value },
        },
    };

    _ = midi_out.send((dev, msg));
}

pub async fn pitch_bend(
    midi_out: web::Data<MidiOut>,
    dev: String,
    channel: MidiChannel,
    bend: u16,
) {
    let msg = midi_msg::MidiMsg::ChannelVoice {
        channel: channel.into(),
        msg: midi_msg::ChannelVoiceMsg::PitchBend { bend },
    };

    _ = midi_out.send((dev, msg));
}
