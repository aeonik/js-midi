#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::{App, CreationContext, egui};
use sdl2::event::Event;
use sdl2::joystick::Joystick;
use sdl2::JoystickSubsystem;
use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort};
use midi_types::{MidiMessage, Channel, Control, Value7, Note};
use std::fs::File;
use midi_convert::render_slice::MidiRenderSlice;
use egui_graphs::{DefaultEdgeShape, DefaultNodeShape, Graph, GraphView};
use petgraph::Directed;
use petgraph::stable_graph::{StableGraph};


const MAX_JOYSTICK_VALUE: i32 = 32767;

struct JoystickState {
    joystick: Joystick,
    axes_states: Vec<f32>,
    buttons_states: Vec<bool>,
    // Add other state fields as necessary
}

impl JoystickState {
    fn new(joystick: Joystick) -> Self {
        let num_axes = joystick.num_axes() as usize;
        let num_buttons = joystick.num_buttons() as usize;

        Self {
            joystick,
            axes_states: vec![0.0; num_axes],
            buttons_states: vec![false; num_buttons],
        }
    }

    fn update_axis(&mut self, axis_idx: u8, value: i16) {
        if let Some(state) = self.axes_states.get_mut(axis_idx as usize) {
            *state = value as f32; // Convert or scale value as needed
        }
    }

    // TODO Add more update methods for buttons, hats, etc.
}

enum JoystickNode {
    Axis(AxisNode),
    Button(ButtonNode),
}

struct AxisNode {
    axis_index: usize,
    value: f32,
}

struct ButtonNode {
    button_index: usize,
    state: bool,
}

impl AxisNode {
    fn new(axis_index: usize, value: f32) -> Self {
        Self { axis_index, value }
    }
}

impl ButtonNode {
    fn new(button_index: usize, state: bool) -> Self {
        Self { button_index, state }
    }
}


struct MyApp {
    sdl_context: sdl2::Sdl,
    joysticks: Vec<JoystickState>,
    event_pump: sdl2::EventPump,
    midi_out: MidiOutput,
    ports: Vec<midir::MidiOutputPort>,
    out_port: MidiOutputPort,
    port_name: String,
    connection_graph: Graph<(), (), Directed>,
}

impl Default for MyApp {
    fn default() -> Self {
        let sdl_context = sdl2::init().unwrap();
        let joystick_subsystem = sdl_context.joystick().unwrap();
        let joysticks = open_virpil_joysticks(&joystick_subsystem);
        let event_pump = sdl_context.event_pump().unwrap();

        let midi_out = MidiOutput::new("My MIDI Output").expect("Failed to create MIDI output");
        let ports = midi_out.ports();
        let out_port = ports.get(0).expect("No MIDI output ports available").clone();
        let port_name = midi_out.port_name(&out_port).unwrap_or_else(|_| "Unknown port".to_string());
        let connection_graph = generate_graph(&joysticks);

        Self {
            sdl_context,
            joysticks,
            event_pump,
            midi_out,
            ports,
            out_port,
            port_name,
            connection_graph: Graph::from(&connection_graph),
        }
    }
}



fn generate_graph(joysticks: &[JoystickState]) -> StableGraph<JoystickNode, ()> {
    let mut g = StableGraph::new();

    for joystick in joysticks {
        let nodes = joystick_to_nodes(joystick);
        let mut prev_node_index = None;

        for node in nodes {
            let node_index = g

                .add_node(node); // Add the joystick node to the graph
            if let Some(prev_index) = prev_node_index {
// Connect each node to the previous node
                g.add_edge(prev_index, node_index, ());
            }
            prev_node_index = Some(node_index);
        }
    }

    g
}

fn joystick_to_nodes(joystick: &JoystickState) -> Vec<JoystickNode> {
    let mut nodes = Vec::new();
    for (axis_idx, &state) in joystick.axes_states.iter().enumerate() {
        nodes.push(JoystickNode::Axis(AxisNode { axis_index: axis_idx, value: state }));
    }
    for (button_idx, &state) in joystick.buttons_states.iter().enumerate() {
        nodes.push(JoystickNode::Button(ButtonNode { button_index: button_idx, state }));
    }
    nodes
}



fn open_virpil_joysticks(p0: &JoystickSubsystem) -> Vec<JoystickState> {
    let mut joystick_states = Vec::new();
    for id in 0..p0.num_joysticks().unwrap() {
        if let Ok(name) = p0.name_for_index(id) {
            if name.contains("VIRPIL") {
                if let Ok(joystick) = p0.open(id) {
                    joystick_states.push(JoystickState::new(joystick));
                }
            }
        }
    }
    joystick_states
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::JoyAxisMotion { which, axis_idx, value, .. } => {
                    // Update joystick state
                    let joystick = &mut self.joysticks[which as usize];
                    joystick.update_axis(axis_idx, value);

                    // Request repaint on each detected axis motion
                    //ctx.request_repaint();
                },
                // Add other joystick event cases here if needed
                Event::JoyButtonUp { which, button_idx, .. } => {
                    let joystick = &mut self.joysticks[which as usize];
                    joystick.buttons_states[button_idx as usize] = false;
                },

                Event::JoyButtonDown { which, button_idx, .. } => {
                    let joystick = &mut self.joysticks[which as usize];
                    joystick.buttons_states[button_idx as usize] = true;
                },


                _ => {}
            }
        }

        // Redraw UI every frame
        ctx.request_repaint();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Joystick to MIDI Mapper");
            // Add the graph
            ui.add(&mut GraphView::<
                _,
                _,
                _,
                _,
                DefaultNodeShape,
                DefaultEdgeShape,
            >::new(&mut self.connection_graph));

            ui.vertical(|ui| {
                for joystick in &self.joysticks {
                    ui.horizontal(|ui| {
                        egui::Window::new(&joystick.joystick.name()).show(ui.ctx(), |ui| {
                            ui.label(format!("Number of axes: {}", joystick.joystick.num_axes()));
                            ui.label(format!("Number of buttons: {}", joystick.joystick.num_buttons()));

                            // Slider for each axis
                            ui.vertical(|ui| {
                                for (axis_idx, state) in joystick.axes_states.iter().enumerate() {
                                    ui.add(egui::Slider::new(&mut state.clone(), 0.0..=MAX_JOYSTICK_VALUE as f32).text(format!("Axis {}", axis_idx)));
                                }
                            });

                            egui::Grid::new("button_grid").striped(true).show(ui, |ui| {
                                for (button_idx, state) in joystick.buttons_states.iter().enumerate() {
                                    ui.checkbox(&mut state.clone(), format!("Button {}", button_idx));

                                    if (button_idx + 1) % 5 == 0 {
                                        ui.end_row();
                                    }
                                }
                            });
                        });
                    });
                }
            });
                    // TODO Add capability to map buttons to MIDI notes, CCs, etc.
                    // Ideas: 1. Click on a button to select it, then click on a MIDI note or CC to map it
                    //        2. Click and drag from a button to a MIDI note or CC to map it
                    //        3. Keyboard shortcut to map a button to a MIDI note or CC
                    // Viz Ideas:
                    //        1. Create panels for each mapping, allow user to drag and drop to reorder)
                    //        2. Draw a line from each button to its mapping, like a circuit diagram
                    //        3. Buttons and axes at top, gravity+funnel style mapping has gravity pull messages down to the bottom
                    //        4. Mapping is a grid of buttons and axes

                    // Add a button to clear all mappings
                    // Add a button to save mappings to a file
                    // Add a button to load mappings from a file

            // A panel that shows all MIDI ports
            egui::TopBottomPanel::bottom("midi_panel").show(ctx, |ui| {
                ui.heading("MIDI Ports");
                egui::Grid::new("midi_grid").show(ui, |ui| {
                    for port in self.midi_out.ports().iter() {
                        let port_name = self.midi_out.port_name(port).unwrap_or_else(|_| "Unknown port".to_string());
                        if ui.selectable_label(self.out_port == *port, &port_name).clicked() {
                            self.out_port = port.clone();
                        }
                        ui.end_row();
                    }
                });
            });


        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let viewport_options = egui::ViewportBuilder {
        inner_size: Some(egui::Vec2::new(1440.0, 1440.0)), // Set your desired window size
        min_inner_size: Some(egui::Vec2::new(400.0, 300.0)), // Optional: Set minimum window size
        max_inner_size: Some(egui::Vec2::new(1600.0, 1200.0)), // Optional: Set maximum window size
        resizable: Some(true), // Optional: Set whether the window is resizable
        // ... set other viewport properties as needed
        ..Default::default()
    };

    let native_options = eframe::NativeOptions {
        viewport: viewport_options,
        // ... include other native options you might need
        ..Default::default()
    };

    eframe::run_native(
        "Joystick to MIDI Mapper",
        native_options,
        Box::new(|_cc| Box::new(MyApp::default())),
    )
}


