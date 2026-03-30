use std::sync::{Arc, RwLock};

use actix::dev::OneshotSender;
use crossbeam::channel::Receiver;
use jack::contrib::controller::ControlledProcessorTrait;
use midi_daw_types::{
    AutomationCommand,
    automation::{
        AUTOMATIONS_PER_SECOND, Automation, AutomationConf, AutomationTrait, AutomationTypes,
    },
};
use midi_msg::MidiMsg;
use serde::{Deserialize, Serialize};
// use tinyaudio::{OutputDeviceParameters, run_output_device};
use tracing::log::*;

// pub fn automation(
//     new_automation_rx: Receiver<AutomationConf>,
//     // midi_msg_out: Sender<(String, MidiMsg)>,
// ) -> ! {
//     let mut automations = Vec::default();
//
//     loop {
//         if let Ok(automation_conf) = new_automation_rx.recv() {
//             match AutomationTypes::try_from(automation_conf) {
//                 Ok(automation) => {
//                     let am = Arc::new(RwLock::new(automation));
//
//                     if let Ok(device) = run_output_device(
//                         OutputDeviceParameters {
//                             channels_count: 1,
//                             sample_rate: AUTOMATIONS_PER_SECOND as usize,
//                             channel_sample_count: 64,
//                         },
//                         {
//                             let am = am.clone();
//
//                             move |data| {
//                                 // Output silence
//                                 if let Ok(mut am) = am.write() {
//                                     for sample in data {
//                                         *sample = am.step() as f32;
//                                     }
//                                 } else {
//                                     // 0.0
//                                     for sample in data {
//                                         *sample = 0.0;
//                                     }
//                                 }
//                             }
//                         },
//                     ) {
//                         automations.push((am, device))
//                     } else {
//                         error!("failed to start automation");
//                     }
//                 }
//                 Err(e) => {
//                     error!("generating automation failed with error \"{e}\"");
//                 }
//             }
//         }
//
//         automations
//             .retain(|(autom, _deviec)| !autom.read().map(|autom| autom.done()).unwrap_or(false));
//     }
// }

pub fn automation(
    new_automation_rx: Receiver<AutomationCommand>,
    // midi_msg_out: Sender<(String, MidiMsg)>,
) -> ! {
    let mut automations = Vec::default();

    loop {
        if let Ok(automation_cmd) = new_automation_rx.recv() {
            match automation_cmd {
                AutomationCommand::New {
                    conf,
                    name,
                    // responder,
                } => match Automation::new(conf, &name) {
                    Ok((automation, jack_client)) => {
                        // let am = Arc::new(RwLock::new(automation));

                        // if let Ok(device) = run_output_device(
                        //     OutputDeviceParameters {
                        //         channels_count: 1,
                        //         sample_rate: AUTOMATIONS_PER_SECOND as usize,
                        //         channel_sample_count: 64,
                        //     },
                        //     {
                        //         let am = am.clone();
                        //
                        //         move |data| {
                        //             // Output silence
                        //             if let Ok(mut am) = am.write() {
                        //                 for sample in data {
                        //                     *sample = am.step() as f32;
                        //                 }
                        //             } else {
                        //                 // 0.0
                        //                 for sample in data {
                        //                     *sample = 0.0;
                        //                 }
                        //             }
                        //         }
                        //     },
                        // ) {
                        //     automations.push((am, device))
                        // } else {
                        //     error!("failed to start automation");
                        // }

                        let (processor_instance, handle) = automation.instance(16, 16);

                        let active_client =
                            jack_client.activate_async((), processor_instance).unwrap();

                        automations.push((name, active_client, handle));
                    }
                    Err(e) => {
                        error!("generating automation failed with error \"{e}\"");
                    }
                },
                AutomationCommand::Stop { name } => {
                    // automations.(
                    let mut new_automations = Vec::with_capacity(automations.len() - 1);

                    for (this_name, jack_client, _handle) in automations {
                        if this_name == name {
                            _ = jack_client.deactivate();
                        } else {
                            new_automations.push((this_name, jack_client, _handle));
                        }
                        // this_name != &name
                    }

                    automations = new_automations;
                }
            }
        }

        // automations.retain(|(name, autom, jack_client, _)| {
        //     let done = autom.read().map(|autom| autom.done()).unwrap_or(false);
        //
        //     if done {
        //         jack_client.deactivate();
        //     }
        //
        //     !done
        // });

        let mut new_automations = Vec::with_capacity(automations.len() - 1);

        for (_this_name, jack_client, mut handle) in automations {
            if !handle.drain_notifications().collect::<Vec<_>>().is_empty() {
                _ = jack_client.deactivate();
            } else {
                new_automations.push((_this_name, jack_client, handle));
            }
            // this_name != &name
        }

        automations = new_automations;
    }
}
