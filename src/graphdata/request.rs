use async_trait::async_trait;
use reqwest::Client;
use std::io::Cursor;

#[derive(Debug, Clone)]
pub struct ImplUpgradePathInterface {}

#[async_trait]
#[allow(unused)]
pub trait UpgradePathInterface {
    // used to interact with cincinnati api
    async fn get_graphdata(&self, url: String) -> Result<String, Box<dyn std::error::Error>>;
    async fn get_graph_tar_gz(
        &self,
        url: String,
        file_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;
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

    async fn get_graph_tar_gz(
        &self,
        url: String,
        file_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let response = reqwest::get(url).await?;
        let mut file = std::fs::File::create(file_name)?;
        let mut content = Cursor::new(response.bytes().await?);
        std::io::copy(&mut content, &mut file)?;
        Ok(())
    }
}
