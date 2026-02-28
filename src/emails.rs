use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::client::SendKit;
use crate::error::Error;

pub struct Emails;

#[derive(Debug, Serialize)]
pub struct Attachment {
    pub filename: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct SendEmailParams {
    pub from: String,
    pub to: Vec<String>,
    pub subject: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cc: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bcc: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduled_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Attachment>>,
}

#[derive(Debug, Deserialize)]
pub struct SendEmailResponse {
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct SendMimeEmailParams {
    pub envelope_from: String,
    pub envelope_to: String,
    pub raw_message: String,
}

#[derive(Debug, Deserialize)]
pub struct SendMimeEmailResponse {
    pub id: String,
}

impl Emails {
    /// Send a structured email.
    pub async fn send(&self, client: &SendKit, params: &SendEmailParams) -> Result<SendEmailResponse, Error> {
        client.post("/v1/emails", params).await
    }

    /// Send a raw MIME email.
    pub async fn send_mime(&self, client: &SendKit, params: &SendMimeEmailParams) -> Result<SendMimeEmailResponse, Error> {
        client.post("/v1/emails/mime", params).await
    }
}
