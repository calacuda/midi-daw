pub mod automation;
pub mod dev;
pub mod out;

pub enum MidiDev {
    Added { dev_name: String, dev_id: String },
    RMed(String),
    CreateVirtual(String),
}
