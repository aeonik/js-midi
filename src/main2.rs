#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod sdl_js;
mod joystick;

use std::time::{SystemTime, UNIX_EPOCH};
use eframe::egui;
use sdl2::event::Event;
use sdl2::joystick::Joystick;


struct MyApp {
    name: String,
    age: u32,
    sdl_context: sdl2::Sdl,
    joysticks: Vec<JoystickData>,
    event_pump: sdl2::EventPump,
}

impl Default for MyApp {
    fn default() -> Self {
        let sdl_context = sdl2::init().unwrap();
        let joystick_subsystem = sdl_context.joystick().unwrap();
        let joysticks = joystick::open_virpil_joysticks(&joystick_subsystem)
            .iter()
            .map(|j| JoystickData {
                name: j.name(),
                num_axes: j.num_axes(),
                num_buttons: j.num_buttons(),
            })
            .collect();
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

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_joystick_events();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Joystick to MIDI Mapper");

            for joystick in &self.joysticks {
                ui.label(format!("Joystick: {}", joystick.name));
                ui.label(format!("Number of axes: {}", joystick.num_axes));
                ui.label(format!("Number of buttons: {}", joystick.num_buttons));
            }
        });
    }
}

struct JoystickData {
    name: String,
    num_axes: u32,
    num_buttons: u32,
}

impl MyApp {
    fn process_joystick_events(&mut self) {
        for event in self.event_pump.poll_iter() {
            let timestamp = current_millis();
            match event {
                Event::JoyAxisMotion { which, axis_idx, value, .. } => {
                    let name = &self.joysticks[which as usize].name;
                    println!("{}: Joystick '{}', Axis {} moved to {}", timestamp, name, axis_idx, value);
                },
                Event::JoyButtonDown { which, button_idx, .. } => {
                    let name = &self.joysticks[which as usize].name;
                    println!("{}: Joystick '{}', Button {} down", timestamp, name, button_idx);
                },
                Event::JoyButtonUp { which, button_idx, .. } => {
                    let name = &self.joysticks[which as usize].name;
                    println!("{}: Joystick '{}', Button {} up", timestamp, name, button_idx);
                },
                Event::JoyHatMotion { which, hat_idx, state, .. } => {
                    let name = &self.joysticks[which as usize].name;
                    println!("{}: Joystick '{}', Hat {} moved to {:?}", timestamp, name, hat_idx, state);
                },
                _ => {}
            }
        }
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
