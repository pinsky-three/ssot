# SSOT (Single Source Of Truth)

## Description

SSOT is a Rust workspace containing two complementary tools for analyzing and visualizing source code:

1. **organization-processor**: Connects to GitHub, fetches and clones repositories from a specified organization, analyzes files within these repositories (metadata, content of small binary files), and generates a detailed markdown document (`output.md`) based on a template. The purpose is to create a single source of truth, possibly for LLMs or documentation.

2. **source-processor**: A visual tool that constructs and displays an interactive graph of source code files and their dependencies. It analyzes local references between files, creates a hierarchical directory structure, and renders the relationships using force-directed graph layout.

## Features

### organization-processor
*   Lists repositories and their files from a GitHub organization.
*   Extracts file metadata (path, format, size).
*   Includes content of small binary files (currently up to 5KB, identified as ArbitraryBinaryData).
*   Configurable GitHub organization.
*   Respects `.ssotignore` for skipping repositories.
*   Skips hidden files/directories and blacklisted directories (e.g., `node_modules`, `.git`).
*   Uses Askama templates for markdown generation.

### source-processor
*   **Parallel file reading** for efficient processing of large codebases.
*   **Graph construction** with local reference detection across source files.
*   **Directory hierarchy visualization** with directory nodes and parent-child edges.
*   **Interactive graph viewer** using egui with force-directed layout.
*   **Reference tracking** between files (imports, includes, requires, etc.).
*   Supports multiple file types through extensible source type detection.

## How it Works

**organization-processor** uses the GitHub API to list repositories in an organization. It then clones each repository, analyzes its files, and generates a markdown document using Askama templates.

**source-processor** walks a local directory, reads source files in parallel, detects references between them, constructs a graph with both file and directory nodes, and displays an interactive visualization with hierarchical relationships.

## Prerequisites

*   Rust (latest stable recommended).
*   Git.

## Setup / Configuration

### General Setup

1.  Clone the `ssot` repository: `git clone <repo_url>` (replace `<repo_url>` with the actual URL if available, otherwise use a placeholder).
2.  Navigate to the project directory: `cd ssot`.
3.  Build the workspace: `cargo build --release` (or `cargo build` for debug builds).

### organization-processor Setup

1.  Create a `.env` file by copying the example: `cp .env.example .env`.
2.  Edit the `.env` file and provide the necessary environment variables:
    *   `GITHUB_USERNAME`: Your GitHub username.
    *   `GITHUB_TOKEN`: Your GitHub personal access token. This token needs permissions to read repository information and clone repositories.
    *   `GITHUB_ORGANIZATION` (optional): The GitHub organization to scan. If not set, it defaults to "vacuul-dev".
3.  (Optional) Create a `.ssotignore` file in the root of the project. List the names of repositories (one per line) that you want the tool to skip.

### source-processor Setup

No additional configuration required. The tool analyzes the local directory from which it is run.

## Usage

### Running organization-processor

```bash
# From workspace root
cargo run --bin organization-processor --release

# Or directly from the binary
./target/release/organization-processor
```

The program will create/update the `output.md` file in the project root. This file will contain the aggregated information from the scanned repositories.

### Running source-processor

```bash
# From workspace root (analyzes the ssot directory itself)
cargo run --bin source-processor --release

# To analyze a specific directory, run from that directory
cd /path/to/your/project
/path/to/ssot/target/release/source-processor
```

The program will open an interactive GUI window displaying a graph of files and their dependencies. The graph uses:
- **Blue nodes**: Source files
- **Directory nodes**: Hierarchical structure
- **Edges**: References between files and directory containment relationships

## Output Examples

### organization-processor Output

The output is a markdown file (`output.md`).
*   It starts with the GitHub organization name.
*   Then, for each repository, it lists its name and clone URL.
*   Under each repository, it lists the files found, along with their:
    *   Full path in the temporary clone.
    *   Relative path within the repository.
    *   Detected file format.
    *   File size.
    *   For certain files (currently small binary files up to 5KB), their content is embedded in a code block.

### source-processor Output

The output is an interactive GUI window showing:
*   A force-directed graph layout of your source code.
*   Nodes representing both files and directories.
*   Edges showing:
    *   Direct references between files (e.g., imports, includes).
    *   Hierarchical relationships (directory containment).
*   Real-time graph manipulation and exploration capabilities.

## Recent Changes

### Graph Visualization & Analysis (Latest)
*   **Renamed BasicApp → SourceCodeVisualizationApp**: Clearer naming for the graph visualization component.
*   **Enhanced graph layout**: Updated force-directed layout with improved center gravity using Fruchterman-Reingold algorithm.
*   **Directory hierarchy nodes**: Graph now includes directory nodes with parent-child relationship edges.
*   **Local reference handling**: Improved detection and tracking of references between source files.
*   **Parallel file reading**: Implemented concurrent file processing using Rayon for better performance on large codebases.

### Organization Processing
*   **Dynamic organization configuration**: GitHub organization can now be configured via environment variables.
*   **Enhanced .ssotignore handling**: Better support for excluding repositories from analysis.
*   **Content size limits**: Binary file content expansion limited to 5KB for manageable output.
*   **Template system**: Integrated Askama for flexible markdown generation.

## Contributing

Contributions are welcome! Please feel free to open an issue or submit a pull request.

## License

This project is currently not licensed. Please refer to the project owner for licensing information.
