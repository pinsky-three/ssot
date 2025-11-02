use std::{
    collections::HashMap,
    fs,
    io::{self, Read},
    path::PathBuf,
    time::Instant,
};

use eframe::{App, CreationContext, NativeOptions, run_native};
use egui::Context;
use egui_graphs::{DefaultGraphView, Graph};
use ignore::WalkBuilder;
use petgraph::{prelude::StableDiGraph, stable_graph::StableGraph};
use rayon::prelude::*;
use source_processor::core::{
    computation::compute_references,
    models::{Source, SourceType},
};
use std::error::Error;

type FileGraph = StableDiGraph<(), ()>;
type NodeLabels = HashMap<usize, String>;
type GraphResult = Result<(FileGraph, NodeLabels), Box<dyn Error>>;

pub struct BasicApp {
    g: Graph,
    node_labels: HashMap<usize, String>,
}

impl BasicApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let (g, labels) = generate_graph();
        Self {
            g: Graph::from(&g),
            node_labels: labels,
        }
    }
}

impl App for BasicApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(&mut DefaultGraphView::new(&mut self.g));

            // Display graph statistics
            ui.separator();
            ui.label(format!("Nodes: {}", self.node_labels.len()));
        });
    }
}

fn generate_graph() -> (StableGraph<(), ()>, HashMap<usize, String>) {
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
    let mut node_labels = HashMap::new();

    // Step 1: Create nodes for each source file
    for source in &sources {
        let node_idx = graph.add_node(());
        let node_label = source.path.to_str().unwrap_or("unknown").to_string();

        path_to_node.insert(source.path.clone(), node_idx);
        node_labels.insert(node_idx.index(), node_label);
    }

    println!("  - Created {} nodes", graph.node_count());

    // Debug: print first few file paths
    println!("  - Sample file paths:");
    for (i, (path, _)) in path_to_node.iter().enumerate().take(5) {
        println!("      {}: {:?}", i, path);
    }

    // Step 1.5: Add filesystem hierarchy edges (directory structure)
    let mut fs_edge_count = 0;
    let mut edges_added = std::collections::HashSet::new();
    println!("  - Creating filesystem structure edges...");

    let paths: Vec<&PathBuf> = path_to_node.keys().collect();
    for (i, &path_a) in paths.iter().enumerate() {
        for &path_b in paths.iter().skip(i + 1) {
            if let (Some(&node_a), Some(&node_b)) =
                (path_to_node.get(path_a), path_to_node.get(path_b))
            {
                // Skip self-loops
                if node_a == node_b {
                    continue;
                }

                // Create edge key to avoid duplicates
                let edge_key = if node_a.index() < node_b.index() {
                    (node_a.index(), node_b.index())
                } else {
                    (node_b.index(), node_a.index())
                };

                // Check if files are in the same directory (siblings)
                let parent_a = path_a.parent();
                let parent_b = path_b.parent();

                // Connect files in the same directory
                if parent_a == parent_b && parent_a.is_some() && !edges_added.contains(&edge_key) {
                    graph.add_edge(node_a, node_b, ());
                    edges_added.insert(edge_key);
                    fs_edge_count += 1;
                }
            }
        }
    }

    println!("  - Created {} filesystem structure edges", fs_edge_count);

    // Step 2: Compute references for all sources
    let start_ref = Instant::now();
    let all_references = compute_references(sources);
    let total_refs: usize = all_references.iter().map(|v| v.len()).sum();
    println!("  - Computed references in {:?}", start_ref.elapsed());
    println!("  - Total references found: {}", total_refs);

    // Step 3: Add edges based on references
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
        "  - Total edges: {} ({} filesystem + {} reference)",
        fs_edge_count + edge_count,
        fs_edge_count,
        edge_count
    );

    Ok((graph, node_labels))
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
