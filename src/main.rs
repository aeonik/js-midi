use evdev_rs::enums::{EventCode, EventType, EV_ABS};
use evdev_rs::Device;
use evdev_rs::ReadFlag;
use std::fs::File;

fn main() {
    let file = File::open("/dev/input/event27").unwrap();
    let device = Device::new_from_file(file).unwrap();

    loop {
        match device.next_event(ReadFlag::NORMAL) {
            Ok((_, event)) => {
                match event.event_type() {
                    Some(EventType::EV_ABS) => match event.event_code {
                        EventCode::EV_ABS(EV_ABS::ABS_X) => {
                            // Process X axis
                            println!("X Axis moved: {}", event.value);
                        }
                        EventCode::EV_ABS(EV_ABS::ABS_Y) => {
                            // Process Y axis
                            println!("Y Axis moved: {}", event.value);
                        }
                        _ => {}
                    },
                    // Handle other event types
                    _ => {}
                }
            }
            Err(e) => {
                eprintln!("Error reading event: {:?}", e);
            }
        }

        // Convert and send MIDI/OSC message
    }
}
