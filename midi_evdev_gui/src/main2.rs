#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use eframe::{App, CreationContext, egui};
use sdl2::event::Event;
use sdl2::joystick::Joystick;
use sdl2::JoystickSubsystem;
use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort};
use midi_types::{MidiMessage, Channel, Control, Value7, Note};
use std::fs::File;
use egui::{Context, Pos2};
use midi_convert::render_slice::MidiRenderSlice;
use egui_graphs::{DefaultEdgeShape, DefaultNodeShape, Graph, GraphView};
use petgraph::Directed;
use petgraph::stable_graph::{StableGraph};
use petgraph::visit::Walker;
use rayon::iter::Positions;

const MAX_JOYSTICK_VALUE: i32 = 32767;

struct JoystickState {
    joystick: Joystick,
    axes_states: Vec<f32>,
    buttons_states: Vec<bool>,
    position: Option<egui::Pos2>
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
            position: None,
        }
    }

    fn update_axis(&mut self, axis_idx: u8, value: i16) {
        if let Some(state) = self.axes_states.get_mut(axis_idx as usize) {
            *state = value as f32; // Convert or scale value as needed
        }
    }
    // TODO Add more update methods for buttons, hats, etc.
}

#[derive(Clone)]
enum JoystickNode {
    Axis(AxisNode),
    Button(ButtonNode),
}

#[derive(Clone)]
struct AxisNode {
    axis_index: usize,
    value: f32,
    position: Option<Pos2>,
}

#[derive(Clone)]
struct ButtonNode {
    button_index: usize,
    state: bool,
    position: Option<Pos2>,
}

impl AxisNode {
    fn new(axis_index: usize, value: f32) -> Self {
        Self { axis_index, value, position: None }
    }
}

impl ButtonNode {
    fn new(button_index: usize, state: bool) -> Self {
        Self { button_index, state, position: None }
    }
}

struct MyApp {
    sdl_context: sdl2::Sdl,
    joysticks: Vec<JoystickState>,
    event_pump: sdl2::EventPump,
    midi_out: MidiOutput,
    ports: Vec<midir::MidiOutputPort>,
    out_port: Option<MidiOutputPort>,
    port_name: Option<String>,
    connection_graph: Graph<(JoystickNode), (), Directed>,
}

impl Default for MyApp {
    fn default() -> Self {
        let sdl_context = sdl2::init().unwrap();
        let joystick_subsystem = sdl_context.joystick().unwrap();
        let joysticks = open_virpil_joysticks(&joystick_subsystem);
        let event_pump = sdl_context.event_pump().unwrap();

        let midi_out = MidiOutput::new("My MIDI Output").expect("Failed to create MIDI output");
        let ports = midi_out.ports();
        let out_port = ports.get(0).cloned();

        let port_name = out_port.as_ref()
            .and_then(|port| midi_out.port_name(port).ok())
            .or_else(|| Some("Unknown port".to_string()));
        let connection_graph = generate_graph(&joysticks, &[], &[]);

        Self {
            sdl_context,
            joysticks,
            event_pump,
            midi_out,
            ports,
            out_port,
            port_name,
            connection_graph,
        }
    }
}

// fn generate_graph(joysticks: &[JoystickState]) -> Graph<(JoystickNode), ()> {
//     let mut g = StableGraph::new();
//
//     for joystick in joysticks {
//         let nodes = joystick_to_nodes(joystick);
//
//         // let mut prev_node_index = None;
//         for node in nodes {
//             let node_index = g
//
//                 .add_node(node); // Add the joystick node to the graph
//             // if let Some(prev_index) = prev_node_index {
//             //     // Connect each node to the previous node
//             //     g.add_edge(prev_index, node_index, ());
//             // }
//             // prev_node_index = Some(node_index);
//         }
//     }
//     // Set all the node positions
//     let graph = Graph::from(&g).nodes_iter().enumerate().map(|(idx, node)| (idx, node.display().pos)).collect::<Vec<_>>();
//
// }

fn generate_graph(
    joysticks: &[JoystickState],
    axes_positions: &[Vec<Option<Pos2>>],
    buttons_positions: &[Vec<Option<Pos2>>],
) -> Graph<JoystickNode, ()> {
    let mut g = StableGraph::new();

    for (joystick, (axes_pos, buttons_pos)) in joysticks.iter().zip(axes_positions.iter().zip(buttons_positions.iter())) {
        //let nodes = joystick_to_nodes(joystick, axes_pos, buttons_pos);
        // Set static positions for now
        let positions = vec![Some(egui::Pos2::new(100.0, 100.0)); joystick.axes_states.len() + joystick.buttons_states.len()];
        let nodes = joystick_to_nodes(joystick, &positions, &positions);


        for node in nodes {
            g.add_node(node);
        }
    }

    Graph::from(&g)
}

fn create_node_with_position(node_type: NodeType, index: usize, state: f32, position: Option<egui::Pos2>) -> JoystickNode {
    match node_type {
        NodeType::Axis => JoystickNode::Axis(AxisNode {
            axis_index: index,
            value: state,
            position,
        }),
        NodeType::Button => JoystickNode::Button(ButtonNode {
            button_index: index,
            state: state != 0.0,  // Assuming button state is represented as a float
            position,
        }),
    }
}

// Enum to distinguish between node types
enum NodeType {
    Axis,
    Button,
}

fn joystick_to_nodes(
    joystick: &JoystickState,
    axes_positions: &[Option<egui::Pos2>],
    buttons_positions: &[Option<egui::Pos2>],
) -> Vec<JoystickNode> {
    let mut nodes = Vec::new();
    for (axis_idx, (&state, &position)) in joystick.axes_states.iter().zip(axes_positions.iter()).enumerate() {
        //nodes.push(create_node_with_position(NodeType::Axis, axis_idx, state, position));
        nodes.push(create_node_with_position(NodeType::Axis, axis_idx, state, Some(egui::Pos2::new(100.0, 100.0))));
    }
    for (button_idx, (&state, &position)) in joystick.buttons_states.iter().zip(buttons_positions.iter()).enumerate() {
        //nodes.push(create_node_with_position(NodeType::Button, button_idx, if state { 1.0 } else { 0.0 }, position));
        nodes.push(create_node_with_position(NodeType::Button, button_idx, if state { 1.0 } else { 0.0 }, Some(egui::Pos2::new(100.0, 100.0))));
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

        let mut axes_positions_all_joysticks = Vec::new();
        let mut buttons_positions_all_joysticks = Vec::new();

        // Render the graph in a full-sized layer
        // egui::Area::new("graph_area")
        //     .order(egui::Order::Foreground)
        //     .show(ctx, |ui| {
        //         ui.add(&mut GraphView::<
        //             _,
        //             _,
        //             _,
        //             _,
        //             DefaultNodeShape,
        //             DefaultEdgeShape,
        //         >::new(&mut self.connection_graph));
        //     });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Joystick to MIDI Mapper");
            ui.vertical(|ui| {
                for joystick in &self.joysticks {
                    let mut joystick_axes_positions = Vec::new();
                    let mut joystick_buttons_positions = Vec::new();

                    ui.horizontal(|ui| {
                        egui::Window::new(&joystick.joystick.name()).show(ui.ctx(), |ui| {
                            ui.label(format!("Number of axes: {}", joystick.joystick.num_axes()));
                            ui.label(format!("Number of buttons: {}", joystick.joystick.num_buttons()));

                            // Slider for each axis
                            ui.vertical(|ui| {
                                for (axis_idx, state) in joystick.axes_states.iter().enumerate() {
                                    let slider_response = ui.add(egui::Slider::new(&mut state.clone(), 0.0..=MAX_JOYSTICK_VALUE as f32).text(format!("Axis {}", axis_idx)));
                                    let slider_center = slider_response.rect.center();
                                    joystick_axes_positions.push(Some(slider_center));
                                }
                            });

                            egui::Grid::new("button_grid").striped(true).show(ui, |ui| {
                                for (button_idx, state) in joystick.buttons_states.iter().enumerate() {
                                    let button_response = ui.checkbox(&mut state.clone(), format!("Button {}", button_idx));
                                    let button_center = button_response.rect.center();
                                    if (button_idx + 1) % 5 == 0 {
                                        ui.end_row();
                                    }
                                    joystick_buttons_positions.push(Some(button_center));
                                }
                            });
                        });
                    });

                    axes_positions_all_joysticks.push(joystick_axes_positions);
                    buttons_positions_all_joysticks.push(joystick_buttons_positions);
                }
            });

            // Get all node positions
            // let node_positions = self.connection_graph.nodes_iter()
            //     .map(|(idx, node)| (idx, node.display().pos))
            //     .collect::<Vec<_>>();
            // println!("node_positions: {:?}", node_positions);


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
            egui::Window::new("MIDI Ports").show(ctx, |ui| {
                match self.midi_out.ports().as_slice() {
                    [] => {
                        ui.label("No MIDI output ports available.");
                    },
                    ports => {
                        for port in ports {
                            let port_name = self.midi_out.port_name(port).unwrap_or_else(|_| "Unknown port".to_string());
                            if ui.selectable_label(self.out_port.as_ref() == Some(port), &port_name).clicked() {
                                self.out_port = Some(port.clone());
                            }
                        }
                    }
                }
            });
            // Add the graph
            self.connection_graph = generate_graph(&self.joysticks, &axes_positions_all_joysticks, &buttons_positions_all_joysticks);
            // Try rendering as background layer
            egui::Area::new("graph_area")
                .order(egui::Order::Background)
                .fixed_pos(egui::pos2(0.0, 0.0))  // Position at the top-left corner
                .show(ctx, |ui| {
                    let screen_size = ctx.screen_rect().size();
                    egui::Frame::default()
                        .inner_margin(egui::vec2(0.0, 0.0))  // No margin
                        .show(ui, |ui| {
                            ui.set_min_size(screen_size);  // Set the size to cover the entire screen
                            ui.add(&mut GraphView::<
                                _,
                                _,
                                _,
                                _,
                                DefaultNodeShape,
                                DefaultEdgeShape,
                            >::new(&mut self.connection_graph));
                        });
                });
        });

    }
}

fn main() -> Result<(), eframe::Error> {
    let viewport_options = egui::ViewportBuilder {
        inner_size: Some(egui::Vec2::new(1440.0, 1440.0)), // Set your desired window size
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
