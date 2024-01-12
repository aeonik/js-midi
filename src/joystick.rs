use sdl2::joystick::Joystick;

pub struct JoystickData {
    pub name: String,
    pub num_axes: u32,
    pub num_buttons: u32,

}

pub fn open_virpil_joysticks(joystick_subsystem: &sdl2::JoystickSubsystem) -> Vec<Joystick> {
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

