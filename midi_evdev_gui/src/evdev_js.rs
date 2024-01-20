use evdev_rs::enums::*;
use evdev_rs::Device;
use std::fs::File;

fn main() {
    let file_path = "/dev/input/event27";
    let file = File::open(file_path).expect("Failed to open device file");
    let device = Device::new_from_file(file).expect("Failed to create device from file");

    loop {
        if let Ok((_, ev)) = device.next_event(evdev_rs::ReadFlag::NORMAL) {
            println!("{:?}", ev);
        }
    }
}