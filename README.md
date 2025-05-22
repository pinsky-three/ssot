# SSOT (Single Source Of Truth)

## Description

This project connects to GitHub, fetches and clones repositories from a specified organization, analyzes files within these repositories (metadata, content of small binary files), and generates a detailed markdown document (`output.md`) based on a template. The purpose is to create a single source of truth, possibly for LLMs or documentation.

## Features

*   Lists repositories and their files.
*   Extracts file metadata (path, format, size).
*   Includes content of small binary files (currently up to 5KB, identified as ArbitraryBinaryData).
*   Configurable GitHub organization.
*   Respects `.ssotignore` for skipping repositories.
*   Skips hidden files/directories and blacklisted directories (e.g., `node_modules`, `.git`).

## How it Works

The project uses the GitHub API to list repositories in an organization. It then clones each repository, analyzes its files, and generates a markdown document using Askama templates.

## Prerequisites

*   Rust (latest stable recommended).
*   Git.

## Setup / Configuration

1.  Clone the `ssot` repository: `git clone <repo_url>` (replace `<repo_url>` with the actual URL if available, otherwise use a placeholder).
2.  Navigate to the project directory: `cd ssot`.
3.  Create a `.env` file by copying the example: `cp .env.example .env`.
4.  Edit the `.env` file and provide the necessary environment variables:
    *   `GITHUB_USERNAME`: Your GitHub username.
    *   `GITHUB_TOKEN`: Your GitHub personal access token. This token needs permissions to read repository information and clone repositories.
    *   `GITHUB_ORGANIZATION` (optional): The GitHub organization to scan. If not set, it defaults to "vacuul-dev".
5.  (Optional) Create a `.ssotignore` file in the root of the project. List the names of repositories (one per line) that you want the tool to skip.

## Usage

1.  Run the program using Cargo: `cargo run`.
2.  The program will create/update the `output.md` file in the project root. This file will contain the aggregated information from the scanned repositories.

## Output Example

The output is a markdown file (`output.md`).
*   It starts with the GitHub organization name.
*   Then, for each repository, it lists its name and clone URL.
*   Under each repository, it lists the files found, along with their:
    *   Full path in the temporary clone.
    *   Relative path within the repository.
    *   Detected file format.
    *   File size.
    *   For certain files (currently small binary files), their content is embedded in a code block.

## Contributing

Contributions are welcome! Please feel free to open an issue or submit a pull request.

## License

This project is currently not licensed. Please refer to the project owner for licensing information.
