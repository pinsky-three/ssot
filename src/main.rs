use askama::Template;
use file_format::FileFormat;
use git2::{Cred, RemoteCallbacks};
use humansize::{format_size, DECIMAL};
use octocrab::Octocrab;
use std::error::Error;
use std::path::PathBuf;
use walkdir::{DirEntry, WalkDir};

#[derive(Template)]
#[template(path = "composer.md")]
struct ComposerTemplate {
    project: Project,
}

struct Project {
    github_organization: String,
    repositories: Vec<Repository>,
}

impl Project {
    fn expand_content(
        &mut self,
        filter_fn: impl Fn(&&mut Source) -> bool,
    ) -> Result<(), Box<dyn Error>> {
        self.repositories
            .iter_mut()
            .flat_map(|repo| &mut repo.sources)
            .filter(filter_fn)
            .for_each(|source| {
                source.content = std::fs::read_to_string(source.path.0.as_path()).ok();
            });

        Ok(())
    }
}

struct Repository {
    name: String,
    clone_url: String,
    sources: Vec<Source>,
}

struct Source {
    path: InternalPathBuf,
    relative_path: InternalPathBuf,
    format: FileFormat,
    size: DisplayableOptionU64,
    content: Option<String>,
}

pub struct InternalPathBuf(pub PathBuf);

impl std::fmt::Display for InternalPathBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

// define new type that wraps Option<u64> to implement display
pub struct DisplayableOptionU64(pub Option<u64>);

impl std::fmt::Display for DisplayableOptionU64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Some(v) => write!(f, "{}", v),
            None => write!(f, "None"),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().expect("dotenv failed");

    let main_username =
        std::env::var("GITHUB_USERNAME").expect("GITHUB_USERNAME env variable is required");

    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env variable is required");

    let octocrab = Octocrab::builder().personal_token(token.clone()).build()?;

    let my_repos = octocrab
        .orgs("vacuul-dev")
        .list_repos()
        .per_page(250)
        .page(0u32)
        .send()
        .await?;

    let mut project = Project {
        github_organization: "vacuul-dev".to_string(),
        repositories: vec![],
    };

    for (i, repo) in my_repos.into_iter().enumerate() {
        let clone_url = repo.clone_url.unwrap();
        println!("{}: {}", i, &clone_url);

        let repo_name = repo.name;

        let mut callbacks = RemoteCallbacks::new();

        callbacks.credentials(|_url, _username_from_url, _allowed_types| {
            Cred::userpass_plaintext(main_username.as_str(), token.as_str())
        });

        let mut fo = git2::FetchOptions::new();

        fo.remote_callbacks(callbacks);

        let repo_temp_dir = std::env::temp_dir().join(repo_name.clone());

        let ssot_ignore = std::fs::read_to_string(".ssotignore")
            .unwrap_or("".to_string())
            .split("\n")
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        if ssot_ignore.contains(&repo_name) {
            continue;
        }

        if repo_temp_dir.exists() {
            let items = repo_temp_dir.read_dir().unwrap().count();
            println!("items: {}", items);

            if repo_temp_dir.is_dir()
                && repo_temp_dir
                    .read_dir()
                    .unwrap()
                    .filter(|entry| entry.as_ref().unwrap().path().starts_with(".git"))
                    .count()
                    < 1
            {
                std::fs::remove_dir_all(repo_temp_dir.as_path()).unwrap();

                // println!("cloning: {}", repo_name);

                // repo_name
                git2::build::RepoBuilder::new()
                    .fetch_options(fo)
                    .clone(clone_url.as_str(), &repo_temp_dir)
                    .unwrap();
            }

            let _repo = git2::Repository::open(repo_temp_dir.as_path()).unwrap();
        }

        let repo_local_dir = repo_temp_dir.to_str().unwrap();

        let mut repo = Repository {
            name: repo_name,
            clone_url: clone_url.to_string(),
            sources: vec![],
        };

        for (j, entry) in WalkDir::new(repo_local_dir)
            .into_iter()
            .filter_entry(|dir_entry| !is_hidden(dir_entry) && !black_listed(dir_entry))
            .enumerate()
        {
            let entry = entry.unwrap();

            if entry.path().is_file() {
                let fmt = FileFormat::from_file(entry.path()).unwrap();

                let relative_path = entry.path().strip_prefix(repo_local_dir).unwrap();

                let size = format_size(entry.metadata().unwrap().len(), DECIMAL);

                let source = Source {
                    format: fmt,
                    path: InternalPathBuf(entry.path().to_path_buf()),
                    relative_path: InternalPathBuf(relative_path.to_path_buf()),
                    size: DisplayableOptionU64(Some(entry.metadata().unwrap().len())),
                    content: None,
                };

                println!(
                    "  [{}.{}] {} ({}) {}",
                    i,
                    j,
                    relative_path.display(),
                    size,
                    fmt
                );

                repo.sources.push(source);
            }
        }

        project.repositories.push(repo);
    }

    let total_repos = project.repositories.len();

    println!("Total repos: {}", total_repos);

    project.expand_content(|source| {
        let size = source.size.0.unwrap();

        let max_size_in_kb = 5;

        size < 1024 * max_size_in_kb && (source.format == FileFormat::ArbitraryBinaryData)
        // || source.format == FileFormat::ScalableVectorGraphics)
    })?;

    let composition = ComposerTemplate { project };

    std::fs::write("output.md", composition.render().unwrap()).unwrap();

    Ok(())
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn black_listed(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("node_modules"))
        .unwrap_or(false)
}
