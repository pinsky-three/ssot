use git2::{Cred, RemoteCallbacks};
use octocrab::Octocrab;

#[tokio::main]
async fn main() -> octocrab::Result<()> {
    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env variable is required");

    let octocrab = Octocrab::builder().personal_token(token.clone()).build()?;

    let my_repos = octocrab
        .orgs("vacuul-dev")
        // .current()
        // .list_repos_for_authenticated_user()
        .list_repos()
        // .type_("owner")
        // .sort("updated")
        .per_page(250)
        .page(0u32)
        // .page(0)
        .send()
        .await?;

    for (i, repo) in my_repos.into_iter().enumerate() {
        let clone_url = repo.clone_url.unwrap();
        println!("{}: {}", i, &clone_url);
        // let owner = repo.owner.login;
        // let repo = repo.name;

        let repo_name = repo.name;

        // let a = octocrab.commits(owner, repo);

        let mut callbacks = RemoteCallbacks::new();

        callbacks.credentials(|_a, _b, _c| Cred::userpass_plaintext("bregydoc", token.as_str()));

        let mut fo = git2::FetchOptions::new();

        fo.remote_callbacks(callbacks);

        let temp_dir = std::env::temp_dir().join(repo_name);

        let repo = git2::build::RepoBuilder::new()
            .fetch_options(fo)
            .clone(clone_url.as_str(), &temp_dir)
            .unwrap();

        let remotes = repo.remotes().unwrap();

        for remote in remotes.iter() {
            println!(" . Remote: {}", remote.unwrap());
        }
    }

    Ok(())
}
