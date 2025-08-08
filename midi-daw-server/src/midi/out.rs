use crate::midi::MidiDev;
use crossbeam::channel::Receiver;
use fx_hash::FxHashMap;
use midi_msg::MidiMsg;
use midir::{os::unix::VirtualOutput, MidiOutput};
use tracing::log::*;

pub fn midi_out(midi_msg_out: Receiver<(String, MidiMsg)>, new_dev: Receiver<MidiDev>) -> ! {
    let mut midi_devs = FxHashMap::default();

    loop {
        // poll for new midi devices
        while let Ok(dev_msg) = new_dev.try_recv() {
            match dev_msg {
                MidiDev::Added { dev_name, dev_id } => {
                    let midi_out = MidiOutput::new(&format!("MIDI-DAW-{dev_name}")).unwrap();

                    if let Some(dev) = midi_out.find_port_by_id(dev_id.to_string()).clone() {
                        if let Ok(dev) = midi_out.connect(&dev, &dev_name) {
                            midi_devs.insert(dev_name, dev);
                        } else {
                            warn!("device named \"{dev_name}\" is no longer connected")
                        }
                    } else {
                        warn!("unknown device id \"{dev_id}\"")
                    }
                }
                MidiDev::RMed(dev_name) => {
                    midi_devs.remove(&dev_name);
                }
                MidiDev::CreateVirtual(dev_name) => {
                    let midi_out = MidiOutput::new("MIDI-DAW-NEW-DEV").unwrap();

                    if let Ok(dev) = midi_out.create_virtual(&dev_name) {
                        midi_devs.insert(dev_name, dev);
                    } else {
                        error!("failed to make virtual output device");
                        // eprintln!("failed to make virtual output device");
                    }
                }
            }
        }

        // poll for msg to send
        while let Ok(midi_msg) = midi_msg_out.try_recv() {
            match midi_msg {
                (dev_name, msg) if midi_devs.contains_key(&dev_name) => {
                    // send messages
                    let Some(dev) = midi_devs.get_mut(&dev_name) else {
                        error!("an error occured finding the midi device with the name \"{dev_name}\"");
                        // eprintln!("an error occured finding the midi device with the name \"{dev_name}\"");
                        continue;
                    };

                    if let Err(e) = dev.send(&msg.to_midi()) {
                        error!("midi output failed with error {e}");
                        // eprintln!("midi output failed with error {e}");
                    }
                }
                (dev_name, _msg) => {
                    error!("the requested midi device, \"{dev_name}\", is not connected.");
                    // eprintln!("the requested midi device, \"{dev_name}\", is not connected.");
                    error!("known devs = {:?}", midi_devs.keys());
                }
            }
        }
    }
}
