use reqwest::Client;
use anyhow::*;

pub struct TestGithubApi<'a> {
    base: &'a str,
    client: Client
}

impl<'a> TestGithubApi<'a> {
    pub fn new(base: &'a str) -> Self {
        let client = Client::new();

        Self {
            base,
            client
        }
    }

    pub async fn get_latest_release(&self) -> Result<()> {
        let url = format!("{}{}", self.base, "releases/latest");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use mockito::Server;    
    use super::*;

    #[tokio::test]
    async fn should_return_country() {
  
    }
}