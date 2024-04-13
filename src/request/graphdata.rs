use async_trait::async_trait;
use reqwest::Client;

#[derive(Debug, Clone)]
pub struct ImplUpgradePathInterface {}

#[async_trait]
pub trait UpgradePathInterface {
    // used to interact with container registry (manifest calls)
    async fn get_graphdata(&self, url: String) -> Result<String, Box<dyn std::error::Error>>;
}

#[async_trait]
impl UpgradePathInterface for ImplUpgradePathInterface {
    async fn get_graphdata(&self, url: String) -> Result<String, Box<dyn std::error::Error>> {
        let client = Client::new();
        // check without token
        let body = client
            .get(url)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .send()
            .await?
            .text()
            .await?;

        Ok(body)
    }
}