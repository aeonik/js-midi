#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod sdl_js;
mod joystick;

use std::time::{SystemTime, UNIX_EPOCH};
use eframe::egui;
use sdl2::event::Event;
use sdl2::joystick::Joystick;

const MAX_JOYSTICK_VALUE: f32 = 65535.0;
const MIDI_MAX_VALUE: u8 = 127;

struct MyApp {
    name: String,
    age: u32,
    sdl_context: sdl2::Sdl,
    joysticks: Vec<JoystickData>,
    event_pump: sdl2::EventPump,
}

impl Default for MyApp {
    fn default() -> Self {
        let sdl_context = initialize_sdl();
        let joystick_subsystem = sdl_context.joystick().unwrap();
        let joysticks = get_virpil_joysticks(&joystick_subsystem);
        let event_pump = sdl_context.event_pump().unwrap();

        Self {
            name: "World".to_owned(),
            age: 42,
            sdl_context,
            joysticks,
            event_pump,
        }
    }
}

fn initialize_sdl() -> sdl2::Sdl {
    sdl2::init().unwrap() // Consider handling this Result more gracefully
}

fn get_virpil_joysticks(joystick_subsystem: &sdl2::JoystickSubsystem) -> Vec<JoystickData> {
    joystick::open_virpil_joysticks(joystick_subsystem)
        .iter()
        .map(|j| JoystickData::from(j))
        .collect()
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_joystick_events();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Joystick to MIDI Mapper");
            egui::Grid::new("joystick_grid").show(ui, |ui| {
                for joystick in &mut self.joysticks {
                    // Start a new column for each joystick
                    ui.vertical(|ui| {
                        ui.label(format!("Joystick: {}", joystick.name));
                        joystick.axes_states.iter_mut().enumerate().for_each(|(i, axis_state)| {
                            ui.label(format!("Axis {}", i));
                            ui.add(egui::Slider::new(axis_state, -32768.0..=32767.0).text(""));
                        });
                        joystick.buttons_states.iter_mut().enumerate().for_each(|(i, button_state)| {
                            ui.label(format!("Button {}", i));
                            ui.add(egui::Checkbox::new(button_state, ""));
                        });
                    });
                    ui.end_row();
                }
            });
        });
    }
}

struct JoystickData {
    name: String,
    num_axes: u32,
    num_buttons: u32,
    axes_states: Vec<f32>,
    // State of each axis (e.g., position)
    buttons_states: Vec<bool>, // State of each button (pressed or not)
}

impl From<&Joystick> for JoystickData {
    fn from(joystick: &Joystick) -> Self {
        Self {
            name: joystick.name(),
            num_axes: joystick.num_axes(),
            num_buttons: joystick.num_buttons(),
            axes_states: vec![0.0; joystick.num_axes() as usize],
            buttons_states: vec![false; joystick.num_buttons() as usize],
        }
    }
}

impl MyApp {
    fn process_joystick_events(&mut self) {
        for event in self.event_pump.poll_iter() {
            handle_sdl_event(event, &mut self.joysticks);
        }
    }
}

fn handle_sdl_event(event: sdl2::event::Event, joysticks: &mut [JoystickData]) {
    let timestamp = current_millis();
    match event {
        Event::JoyAxisMotion { which, axis_idx, value, .. } => {
            if let Some(joystick) = joysticks.get_mut(which as usize) {
                joystick.axes_states[axis_idx as usize] = value as f32;
                println!("{}: Joystick '{}', Axis {} moved to {}", timestamp, joystick.name, axis_idx, value)
            }
        }
        Event::JoyButtonDown { which, button_idx, .. } => {
            if let Some(joystick) = joysticks.get_mut(which as usize) {
                joystick.buttons_states[button_idx as usize] = true;
                println!("{}: Joystick '{}', Button {} down", timestamp, joystick.name, button_idx)
            }
        }
        Event::JoyButtonUp { which, button_idx, .. } => {
            if let Some(joystick) = joysticks.get_mut(which as usize) {
                joystick.buttons_states[button_idx as usize] = false;
                println!("{}: Joystick '{}', Button {} up", timestamp, joystick.name, button_idx)
            }
        }
        Event::JoyHatMotion { which, hat_idx, state, .. } => {
            // Handle hat motion if needed
        }
        _ => {}
    }
}

fn current_millis() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Box::new(MyApp::default())
        }),
    )
}
