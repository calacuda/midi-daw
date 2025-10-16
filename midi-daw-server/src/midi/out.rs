use crate::{
    midi::MidiDev,
    server::{BPQ, Tempo},
};
use crossbeam::channel::{Receiver, Sender};
use fx_hash::FxHashMap;
use midi_msg::MidiMsg;
use midir::{MidiOutput, os::unix::VirtualOutput};
use std::{
    sync::{Arc, RwLock},
    thread::{sleep, spawn},
    time::Duration,
};
use tracing::log::*;

pub fn unwrap_rw_lock<T>(thing: &Arc<RwLock<T>>, default: T) -> T
where
    T: Copy,
{
    if let Ok(res) = thing.read() {
        *res
    } else {
        default
    }
}

pub fn midi_out(
    midi_msg_out: Receiver<(String, MidiMsg /*, Sender<()>*/)>,
    new_dev: Receiver<MidiDev>,
    tempo: Tempo,
    bpq: BPQ,
    pulse_counter: Arc<RwLock<usize>>,
) -> ! {
    let mut midi_devs = FxHashMap::default();

    loop {
        // start a sleep thread to sleep for sleep_time.
        let sleep_thread = spawn({
            // calculate sleep_time based on BPQ & tempo.
            let (tempo, beats) = (unwrap_rw_lock(&tempo, 99.), unwrap_rw_lock(&bpq, 48.));
            let sleep_time = Duration::from_secs_f64((60.0 / tempo) / beats);

            move || {
                sleep(sleep_time);
            }
        });

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

                    if let Ok(dev) = midi_out.create_virtual(&dev_name)
                        && !midi_devs.contains_key(&dev_name)
                    {
                        midi_devs.insert(dev_name, dev);
                    } else if midi_devs.contains_key(&dev_name) {
                        info!("device already exists")
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
                (dev_name, msg /*, responce_dev*/) if midi_devs.contains_key(&dev_name) => {
                    // send messages
                    let Some(dev) = midi_devs.get_mut(&dev_name) else {
                        error!(
                            "an error occured finding the midi device with the name \"{dev_name}\""
                        );
                        // eprintln!("an error occured finding the midi device with the name \"{dev_name}\"");
                        continue;
                    };

                    if let Err(e) = dev.send(&msg.to_midi()) {
                        error!("midi output failed with error {e}");
                        // eprintln!("midi output failed with error {e}");
                    }

                    // if let Err(e) = responce_dev.send()
                }
                (dev_name, _msg /*, _responce_dev*/) => {
                    error!("the requested midi device, \"{dev_name}\", is not connected.");
                    // eprintln!("the requested midi device, \"{dev_name}\", is not connected.");
                    error!("known devs = {:?}", midi_devs.keys());
                }
            }
        }

        // increment pulse_counter.
        if let Ok(mut counter) = pulse_counter.write() {
            if counter.to_owned() == usize::MAX {
                *counter = 0;
            } else {
                *counter += 1;
            }
        }

        // time sync
        if let Err(e) = sleep_thread.join() {
            error!(
                "joinning sleep thread in midi_out thread resultd in error; {e:?}. this likely means that the processing for midi output took longer then the step duration"
            );
        }
    }
}
