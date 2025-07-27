use crossbeam::channel::{Receiver, Sender};
use fx_hash::FxHashMap;
use log::error;
use midi_msg::MidiMsg;
use midir::MidiOutput;

pub enum MidiDev {
    Added { dev_name: String, dev_id: String },
    RMed(String),
}

pub fn midi_out(
    midi_msg_out: Receiver<(String, MidiMsg)>,
    // midi_msg_in: Sender<(String, MidiMsg)>,
    new_dev: Receiver<MidiDev>,
) {
    let mut midi_devs = FxHashMap::default();

    loop {
        // poll for new midi devices
        match new_dev.try_recv() {
            Ok(MidiDev::Added { dev_name, dev_id }) => {
                let midi_out = MidiOutput::new(&format!("MIDI-DAW-{dev_name}")).unwrap();

                if let Some(dev) = midi_out.find_port_by_id(dev_id).clone() {
                    if let Ok(dev) = midi_out.connect(&dev, &dev_name) {
                        midi_devs.insert(dev_name, dev);
                    }
                }
            }
            Ok(MidiDev::RMed(dev_name)) => {
                midi_devs.remove(&dev_name);
            }
            Err(_) => {}
        }

        // poll for msg to send
        match midi_msg_out.try_recv() {
            Ok((dev_name, msg)) if midi_devs.contains_key(&dev_name) => {
                // send messages
                let Some(dev) = midi_devs.get_mut(&dev_name) else {
                    continue;
                };

                _ = dev.send(&msg.to_midi());
            }
            Ok((dev_name, _msg)) => {
                error!("the requested midi device, \"{dev_name}\", is not connected.")
            }
            Err(_) => {}
        }
    }
}
