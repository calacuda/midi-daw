use crate::midi::MidiDev;
use crossbeam::channel::Sender;
use fx_hash::{FxHashMap, FxHashSet};
use midir::MidiOutput;
use std::{thread::sleep, time::Duration};

pub fn new_midi_dev(new_dev_tx: Sender<MidiDev>) -> ! {
    let mut midi_devs: FxHashMap<String, String> = FxHashMap::default();

    loop {
        // check for new devices
        let midi_out = MidiOutput::new("MIDI-DAW").unwrap();
        let midi_devs_names: FxHashSet<(String, String)> = midi_out
            .ports()
            .into_iter()
            .filter_map(|port| {
                midi_out
                    .port_name(&port)
                    .map_or(None, |name| Some((name, port.id())))
            })
            .collect();

        // send new devices
        for (dev_name, dev_id) in midi_devs_names.iter() {
            if !midi_devs.contains_key(dev_name) {
                _ = new_dev_tx.send(MidiDev::Added {
                    dev_name: dev_name.clone(),
                    dev_id: dev_id.clone(),
                });
            }
        }

        // send rmed devs
        for (dev_name, dev_id) in midi_devs.iter() {
            if !midi_devs_names.contains(&(dev_name.clone(), dev_id.clone())) {
                _ = new_dev_tx.send(MidiDev::RMed(dev_name.clone()));
            }
        }

        midi_devs = midi_devs_names.into_iter().collect();

        sleep(Duration::from_millis(100));
    }
}
