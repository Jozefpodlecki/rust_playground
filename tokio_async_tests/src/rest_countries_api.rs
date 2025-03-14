use reqwest::Client;
use anyhow::*;
use crate::models::Country;


pub struct RestCountriesApi<'a> {
    base: &'a str,
    client: Client
}

impl<'a> RestCountriesApi<'a> {
    pub fn new(base: &'a str) -> Self {
        let client = Client::new();

        Self {
            base,
            client
        }
    }

    pub async fn get_country(&self, name: &str) -> Result<Option<Country>> {
        let url = format!("{}{}", self.base, name);

        let response = self.client.get(&url).send().await?;
        let countries = response.json::<Vec<Country>>().await?;
        let country = countries.first().cloned();

        Ok(country)
    }
}

#[cfg(test)]
mod tests {
    use mockito::Server;    
    use super::*;

    #[tokio::test]
    async fn should_return_country() {
        let mut server = Server::new_async().await;
        server.mock("GET", "/pl")
            .with_status(200)
            .with_body_from_file("src/tests/pl.json").create();

        let url = format!("{}/", server.url());
        let rest_countries_api = RestCountriesApi::new(&url);

        let country = rest_countries_api.get_country("pl").await.unwrap();
        let country = country.unwrap();

        assert_eq!(country.name.common, "Poland");
        assert_eq!(country.name.official, "Republic of Poland");
    }
}