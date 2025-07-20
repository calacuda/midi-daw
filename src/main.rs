use crate::midi::midi_out;
use crossbeam::channel::unbounded;

pub mod midi;
pub mod server;

fn main() {
    // prepare mpsc.
    let (midi_msg_out_tx, midi_msg_out_rx) = unbounded();
    // let (midi_msg_in_tx, midi_msg_in_rx) = unbounded();

    {
        let (new_midi_dev_tx, new_midi_dev_rx) = unbounded();

        // start midi output thread.
        std::thread::spawn(move || midi_out(midi_msg_out_rx, new_midi_dev_rx));

        // TODO: start a thread for midi device discovery.
    }

    // TODO: run webserver.
}
