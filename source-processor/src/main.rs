use std::{error::Error, path::PathBuf};

use petgraph::prelude::StableDiGraph;

fn process_source(source: PathBuf) -> Result<StableDiGraph<(), ()>, Box<dyn Error>> {
    Ok(StableDiGraph::new())
}

fn main() {
    println!("Hello, world!");
}
