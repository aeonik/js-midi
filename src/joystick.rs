use sdl2::joystick::Joystick;

struct JoystickData {
    name: String,
    num_axes: u32,
    num_buttons: u32,
    axes_states: Vec<f32>, // State of each axis (e.g., position)
    buttons_states: Vec<bool>, // State of each button (pressed or not)
}

impl JoystickData {
    fn new(joystick: &Joystick) -> Self {
        Self {
            name: joystick.name(),
            num_axes: joystick.num_axes(),
            num_buttons: joystick.num_buttons(),
            axes_states: vec![0.0; joystick.num_axes() as usize],
            buttons_states: vec![false; joystick.num_buttons() as usize],
        }
    }
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

