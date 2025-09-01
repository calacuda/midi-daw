use bevy::prelude::*;
use crossbeam::channel::Sender;
use midi_daw::midi::MidiDev;
use midi_daw_types::MidiDeviceName;
use midi_msg::MidiMsg;

#[derive(Clone, DerefMut, Deref, Resource)]
pub struct NewMidiDev(pub Sender<MidiDev>);

impl NewMidiDev {
    pub fn create_new(&mut self, dev_name: MidiDeviceName) {
        if let Err(e) = self.0.send(MidiDev::CreateVirtual(dev_name.clone())) {
            error!("failed to send message to create virtual dev {dev_name}. {e}");
        }
    }
}

#[derive(Clone, DerefMut, Deref, Resource)]
pub struct MidiOutput(pub Sender<(String, MidiMsg)>);

impl MidiOutput {
    pub fn send(&mut self, dev: MidiDeviceName, msg: MidiMsg) {
        if let Err(e) = self.0.send((dev.clone(), msg)) {
            error!("failed to send message to device {dev}. {e}");
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn it_works() {
//         // let result = add(2, 2);
//         // assert_eq!(result, 4);
//     }
// }
