use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::emails::Emails;
use crate::error::{Error, ErrorResponse};

const DEFAULT_BASE_URL: &str = "https://api.sendkit.com";

pub struct SendKit {
    pub(crate) client: Client,
    pub(crate) base_url: String,
    pub(crate) api_key: String,

    /// Access the emails API.
    pub emails: Emails,
}

impl SendKit {
    /// Create a new SendKit client.
    ///
    /// If `api_key` is empty, reads from the `SENDKIT_API_KEY` environment variable.
    pub fn new(api_key: &str) -> Result<Self, Error> {
        let key = if api_key.is_empty() {
            std::env::var("SENDKIT_API_KEY").unwrap_or_default()
        } else {
            api_key.to_string()
        };

        if key.is_empty() {
            return Err(Error::MissingApiKey);
        }

        Ok(Self {
            client: Client::new(),
            base_url: DEFAULT_BASE_URL.to_string(),
            api_key: key,
            emails: Emails,
        })
    }

    /// Create a new SendKit client with a custom base URL.
    pub fn with_base_url(api_key: &str, base_url: &str) -> Result<Self, Error> {
        let mut client = Self::new(api_key)?;
        client.base_url = base_url.to_string();
        Ok(client)
    }

    pub(crate) async fn post<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, Error> {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: ErrorResponse = response.json().await.unwrap_or(ErrorResponse {
                name: "application_error".to_string(),
                message: "Unknown error".to_string(),
                status_code: None,
            });
            return Err(Error::Api(error));
        }

        Ok(response.json().await?)
    }
}
