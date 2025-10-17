use bevy::prelude::*;
use fx_hash::FxHashMap;
use std::time::Instant;

use crate::MainState;

/// maps buttons to when time: Res<Time>,they were pressed
#[derive(Clone, Default, Debug, PartialEq, Eq, Resource)]
pub struct ButtonPressTimers(pub FxHashMap<GamepadButton, (Instant, bool)>);

pub struct ButtonTrackerPlugin;

impl Plugin for ButtonTrackerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ButtonPressTimers>().add_systems(
            Update,
            (
                arrow_keys_timer(GamepadButton::DPadLeft),
                arrow_keys_timer(GamepadButton::DPadRight),
                arrow_keys_timer(GamepadButton::DPadUp),
                arrow_keys_timer(GamepadButton::DPadDown),
            )
                .run_if(not(in_state(MainState::StartUp))),
        );
    }
}

fn arrow_keys_timer(button: GamepadButton) -> impl Fn(Query<&Gamepad>, ResMut<ButtonPressTimers>) {
    // if inputs.just_pressed(GamepadButton::DPadLeft) {}
    // |// mut commands: Commands,
    //  inputs: Res<ButtonInput<GamepadButton>>,
    //  mut timers: ResMut<ButtonPressTimers>,
    //  time: Res<Time>| {
    //     do_arrow_keys_timer(button, inputs, timers, time);
    // }
    fn do_arrow_keys_timer(
        button: GamepadButton,
        // mut commands: Commands,
        inputs: Query<&Gamepad>,
        mut timers: ResMut<ButtonPressTimers>,
        // time: Res<Time>,
    ) {
        for input in inputs {
            if input.just_pressed(button) {
                timers.0.insert(button, (Instant::now(), true));
            } else if input.just_released(button) {
                _ = timers.0.remove(&button);
            } else if let Some((_, just_pressed)) = timers.0.get_mut(&button) {
                *just_pressed = false;
            }
        }
    }

    move |inputs, timers| do_arrow_keys_timer(button, inputs, timers)
}
