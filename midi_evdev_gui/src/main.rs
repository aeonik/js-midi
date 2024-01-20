mod midi_utils;
mod sdl_js;
mod evdev_js;

use evdev_rs::enums::EV_KEY;
use evdev_rs::InputEvent;
use midi_utils::MidiCC;
use evdev_rs::enums::{EventCode, EventType, EV_ABS};
use evdev_rs::Device;
use evdev_rs::ReadFlag;
use midir::{MidiOutput, MidiOutputConnection};
use midi_types::{MidiMessage, Channel, Control, Value7, Note};
use std::fs::File;
use midi_convert::render_slice::MidiRenderSlice;
use midi_types::status::{NOTE_OFF, NOTE_ON};



const MAX_JOYSTICK_VALUE: f32 = 65535.0;
const MIDI_MAX_VALUE: u8 = 127;

fn main() {
    let file = File::open("/dev/input/event27").expect("Failed to open device file");
    let device = Device::new_from_file(file).expect("Failed to create device from file");

    let midi_out = MidiOutput::new("My MIDI Output").expect("Failed to create MIDI output");
    let ports = midi_out.ports();
    let out_port = ports.get(0).expect("No MIDI output ports available");
    let port_name = midi_out.port_name(out_port).unwrap_or_else(|_| "Unknown port".to_string());

    let mut conn_out = midi_out.connect(out_port, "midir-test").expect("Failed to connect MIDI output");

    loop {
        match device.next_event(ReadFlag::NORMAL) {
            Ok((_, event)) => {
                match event.event_type() {
                    Some(EventType::EV_ABS) => process_abs_event(event, &mut conn_out, &port_name),
                    Some(EventType::EV_KEY) => process_key_event(event, &mut conn_out, &port_name),
                    _ => {}
                }
            }
            Err(e) => {
                eprintln!("Error reading event: {:?}", e);
            }
        }
    }
}


fn process_abs_event(event: evdev_rs::InputEvent, conn_out: &mut MidiOutputConnection, port_name: &str) {
    let (control, value) = match event.event_code {
        EventCode::EV_ABS(EV_ABS::ABS_X) => (MidiCC::Pan, map_value(event.value)),
        EventCode::EV_ABS(EV_ABS::ABS_Y) => (MidiCC::Volume, map_value(event.value)),
        EventCode::EV_ABS(EV_ABS::ABS_RX) => (MidiCC::ModulationWheel, map_value(event.value)),
        EventCode::EV_ABS(EV_ABS::ABS_RY) => (MidiCC::Expression, map_value(event.value)),

        _ => return,
    };

    let msg = MidiMessage::ControlChange(
        Channel::new(0),
        Control::new(control as u8),
        Value7::new(value),
    );
    let mut buf = [0; 3];
    let len = msg.render_slice(&mut buf);
    conn_out.send(&buf[..len]).expect("Failed to send MIDI message");
    println!("Axis moved: {} converted to MIDI {:?} on {}", event.value, value, port_name);
}

fn process_key_event(event: InputEvent, conn_out: &mut MidiOutputConnection, port_name: &str) {
    let note = Note::new(60);

    if let EventCode::EV_KEY(key_code) = event.event_code {
        // Check if the key code matches the desired button (e.g., BTN_BASE6 or code 300)
        if key_code == EV_KEY::BTN_BASE6 || key_code == EV_KEY::KEY_UNKNOWN {  // KEY_UNKNOWN as placeholder for custom key code
            let velocity = if event.value == 1 { Value7::new(127) } else { Value7::new(0) }; // Velocity 127 for note on, 0 for note off

            let msg = if event.value == 1 {
                MidiMessage::NoteOn(Channel::new(0), note, velocity)
            } else {
                MidiMessage::NoteOff(Channel::new(0), note, velocity)
            };

            let mut buf = [0; 3];
            let len = msg.render_slice(&mut buf);
            conn_out.send(&buf[..len]).expect("Failed to send MIDI message");
            println!("Button {:?}: {} converted to MIDI Note {:?} on {}", key_code, event.value, note, port_name);
        }
    }
}
fn map_value(value: i32) -> u8 {
    (value as f32 / MAX_JOYSTICK_VALUE * MIDI_MAX_VALUE as f32) as u8
}
