mod midi_utils;

use midi_utils::MidiCC;
use evdev_rs::enums::{EventCode, EventType, EV_ABS};
use evdev_rs::Device;
use evdev_rs::ReadFlag;
use midir::{MidiOutput, MidiOutputConnection};
use midi_types::{MidiMessage, Channel, Control};
use std::fs::File;

fn main() {
    let file = File::open("/dev/input/event27").unwrap();
    let device = Device::new_from_file(file).unwrap();

    let midi_out = MidiOutput::new("My MIDI Output").unwrap();
    let ports = midi_out.ports();
    let out_port = ports.get(0).expect("No MIDI output ports available");
    let port_name = midi_out.port_name(out_port).unwrap_or_else(|_| "Unknown port".to_string());

    let mut conn_out = midi_out.connect(out_port, "midir-test").unwrap();

    loop {
        match device.next_event(ReadFlag::NORMAL) {
            Ok((_, event)) => {
                match event.event_type() {
                    Some(EventType::EV_ABS) => match event.event_code {
                        EventCode::EV_ABS(EV_ABS::ABS_X) => {
                            let midi_value = map_value(event.value);
                            let midi_message = MidiMessage::ControlChange(Channel::C1, MidiCC::Pan.into(), midi_value.into());
                            conn_out.send(&midi_message.into());
                            println!("X Axis moved: {} converted to MIDI {:?} on {}", event.value, midi_message, port_name);
                        }
                        EventCode::EV_ABS(EV_ABS::ABS_Y) => {
                            let midi_value = map_value(event.value);
                            let midi_message = MidiMessage::ControlChange(Channel::C1, MidiCC::Expression.into(), midi_value.into());
                            conn_out.send(&midi_message.into());
                            println!("Y Axis moved: {} converted to MIDI {:?} on {}", event.value, midi_message, port_name);
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            Err(e) => {
                eprintln!("Error reading event: {:?}", e);
            }
        }
    }
}

fn map_value(value: i32) -> u8 {
    (value as f32 / 65535.0 * 127.0) as u8
}
