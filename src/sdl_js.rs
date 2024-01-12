extern crate sdl2;

use sdl2::event::Event;
use sdl2::joystick::Joystick;
use sdl2::EventPump;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let joystick_subsystem = sdl_context.joystick().unwrap();

    let joysticks = open_virpil_joysticks(&joystick_subsystem);

    // Store joystick names for easy access
    let joystick_names: Vec<String> = joysticks.iter().map(|j| j.name()).collect();

    // Print information about each opened joystick
    for joystick in &joysticks {
        println!("Opened joystick: {}", joystick.name());
        println!("Number of axes: {}", joystick.num_axes());
        println!("Number of buttons: {}", joystick.num_buttons());
        println!("Number of balls: {}", joystick.num_balls());
    }

    let mut event_pump = sdl_context.event_pump().unwrap();
    loop {
        for event in event_pump.poll_iter() {
            let timestamp = current_millis();
            match event {
                Event::JoyAxisMotion { which, axis_idx, value, .. } => {
                    let name = &joystick_names[which as usize];
                    println!("{}: Joystick '{}', Axis {} moved to {}", timestamp, name, axis_idx, value);
                },
                Event::JoyButtonDown { which, button_idx, .. } => {
                    let name = &joystick_names[which as usize];
                    println!("{}: Joystick '{}', Button {} down", timestamp, name, button_idx);
                },
                Event::JoyButtonUp { which, button_idx, .. } => {
                    let name = &joystick_names[which as usize];
                    println!("{}: Joystick '{}', Button {} up", timestamp, name, button_idx);
                },
                Event::JoyHatMotion { which, hat_idx, state, .. } => {
                    let name = &joystick_names[which as usize];
                    println!("{}: Joystick '{}', Hat {} moved to {:?}", timestamp, name, hat_idx, state);
                },
                _ => {}
            }
        }
    }
}

fn open_virpil_joysticks(joystick_subsystem: &sdl2::JoystickSubsystem) -> Vec<Joystick> {
    let mut joysticks = Vec::new();
    for id in 0..joystick_subsystem.num_joysticks().unwrap() {
        if let Ok(name) = joystick_subsystem.name_for_index(id) {
            if name.contains("VIRPIL") {
                if let Ok(joystick) = joystick_subsystem.open(id) {
                    joysticks.push(joystick);
                }
            }
        }
    }
    joysticks
}

fn current_millis() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
}
