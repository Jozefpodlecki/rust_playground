use rest_countries_api::RestCountriesApi;
use anyhow::*;

mod rest_countries_api;
mod models;

#[tokio::main]
async fn main() -> Result<()> {

    let rest_countries_api = RestCountriesApi::new("https://restcountries.com/v3.1/alpha/");
    let name = "pl";

    let country = rest_countries_api.get_country(name).await?;

    println!("{:#?}", country);

    Ok(())
}

