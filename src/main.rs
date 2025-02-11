use file_format::FileFormat;
use git2::{Cred, RemoteCallbacks};
use humansize::{format_size, DECIMAL};
use octocrab::Octocrab;
use walkdir::{DirEntry, WalkDir};

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

        let repo_temp_dir = std::env::temp_dir().join(repo_name);

        let _repo = match git2::Repository::open(repo_temp_dir.as_path()) {
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
                        .clone(clone_url.as_str(), &repo_temp_dir)
                        .unwrap()
                } else {
                    panic!("error: {}", err.message());
                }
            }
        };

        let repo_local_dir = repo_temp_dir.to_str().unwrap();
        // println!("  L {}", repo_local_dir);

        for (j, entry) in WalkDir::new(repo_local_dir)
            .into_iter()
            .filter_entry(|dir_entry| !is_hidden(dir_entry) && !black_listed(dir_entry))
            .enumerate()
        {
            let entry = entry.unwrap();

            if entry.path().is_file() {
                // println!("  L {}", entry.path().display());

                // let kind = infer::get_from_path(entry.path())
                //     .expect("file read successfully")
                //     .map(|x| x.matcher_type());
                let fmt = FileFormat::from_file(entry.path()).unwrap();

                // if the entry.path() is: `/var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/api-specification/json-schema/settings-req.schema.json`
                // and the repo_local_dir = "/var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/api-specification"

                // calculate the relative path
                let relative_path = entry.path().strip_prefix(repo_local_dir).unwrap();

                let size = format_size(entry.metadata().unwrap().len(), DECIMAL);

                println!(
                    "  [{}.{}] {} ({}) {}",
                    i,
                    j,
                    relative_path.display(),
                    size,
                    fmt
                );

                // match kind {
                //     Some(MatcherType::Text) | Some(MatcherType::Doc) => {

                //     } // let content = std::fs::read_to_string(entry.path()).unwrap();
                //     // println!("  [{}] {}: {:?}", i, entry.path().display(), kind);
                //     // println!("{}", content.len());
                //     _ => {}
                // }

                // if kind == "text/plain" {
                //     let content = std::fs::read_to_string(entry.path()).unwrap();

                // println!("{}", content.len());
                // }
            }
        }

        // for remote in remotes.iter() {

        // }
    }

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
