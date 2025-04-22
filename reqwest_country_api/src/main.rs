use std::env;
use dotenv::dotenv;

use rest_countries_api::RestCountriesApi;
use anyhow::*;

mod rest_countries_api;
mod models;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let api_url = env::var("API_URL")?;
    let rest_countries_api = RestCountriesApi::new(&api_url);
    let name = "pl";

    let country = rest_countries_api.get_country(name).await?;

    println!("{:#?}", country);

    Ok(())
}

