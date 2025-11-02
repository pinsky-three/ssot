use std::{
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
use source_processor::core::models::{Source, SourceType};
use std::error::Error;

pub struct BasicApp {
    g: Graph,
}

impl BasicApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let g = generate_graph();
        Self { g: Graph::from(&g) }
    }
}

impl App for BasicApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(&mut DefaultGraphView::new(&mut self.g));
        });
    }
}

fn generate_graph() -> StableGraph<(), ()> {
    let source = process_source(PathBuf::from("."));

    source.unwrap()
}

fn process_source(source: PathBuf) -> Result<StableDiGraph<(), ()>, Box<dyn Error>> {
    let contents = read_all_files_parallel(source.as_path().to_str().unwrap())?;

    let start = Instant::now();
    println!("total files: {:?}", contents.len());
    let end = Instant::now();

    println!("time taken: {:?}", end.duration_since(start));

    Ok(StableDiGraph::new())
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
