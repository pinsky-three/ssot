// use std::fs::DirEntry;

use git2::{Cred, RemoteCallbacks};
use octocrab::Octocrab;

use walkdir::WalkDir;

#[tokio::main]

async fn main() -> octocrab::Result<()> {
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

        let temp_dir = std::env::temp_dir().join(repo_name);

        let _repo = match git2::Repository::open(temp_dir.as_path()) {
            Ok(repo) => {
                // println!("{}: already cloned", repo_name);
                repo
            }
            Err(err) => {
                if err
                    .message()
                    .contains("exists and is not an empty directory")
                {
                    git2::build::RepoBuilder::new()
                        .fetch_options(fo)
                        .clone(clone_url.as_str(), &temp_dir)
                        .unwrap()
                } else {
                    panic!("error: {}", err.message());
                }
            }
        };

        let repo_local_dir = temp_dir.to_str().unwrap();

        for entry in WalkDir::new(repo_local_dir) {
            let entry = entry.unwrap();

            if entry.path().is_file() {
                let kind = infer::get_from_path(entry.path())
                    .expect("file read successfully")
                    .map(|x| x.mime_type())
                    .unwrap_or("unknown");

                if kind == "text/plain" {
                    let content = std::fs::read_to_string(entry.path()).unwrap();
                    println!("{}: {:?} [{}]", entry.path().display(), kind, content.len());
                    // println!("{}", content.len());
                }
            }
        }

        // for remote in remotes.iter() {

        // }
    }

    Ok(())
}
