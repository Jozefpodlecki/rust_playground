use std::{fs::File, path::PathBuf};

use reqwest::Client;
use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AwsIpRanges {
    pub sync_token: String,
    pub create_date: String,
    pub prefixes: Vec<IpPrefix>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IpPrefix {
    pub ip_prefix: String,
    pub region: String,
    pub service: String,
    pub network_border_group: String,
}


pub struct AwsIpRange {
    url: String,
    cached_json: PathBuf,
    client: Client
}

impl AwsIpRange {

    pub fn new() -> Self {
        let url = "https://ip-ranges.amazonaws.com/ip-ranges.json".into();
        let cached_json = "ip-ranges.json".into();
        let client = Client::new();

        Self { url, cached_json, client }
    }

    pub async fn get(&self) -> Result<AwsIpRanges> {

        if self.cached_json.exists() {
            let file = File::open(&self.cached_json)?;
            let ranges: AwsIpRanges = serde_json::from_reader(file)?;

            return Ok(ranges);
        }

        let result = self.client.get(&self.url).send().await?;

        let ranges: AwsIpRanges = result.json().await?;

        Ok(ranges)
    }
}