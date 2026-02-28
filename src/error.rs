use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    pub name: String,
    pub message: String,
    #[serde(rename = "statusCode")]
    pub status_code: Option<u16>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("sendkit: {0}")]
    Api(ErrorResponse),

    #[error("sendkit: {0}")]
    Http(#[from] reqwest::Error),

    #[error("sendkit: missing API key")]
    MissingApiKey,
}

impl std::fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({}): {}",
            self.name,
            self.status_code.unwrap_or(0),
            self.message
        )
    }
}
