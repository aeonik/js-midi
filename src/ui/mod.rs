use crate::joystick::JoystickData;
use egui::{CtxRef, Window};

pub fn render(ctx: &CtxRef, joystick_data: &[JoystickData]) {
    egui::CentralPanel::default().show(ctx, |ui| {
        for joystick in joystick_data {
            Window::new(&joystick.name).show(ui.ctx(), |ui| {
                ui.label(format!("Number of axes: {}", joystick.num_axes));
                ui.label(format!("Number of buttons: {}", joystick.num_buttons));
                // ... other UI elements
            });
        }
    });
}
