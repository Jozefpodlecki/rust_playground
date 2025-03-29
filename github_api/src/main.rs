mod github_api;

use std::{env, fs::File};
use std::io::copy;
use dotenv::dotenv;

use github_api::TestGithubApi;
use octocrate::*;
use anyhow::*;
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let api_url = env::var("API_URL")?;
    let github_user = env::var("GITHUB_USER")?;
    let github_project = env::var("GITHUB_PROJECT")?;
    let personal_access_token_str = env::var("GITHUB_PAT")?;
    // let github_api = TestGithubApi::new(&api_url);

    println!("{} {} {}", github_user, github_project, personal_access_token_str);

    let personal_access_token = PersonalAccessToken::new(personal_access_token_str);

    let config = APIConfig::with_token(personal_access_token).shared();
  
    let api = GitHubAPI::new(&config);
    let result = api.repos.get_latest_release(github_user, github_project).send().await;
    let client = Client::new();
  

    match result {
        std::result::Result::Ok(release) => {
            let release: Release = release;

            let assets: Vec<_> = release.assets.iter()
                .filter(|pr| pr.name.ends_with(".dll"))
                .collect();

            let asset = assets.first().unwrap();
            let response = client.get(&asset.browser_download_url).send().await?;
            let mut file = File::create(&asset.name)?;
            let bytes = response.bytes().await?;
            let mut content = bytes.as_ref();
            copy(&mut content, &mut file)?;
        },
        Err(err) => println!("{:?}", err)
    };

    Ok(())
}

