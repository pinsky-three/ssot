use std::{
    collections::HashMap,
    fs,
    io::{self, Read},
    path::PathBuf,
    time::Instant,
};

use eframe::{App, CreationContext, NativeOptions, run_native};
use egui::Context;
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

pub struct BasicApp {
    g: Graph<(), ()>,
}

impl BasicApp {
    fn new(_cc: &CreationContext<'_>) -> Self {
        let (g, label_map) = generate_graph();

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

        Self { g: egui_graph }
    }
}

impl App for BasicApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(&mut GraphView::<_, _, _, _, _, _, ForceState, ForceLayout>::new(&mut self.g));

            // Display graph statistics
            ui.separator();
            ui.label(format!("Nodes: {}", self.g.node_count()));
            ui.label(format!("Edges: {}", self.g.edge_count()));
        });
    }
}

fn generate_graph() -> (
    StableGraph<String, ()>,
    HashMap<petgraph::stable_graph::NodeIndex, String>,
) {
    let source = process_source(PathBuf::from("."));

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
    run_native(
        "basic",
        NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(BasicApp::new(cc)))),
    )
    .unwrap();
}
