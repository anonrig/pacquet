use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};

use crate::{
    error::RegistryError,
    package::{Package, PackageVersion},
};

pub struct HttpClient {
    client: ClientWithMiddleware,
    cache: elsa::FrozenMap<String, Box<Package>>,
    registry: String,
}

impl HttpClient {
    pub fn new(registry: &str) -> Self {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
        let client = ClientBuilder::new(reqwest::Client::new())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        HttpClient { client, cache: elsa::FrozenMap::new(), registry: registry.to_string() }
    }

    pub async fn get_package(&self, name: &str) -> Result<&Package, RegistryError> {
        if let Some(package) = &self.cache.get(name) {
            return Ok(package);
        }

        let package: Package = self
            .client
            .get(format!("{0}{name}", &self.registry))
            .header("user-agent", "pacquet-cli")
            .header("content-type", "application/json")
            .send()
            .await?
            .json::<Package>()
            .await?;

        let package = self.cache.insert(name.to_string(), Box::new(package));

        Ok(package)
    }

    pub async fn get_package_by_version(
        &self,
        name: &str,
        version: &str,
    ) -> Result<PackageVersion, RegistryError> {
        Ok(self
            .client
            .get(format!("{0}{name}/{version}", &self.registry))
            .header("user-agent", "pacquet-cli")
            .header("content-type", "application/json")
            .send()
            .await?
            .json::<PackageVersion>()
            .await?)
    }
}
