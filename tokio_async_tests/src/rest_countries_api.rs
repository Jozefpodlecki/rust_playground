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
    
    use super::*;

    #[tokio::test]
    async fn should_return_country() {
        let rest_countries_api = RestCountriesApi::new("https://restcountries.com/v3.1/alpha/");

        let country = rest_countries_api.get_country("pl").await.unwrap();
        let country = country.unwrap();

        assert_eq!(country.name.common, "poland");
        assert_eq!(country.name.official, "Republic of Poland");
    }
}