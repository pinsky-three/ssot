use crate::core::models::{Reference, ReferenceType, Source, SourceType};
use std::path::PathBuf;

fn heuristic_rust_code_references(source: Source) -> Vec<Reference> {
    let mut references = Vec::new();

    for line in source.content.lines() {
        let trimmed = line.trim();

        // Match Rust use statements
        if trimmed.starts_with("use ") {
            // Extract the module path (everything between "use " and ";" or "{")
            let use_part = trimmed
                .strip_prefix("use ")
                .unwrap_or("")
                .split(';')
                .next()
                .unwrap_or("")
                .split('{')
                .next()
                .unwrap_or("")
                .trim();

            if !use_part.is_empty() {
                // Only process local imports (crate::, self::, super::, or local package names)
                // Skip external crates like std, serde, etc.
                let is_local = use_part.starts_with("crate::")
                    || use_part.starts_with("self::")
                    || use_part.starts_with("super::")
                    || use_part.starts_with("source_processor::")
                    || use_part.starts_with("organization_processor::");

                if is_local {
                    // Remove the "crate::", "self::", "super::", or package prefix
                    let module_path = use_part
                        .strip_prefix("crate::")
                        .or_else(|| use_part.strip_prefix("self::"))
                        .or_else(|| use_part.strip_prefix("super::"))
                        .or_else(|| use_part.strip_prefix("source_processor::"))
                        .or_else(|| use_part.strip_prefix("organization_processor::"))
                        .unwrap_or(use_part);

                    // Convert module path to potential file path
                    let path_str = module_path
                        .replace("::", "/")
                        .trim_end_matches(|c: char| !c.is_alphanumeric() && c != '_')
                        .to_string();

                    let route = PathBuf::from(format!("{}.rs", path_str));

                    references.push(Reference {
                        source: source.clone(),
                        reference_type: ReferenceType::Uses,
                        route,
                    });
                }
            }
        }
    }

    references
}

fn heuristic_python_code_references(source: Source) -> Vec<Reference> {
    let mut references = Vec::new();

    for line in source.content.lines() {
        let trimmed = line.trim();

        // Match "import module" statements
        if trimmed.starts_with("import ") && !trimmed.starts_with("import(") {
            let import_part = trimmed
                .strip_prefix("import ")
                .unwrap_or("")
                .split_whitespace()
                .next()
                .unwrap_or("")
                .split(',')
                .next()
                .unwrap_or("")
                .trim();

            if !import_part.is_empty() {
                // Convert module path to file path
                let path_str = import_part.replace('.', "/");
                let route = PathBuf::from(format!("{}.py", path_str));

                references.push(Reference {
                    source: source.clone(),
                    reference_type: ReferenceType::Imports,
                    route,
                });
            }
        }

        // Match "from module import something" statements
        if let Some(module_part) = trimmed
            .strip_prefix("from ")
            .and_then(|s| s.split(" import ").next())
        {
            let module = module_part.trim();
            if !module.is_empty() && module != "." && !module.starts_with("..") {
                // Convert module path to file path
                let path_str = module.replace('.', "/");
                let route = PathBuf::from(format!("{}.py", path_str));

                references.push(Reference {
                    source: source.clone(),
                    reference_type: ReferenceType::Imports,
                    route,
                });
            }
        }
    }

    references
}

pub fn compute_references(sources: Vec<Source>) -> Vec<Vec<Reference>> {
    let mut references = Vec::new();

    for source in sources {
        let source_references = match source.source_type {
            SourceType::RustCode => heuristic_rust_code_references(source),
            SourceType::PythonCode => heuristic_python_code_references(source),
            _ => vec![],
        };

        references.push(source_references);
    }

    references
}
