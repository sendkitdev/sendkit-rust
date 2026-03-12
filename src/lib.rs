mod client;
mod emails;
mod error;

pub use client::SendKit;
pub use emails::{Attachment, SendEmailParams, SendEmailResponse, SendMimeEmailParams, SendMimeEmailResponse, Tag};
pub use error::{Error, ErrorResponse};
