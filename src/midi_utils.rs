use midi_types::Control;

#[derive(Copy, Clone)]
pub enum MidiCC {
    ModulationWheel = 1,
    BreathController = 2,
    FootController = 4,
    PortamentoTime = 5,
    Volume = 7,
    Balance = 8,
    Pan = 10,
    Expression = 11,
    SustainPedal = 64,
    Portamento = 65,
    ReverbLevel = 91,
    ChorusLevel = 93,
    ResetAllControllers = 121,
    // ... add other controls as needed
}

impl Into<Control> for MidiCC {
    fn into(self) -> Control {
        Control::new(self as u8)
    }
}

