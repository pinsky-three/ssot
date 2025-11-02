use std::{
    collections::HashMap,
    fs,
    io::{self, Read},
    path::PathBuf,
    time::Instant,
};

use eframe::{App, CreationContext, NativeOptions, run_native};
use egui::{CollapsingHeader, Context, ScrollArea}; // Align2
use egui_graphs::{
    FruchtermanReingoldWithCenterGravity, FruchtermanReingoldWithCenterGravityState, Graph,
    GraphView, LayoutForceDirected,
};
use ignore::WalkBuilder;
use petgraph::{prelude::StableDiGraph, stable_graph::StableGraph};
use rand::Rng;
use rayon::prelude::*;
use source_processor::core::{
    computation::compute_references,
    models::{Source, SourceType},
};
use std::error::Error;

mod keybindings;
mod settings;

use keybindings::Command;
use settings::{SettingsInteraction, SettingsNavigation, SettingsStyle};

type FileGraph = StableDiGraph<String, ()>;
type GraphResult = Result<
    (
        FileGraph,
        HashMap<petgraph::stable_graph::NodeIndex, String>,
    ),
    Box<dyn Error>,
>;

// Type aliases for Force-Directed layout with Center Gravity
type ForceLayout = LayoutForceDirected<FruchtermanReingoldWithCenterGravity>;
type ForceState = FruchtermanReingoldWithCenterGravityState;

pub struct SourceCodeVisualizationApp {
    g: Graph<(), ()>,
    settings_interaction: SettingsInteraction,
    settings_navigation: SettingsNavigation,
    settings_style: SettingsStyle,
    show_sidebar: bool,
    show_debug_overlay: bool,
    show_keybindings_overlay: bool,
    keybindings_just_opened: bool,
    dark_mode: bool,
}

impl SourceCodeVisualizationApp {
    fn new(cc: &CreationContext<'_>) -> Self {
        let (g, label_map) = generate_graph(PathBuf::from("."));

        // Create egui_graphs graph with empty node data
        let mut petgraph_empty = StableDiGraph::<(), ()>::new();
        let mut petgraph_to_egui = HashMap::new();

        // Copy graph structure, mapping node indices
        for node_idx in g.node_indices() {
            let new_idx = petgraph_empty.add_node(());
            petgraph_to_egui.insert(node_idx, new_idx);
        }

        for edge in g.edge_indices() {
            let (source, target) = g.edge_endpoints(edge).unwrap();
            let egui_source = *petgraph_to_egui.get(&source).unwrap();
            let egui_target = *petgraph_to_egui.get(&target).unwrap();
            petgraph_empty.add_edge(egui_source, egui_target, ());
        }

        let mut egui_graph = Graph::from(&petgraph_empty);

        // Randomize initial node positions around the origin for better force-directed animation
        let mut rng = rand::thread_rng();
        let spread = 200.0; // Spread nodes in a 400x400 area centered at origin

        for &egui_idx in petgraph_to_egui.values() {
            if let Some(node) = egui_graph.node_mut(egui_idx) {
                let x = rng.gen_range(-spread..spread);
                let y = rng.gen_range(-spread..spread);
                node.set_location(egui::Pos2::new(x, y));
            }
        }

        // Set node labels using the mapping
        for (petgraph_idx, label) in label_map {
            if let Some(egui_idx) = petgraph_to_egui.get(&petgraph_idx)
                && let Some(node) = egui_graph.node_mut(*egui_idx)
            {
                node.set_label(label);
            }
        }

        let dark_mode = cc.egui_ctx.style().visuals.dark_mode;

        Self {
            g: egui_graph,
            settings_interaction: SettingsInteraction::default(),
            settings_navigation: SettingsNavigation::default(),
            settings_style: SettingsStyle::default(),
            show_sidebar: true,
            show_debug_overlay: true,
            show_keybindings_overlay: false,
            keybindings_just_opened: false,
            dark_mode,
        }
    }

    fn info_icon(ui: &mut egui::Ui, tip: &str) {
        ui.add_space(4.0);
        ui.small_button("ℹ").on_hover_text(tip);
    }

    fn ui_navigation(&mut self, ui: &mut egui::Ui) {
        CollapsingHeader::new("Navigation")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .checkbox(
                            &mut self.settings_navigation.fit_to_screen_enabled,
                            "fit_to_screen",
                        )
                        .clicked()
                    {
                        self.settings_navigation.zoom_and_pan_enabled =
                            !self.settings_navigation.zoom_and_pan_enabled;
                    }
                    Self::info_icon(
                        ui,
                        "Continuously recompute zoom/pan so whole graph stays visible.",
                    );
                });

                ui.add_enabled_ui(self.settings_navigation.fit_to_screen_enabled, |ui| {
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::Slider::new(
                                &mut self.settings_navigation.fit_to_screen_padding,
                                0.0..=1.0,
                            )
                            .text("padding"),
                        );
                        Self::info_icon(
                            ui,
                            "Extra fractional padding around graph when auto-fitting.",
                        );
                    });
                });

                ui.horizontal(|ui| {
                    if ui
                        .checkbox(
                            &mut self.settings_navigation.zoom_and_pan_enabled,
                            "zoom_and_pan",
                        )
                        .clicked()
                    {
                        self.settings_navigation.fit_to_screen_enabled =
                            !self.settings_navigation.fit_to_screen_enabled;
                    }
                    Self::info_icon(
                        ui,
                        "Manual navigation: trackpad pinch to zoom, Ctrl+wheel (zoom), drag (pan / node drag).",
                    );
                });

                ui.add_enabled_ui(self.settings_navigation.zoom_and_pan_enabled, |ui| {
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::Slider::new(&mut self.settings_navigation.zoom_speed, 0.01..=2.0)
                                .text("zoom_speed"),
                        );
                        Self::info_icon(
                            ui,
                            "Multiplier controlling zoom speed. Use trackpad pinch or Ctrl+wheel to zoom.",
                        );
                    });
                });
            });
    }

    fn ui_layout(&mut self, ui: &mut egui::Ui) {
        CollapsingHeader::new("Layout")
            .default_open(true)
            .show(ui, |ui| {
                let mut state = egui_graphs::get_layout_state::<
                    FruchtermanReingoldWithCenterGravityState,
                >(ui, None);

                // Animation section
                CollapsingHeader::new("Animation")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut state.base.is_running, "running");
                            Self::info_icon(ui, "Run/pause the simulation.");
                        });

                        ui.horizontal(|ui| {
                            ui.add(egui::Slider::new(&mut state.base.dt, 0.001..=0.2).text("dt"));
                            Self::info_icon(
                                ui,
                                "Integration time step. Larger = faster movement but less stable.",
                            );
                        });

                        ui.horizontal(|ui| {
                            ui.add(
                                egui::Slider::new(&mut state.base.damping, 0.0..=1.0)
                                    .text("damping"),
                            );
                            Self::info_icon(
                                ui,
                                "Velocity damping per frame. 1 = no damping, 0 = immediate stop.",
                            );
                        });
                    });

                // Forces section
                CollapsingHeader::new("Forces")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::Slider::new(&mut state.base.k_scale, 0.2..=3.0)
                                    .text("k_scale"),
                            );
                            Self::info_icon(
                                ui,
                                "Scale ideal edge length; >1 spreads the layout, <1 compacts it.",
                            );
                        });

                        ui.horizontal(|ui| {
                            ui.add(
                                egui::Slider::new(&mut state.base.c_attract, 0.1..=3.0)
                                    .text("c_attract"),
                            );
                            Self::info_icon(ui, "Multiplier for attractive force along edges.");
                        });

                        ui.horizontal(|ui| {
                            ui.add(
                                egui::Slider::new(&mut state.base.c_repulse, 0.1..=3.0)
                                    .text("c_repulse"),
                            );
                            Self::info_icon(ui, "Multiplier for repulsive force between nodes.");
                        });

                        ui.separator();
                        ui.label("Center Gravity");
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut state.extras.0.enabled, "enabled");
                            Self::info_icon(ui, "Pull nodes toward viewport center.");
                        });

                        ui.add_enabled_ui(state.extras.0.enabled, |ui| {
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::Slider::new(&mut state.extras.0.params.c, 0.0..=2.0)
                                        .text("strength"),
                                );
                                Self::info_icon(ui, "Coefficient for pull toward center.");
                            });
                        });
                    });

                egui_graphs::set_layout_state::<FruchtermanReingoldWithCenterGravityState>(
                    ui, state, None,
                );
            });
    }

    fn ui_interaction(&mut self, ui: &mut egui::Ui) {
        CollapsingHeader::new("Interaction").show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .checkbox(
                        &mut self.settings_interaction.dragging_enabled,
                        "dragging_enabled",
                    )
                    .clicked()
                    && self.settings_interaction.dragging_enabled
                {
                    self.settings_interaction.node_clicking_enabled = true;
                    self.settings_interaction.hover_enabled = true;
                }
                Self::info_icon(ui, "Drag nodes to reposition them.");
            });

            ui.horizontal(|ui| {
                ui.checkbox(
                    &mut self.settings_interaction.hover_enabled,
                    "hover_enabled",
                );
                Self::info_icon(ui, "Highlight nodes on hover.");
            });

            ui.horizontal(|ui| {
                if ui
                    .checkbox(
                        &mut self.settings_interaction.node_selection_enabled,
                        "node_selection",
                    )
                    .clicked()
                    && self.settings_interaction.node_selection_enabled
                {
                    self.settings_interaction.node_clicking_enabled = true;
                    self.settings_interaction.hover_enabled = true;
                }
                Self::info_icon(ui, "Click to select nodes.");
            });

            ui.horizontal(|ui| {
                if ui
                    .checkbox(
                        &mut self.settings_interaction.node_selection_multi_enabled,
                        "multi_selection",
                    )
                    .changed()
                    && self.settings_interaction.node_selection_multi_enabled
                {
                    self.settings_interaction.node_selection_enabled = true;
                    self.settings_interaction.node_clicking_enabled = true;
                    self.settings_interaction.hover_enabled = true;
                }
                Self::info_icon(ui, "Hold Ctrl/Cmd to select multiple nodes.");
            });
        });
    }

    fn ui_style(&mut self, ui: &mut egui::Ui) {
        CollapsingHeader::new("Style").show(ui, |ui| {
            ui.horizontal(|ui| {
                let mut dark = ui.ctx().style().visuals.dark_mode;
                if ui
                    .checkbox(&mut dark, "dark mode")
                    .on_hover_text("Toggle dark or light visuals")
                    .changed()
                {
                    if dark {
                        ui.ctx().set_visuals(egui::Visuals::dark());
                    } else {
                        ui.ctx().set_visuals(egui::Visuals::light());
                    }
                    self.dark_mode = dark;
                }
                Self::info_icon(ui, "Toggle between dark and light themes.");
            });

            ui.horizontal(|ui| {
                ui.checkbox(&mut self.settings_style.labels_always, "labels_always");
                Self::info_icon(ui, "Always show node labels.");
            });
        });
    }

    fn ui_debug(&mut self, ui: &mut egui::Ui) {
        CollapsingHeader::new("Debug")
            .default_open(false)
            .show(ui, |ui| {
                ui.checkbox(&mut self.show_debug_overlay, "show debug overlay")
                    .on_hover_text("Toggle debug overlay (d)");

                if ui
                    .button("keybindings")
                    .on_hover_text("Show keybindings (h / ?)")
                    .clicked()
                {
                    self.show_keybindings_overlay = true;
                    self.keybindings_just_opened = true;
                }
            });
    }

    fn ui_selected(&mut self, ui: &mut egui::Ui) {
        CollapsingHeader::new("Selected")
            .default_open(true)
            .show(ui, |ui| {
                ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                    for n in self.g.selected_nodes() {
                        ui.label(format!("{:?}", n));
                    }
                    for e in self.g.selected_edges() {
                        ui.label(format!("{:?}", e));
                    }
                });
            });
    }

    fn process_keybindings(&mut self, ctx: &egui::Context) {
        // Tab always toggles sidebar
        let mut toggle = false;
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Tab) && !i.modifiers.any() {
                toggle = true;
            }
        });

        // Consume Tab to prevent focus traversal
        let shifted = egui::Modifiers {
            shift: true,
            ..egui::Modifiers::default()
        };
        ctx.input_mut(|i| {
            let _ = i.consume_key(egui::Modifiers::default(), egui::Key::Tab);
            let _ = i.consume_key(shifted, egui::Key::Tab);
            i.events.retain(|ev| match ev {
                egui::Event::Key {
                    key: egui::Key::Tab,
                    ..
                } => false,
                egui::Event::Text(t) if t == "\t" => false,
                _ => true,
            });
        });

        if toggle {
            self.show_sidebar = !self.show_sidebar;
        }

        let cmds = keybindings::dispatch(ctx);

        for cmd in cmds {
            match cmd {
                Command::ToggleDebug => {
                    self.show_debug_overlay = !self.show_debug_overlay;
                }
                Command::OpenKeybindings => {
                    if self.show_keybindings_overlay {
                        self.show_keybindings_overlay = false;
                    } else {
                        self.show_keybindings_overlay = true;
                        self.keybindings_just_opened = true;
                    }
                }
                Command::CloseKeybindings => {
                    self.show_keybindings_overlay = false;
                }
                Command::ToggleNavMode => {
                    let enable_zoom_pan = !self.settings_navigation.zoom_and_pan_enabled;
                    self.settings_navigation.zoom_and_pan_enabled = enable_zoom_pan;
                    self.settings_navigation.fit_to_screen_enabled = !enable_zoom_pan;
                }
                Command::FitToScreenOnce => {
                    if !self.settings_navigation.fit_to_screen_enabled {
                        self.settings_navigation.fit_to_screen_enabled = true;
                    }
                }
                Command::ResetAll => {
                    self.settings_interaction = SettingsInteraction::default();
                    self.settings_navigation = SettingsNavigation::default();
                    self.settings_style = SettingsStyle::default();
                }
            }
        }

        // Close keybindings modal on any key/click
        if self.show_keybindings_overlay && !self.keybindings_just_opened {
            let mut any_interaction = false;
            ctx.input(|i| {
                for ev in &i.events {
                    match ev {
                        egui::Event::Key { pressed, .. } if *pressed => any_interaction = true,
                        egui::Event::PointerButton { pressed, .. } if *pressed => {
                            any_interaction = true
                        }
                        _ => {}
                    }
                }
            });
            if any_interaction {
                self.show_keybindings_overlay = false;
            }
        }
        self.keybindings_just_opened = false;
    }

    fn keybindings_modal(&mut self, ctx: &egui::Context) {
        if self.show_keybindings_overlay {
            egui::Window::new("Keybindings")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.heading("Keyboard Shortcuts");
                    ui.separator();

                    egui::Grid::new("keybindings_grid")
                        .num_columns(2)
                        .spacing(egui::vec2(12.0, 8.0))
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("Navigation").strong());
                            ui.end_row();

                            ui.code("Space");
                            ui.label("Fit to screen");
                            ui.end_row();

                            ui.code("Ctrl+Space");
                            ui.label("Toggle zoom & pan / fit to screen");
                            ui.end_row();

                            ui.code("Drag");
                            ui.label("Move nodes (when dragging enabled)");
                            ui.end_row();

                            ui.code("Ctrl+Wheel");
                            ui.label("Zoom (when zoom & pan enabled)");
                            ui.end_row();

                            ui.separator();
                            ui.end_row();

                            ui.label(egui::RichText::new("Interface").strong());
                            ui.end_row();

                            ui.code("Tab");
                            ui.label("Toggle sidebar");
                            ui.end_row();

                            ui.code("d");
                            ui.label("Toggle debug overlay");
                            ui.end_row();

                            ui.code("h / ?");
                            ui.label("Show/hide this keybindings dialog");
                            ui.end_row();

                            ui.code("Backspace");
                            ui.label("Reset all settings");
                            ui.end_row();

                            ui.code("Esc");
                            ui.label("Close dialogs");
                            ui.end_row();
                        });

                    ui.separator();
                    ui.label("Press any key or click anywhere to close");
                });
        }
    }

    // fn debug_overlay(&self, ui: &mut egui::Ui) {
    //     if !self.show_debug_overlay {
    //         return;
    //     }

    //     let painter = ui.ctx().debug_painter();
    //     let screen_rect = ui.ctx().input(|i| {
    //         i.viewport()
    //             .inner_rect
    //             .unwrap_or_else(|| egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::ZERO))
    //     });

    //     let pos = egui::pos2(screen_rect.right() - 200.0, 10.0);

    //     let text = format!(
    //         "Debug Info\n\
    //          Nodes: {}\n\
    //          Edges: {}\n\
    //          FPS: {:.1}",
    //         self.g.node_count(),
    //         self.g.edge_count(),
    //         ui.ctx().input(|i| 1.0 / i.stable_dt.max(0.001))
    //     );

    //     painter.debug_text(pos, Align2::RIGHT_TOP, egui::Color32::YELLOW, text);
    // }

    fn sidebar_toggle_button(&mut self, ui: &mut egui::Ui) {
        let g_rect = ui.max_rect();
        let btn_size = egui::vec2(32.0, 32.0);
        let right_margin = 10.0;
        let bottom_margin = 10.0;

        let toggle_pos = egui::pos2(
            g_rect.right() - right_margin - btn_size.x,
            g_rect.bottom() - bottom_margin - btn_size.y,
        );

        let help_pos = egui::pos2(toggle_pos.x - 40.0 - btn_size.x, toggle_pos.y);

        let (arrow, tip) = if self.show_sidebar {
            ("▶", "Hide sidebar (Tab)")
        } else {
            ("◀", "Show sidebar (Tab)")
        };

        // Help button
        egui::Area::new(egui::Id::new("help_btn"))
            .order(egui::Order::Foreground)
            .fixed_pos(help_pos)
            .movable(false)
            .show(ui.ctx(), |ui_area| {
                ui_area.set_clip_rect(g_rect);
                let help_text = egui::RichText::new("ℹ").size(18.0);
                let response = ui_area.add_sized(btn_size, egui::Button::new(help_text));
                if response.on_hover_text("Open keybindings (h / ?)").clicked() {
                    self.show_keybindings_overlay = true;
                    self.keybindings_just_opened = true;
                }
            });

        // Sidebar toggle button
        egui::Area::new(egui::Id::new("sidebar_toggle_btn"))
            .order(egui::Order::Foreground)
            .fixed_pos(toggle_pos)
            .movable(false)
            .show(ui.ctx(), |ui_area| {
                ui_area.set_clip_rect(g_rect);
                let arrow_text = egui::RichText::new(arrow).size(18.0);
                let response = ui_area.add_sized(btn_size, egui::Button::new(arrow_text));
                if response.on_hover_text(tip).clicked() {
                    self.show_sidebar = !self.show_sidebar;
                }
            });
    }
}

impl App for SourceCodeVisualizationApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        // Process keyboard shortcuts first
        self.process_keybindings(ctx);

        // Right sidebar with controls
        if self.show_sidebar {
            egui::SidePanel::right("right_panel")
                .default_width(300.0)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.heading("Controls");
                        ui.separator();

                        // Navigation controls
                        self.ui_navigation(ui);
                        ui.separator();

                        // Layout controls
                        self.ui_layout(ui);
                        ui.separator();

                        // Interaction controls
                        self.ui_interaction(ui);
                        ui.separator();

                        // Style controls
                        self.ui_style(ui);
                        ui.separator();

                        // Selected items
                        self.ui_selected(ui);
                        ui.separator();

                        // Debug controls
                        self.ui_debug(ui);
                    });
                });
        }

        // Central panel with graph
        egui::CentralPanel::default().show(ctx, |ui| {
            // Build settings from our state
            let settings_interaction = egui_graphs::SettingsInteraction::new()
                .with_dragging_enabled(self.settings_interaction.dragging_enabled)
                .with_hover_enabled(self.settings_interaction.hover_enabled)
                .with_node_clicking_enabled(self.settings_interaction.node_clicking_enabled)
                .with_node_selection_enabled(self.settings_interaction.node_selection_enabled)
                .with_node_selection_multi_enabled(
                    self.settings_interaction.node_selection_multi_enabled,
                )
                .with_edge_clicking_enabled(self.settings_interaction.edge_clicking_enabled)
                .with_edge_selection_enabled(self.settings_interaction.edge_selection_enabled)
                .with_edge_selection_multi_enabled(
                    self.settings_interaction.edge_selection_multi_enabled,
                );

            let settings_navigation = egui_graphs::SettingsNavigation::new()
                .with_fit_to_screen_enabled(self.settings_navigation.fit_to_screen_enabled)
                .with_zoom_and_pan_enabled(self.settings_navigation.zoom_and_pan_enabled)
                .with_zoom_speed(self.settings_navigation.zoom_speed)
                .with_fit_to_screen_padding(self.settings_navigation.fit_to_screen_padding);

            let settings_style = egui_graphs::SettingsStyle::new()
                .with_labels_always(self.settings_style.labels_always);

            // Render the graph
            ui.add(
                &mut GraphView::<_, _, _, _, _, _, ForceState, ForceLayout>::new(&mut self.g)
                    .with_interactions(&settings_interaction)
                    .with_navigations(&settings_navigation)
                    .with_styles(&settings_style),
            );

            // Draw debug overlay
            // self.debug_overlay(ui);

            // Draw toggle buttons
            self.sidebar_toggle_button(ui);
        });

        // Show keybindings modal if requested
        self.keybindings_modal(ctx);
    }
}

fn generate_graph(
    source_path: PathBuf,
) -> (
    StableGraph<String, ()>,
    HashMap<petgraph::stable_graph::NodeIndex, String>,
) {
    let source = process_source(source_path);

    source.unwrap()
}

fn process_source(source: PathBuf) -> GraphResult {
    let start_read = Instant::now();
    let sources = read_all_files_parallel(source.as_path().to_str().unwrap())?;
    println!(
        "✓ Read {} files in {:?}",
        sources.len(),
        start_read.elapsed()
    );

    let start_graph = Instant::now();
    let (graph, labels) = build_graph(sources)?;
    println!("✓ Built graph in {:?}", start_graph.elapsed());
    println!("  - Nodes: {}", graph.node_count());
    println!("  - Edges: {}", graph.edge_count());

    Ok((graph, labels))
}

fn build_graph(sources: Vec<Source>) -> GraphResult {
    let mut graph = StableDiGraph::new();
    let mut path_to_node = HashMap::new();
    let mut all_dirs = std::collections::HashSet::new();

    // Step 0: Collect all directories (parent directories of files)
    for source in &sources {
        let mut current_path = source.path.parent();
        while let Some(dir_path) = current_path {
            all_dirs.insert(dir_path.to_path_buf());
            current_path = dir_path.parent();
        }
    }

    // Step 1: Create nodes for directories first (tree backbone)
    for dir_path in &all_dirs {
        let dir_label = dir_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_else(|| dir_path.to_str().unwrap_or("."))
            .to_string();
        let node_idx = graph.add_node(dir_label);
        path_to_node.insert(dir_path.clone(), node_idx);
    }

    // Step 2: Create nodes for each source file with file names as labels
    for source in &sources {
        // Use filename only for cleaner display, fallback to full path
        let node_label = source
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_else(|| source.path.to_str().unwrap_or("unknown"))
            .to_string();

        let node_idx = graph.add_node(node_label);
        path_to_node.insert(source.path.clone(), node_idx);
    }

    println!("  - Created {} directory nodes", all_dirs.len());
    println!("  - Created {} file nodes", sources.len());
    println!("  - Total nodes: {}", graph.node_count());

    // Step 3: Create filesystem hierarchy edges (directory -> file, directory -> subdirectory)
    let mut hierarchy_edge_count = 0;
    for source in &sources {
        // Add edge from parent directory to file
        if let Some(parent_dir) = source.path.parent()
            && let (Some(&dir_node), Some(&file_node)) =
                (path_to_node.get(parent_dir), path_to_node.get(&source.path))
        {
            graph.add_edge(dir_node, file_node, ());
            hierarchy_edge_count += 1;
        }
    }

    // Add edges from parent directories to child directories
    for dir_path in &all_dirs {
        if let Some(parent_dir) = dir_path.parent()
            && let (Some(&parent_node), Some(&child_node)) =
                (path_to_node.get(parent_dir), path_to_node.get(dir_path))
        {
            graph.add_edge(parent_node, child_node, ());
            hierarchy_edge_count += 1;
        }
    }

    println!(
        "  - Created {} filesystem hierarchy edges",
        hierarchy_edge_count
    );

    // Debug: print first few file paths
    println!("  - Sample file paths:");
    for (i, (path, _)) in path_to_node.iter().enumerate().take(5) {
        println!("      {}: {:?}", i, path);
    }

    // Step 4: Compute references for all sources
    let start_ref = Instant::now();
    let all_references = compute_references(sources);
    let total_refs: usize = all_references.iter().map(|v| v.len()).sum();
    println!("  - Computed references in {:?}", start_ref.elapsed());
    println!("  - Total references found: {}", total_refs);

    // Step 5: Add edges based on references
    let mut edge_count = 0;
    let mut reference_count = 0;
    let mut matched_count = 0;
    let mut debug_printed = 0;

    println!("  - Processing references...");

    for source_refs in all_references {
        for reference in source_refs {
            reference_count += 1;

            // Debug: print first few references found
            if debug_printed < 5 {
                println!("    [REF] Found reference:");
                println!("          Route: {:?}", reference.route);
                println!("          From: {:?}", reference.source.path);
                debug_printed += 1;
            }

            // Get the source node
            if let Some(&source_node) = path_to_node.get(&reference.source.path) {
                // Try multiple matching strategies
                let target_node = path_to_node.iter().find_map(|(path, &node_idx)| {
                    let path_str = path.to_string_lossy();
                    let route_str = reference.route.to_string_lossy();
                    let path_str_ref: &str = &path_str;
                    let route_str_ref: &str = &route_str;

                    // Strategy 1: Direct path match (exact or ends with)
                    if path_str_ref.ends_with(route_str_ref) {
                        return Some(node_idx);
                    }

                    // Strategy 2: Normalize and compare (remove ./ prefix)
                    let normalized_path: String =
                        path_str.trim_start_matches("./").replace('\\', "/");
                    let normalized_route: String =
                        route_str.trim_start_matches("./").replace('\\', "/");
                    if normalized_path.ends_with(&normalized_route) {
                        return Some(node_idx);
                    }

                    // Strategy 3: Match module path to file structure
                    // e.g., "core/models.rs" should match "source-processor/src/core/models.rs"
                    // Check if the route appears as a path segment
                    let route_parts: Vec<&str> = normalized_route.split('/').collect();
                    let path_parts: Vec<&str> = normalized_path.split('/').collect();

                    // Check if route parts appear consecutively in path
                    if route_parts.len() <= path_parts.len() {
                        for window in path_parts.windows(route_parts.len()) {
                            if window == route_parts.as_slice() {
                                return Some(node_idx);
                            }
                        }
                    }

                    // Strategy 4: Match by filename (more lenient)
                    // e.g., "models.rs" matches any file named "models.rs"
                    if let (Some(file_name), Some(route_name)) = (
                        path.file_name().and_then(|n| n.to_str()),
                        reference.route.file_name().and_then(|n| n.to_str()),
                    ) && file_name == route_name
                    {
                        return Some(node_idx);
                    }

                    None
                });

                // Avoid self-loops
                if let Some(target_node) = target_node
                    && source_node != target_node
                {
                    graph.add_edge(source_node, target_node, ());
                    edge_count += 1;
                    matched_count += 1;
                }
            }
        }
    }

    println!("  - Found {} total references", reference_count);
    println!("  - Matched {} references to files", matched_count);
    println!("  - Added {} reference edges", edge_count);
    println!(
        "  - Total edges: {} ({} hierarchy + {} reference)",
        hierarchy_edge_count + edge_count,
        hierarchy_edge_count,
        edge_count
    );

    // Extract labels from graph nodes into HashMap
    let mut label_map = HashMap::new();
    for node_idx in graph.node_indices() {
        let label = graph[node_idx].clone();
        label_map.insert(node_idx, label);
    }

    Ok((graph, label_map))
}

fn read_all_files_parallel(dir_path: &str) -> io::Result<Vec<Source>> {
    // WalkBuilder respects .gitignore files by default
    let paths: Vec<_> = WalkBuilder::new(dir_path)
        .build()
        .filter_map(|e| e.ok())
        .filter(|entry| entry.file_type().is_some_and(|ft| ft.is_file()))
        .map(|entry| entry.path().to_owned())
        .collect();

    let sources: Vec<Source> = paths
        .par_iter()
        .filter_map(|path| {
            let mut file = fs::File::open(path).ok()?;
            let mut file_content = String::new();
            file.read_to_string(&mut file_content).ok()?;

            // Determine source type based on file extension
            let source_type = match path.extension().and_then(|ext| ext.to_str()) {
                Some("rs") => SourceType::RustCode,
                Some("py") => SourceType::PythonCode,
                Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("webp") => {
                    SourceType::Image
                }
                Some("txt") | Some("md") => SourceType::Text,
                Some(ext) => SourceType::Other(ext.to_string()),
                None => SourceType::Other("no_extension".to_string()),
            };

            Some(Source {
                path: path.clone(),
                source_type,
                content: file_content,
            })
        })
        .collect();

    Ok(sources)
}

fn main() {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    run_native(
        "Source Code Visualization",
        options,
        Box::new(|cc| Ok(Box::new(SourceCodeVisualizationApp::new(cc)))),
    )
    .unwrap();
}
