use std::collections::HashMap;

use sendkit::{Attachment, Error, SendEmailParams, SendKit, SendMimeEmailParams, Tag};
use wiremock::matchers::{body_partial_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[test]
fn test_new_client_with_api_key() {
    let client = SendKit::new("sk_test_123");
    assert!(client.is_ok());
}

#[test]
fn test_env_variable_fallback() {
    // Run env-dependent tests sequentially in one function to avoid races
    // with parallel tests (env vars are process-global).

    // 1. Missing API key when env is unset
    std::env::remove_var("SENDKIT_API_KEY");
    let client = SendKit::new("");
    assert!(matches!(client, Err(Error::MissingApiKey)));

    // 2. Fallback to env variable when key is empty
    std::env::set_var("SENDKIT_API_KEY", "sk_from_env");
    let client = SendKit::new("");
    assert!(client.is_ok());

    // Cleanup
    std::env::remove_var("SENDKIT_API_KEY");
}

#[test]
fn test_custom_base_url() {
    let client = SendKit::with_base_url("sk_test_123", "https://custom.api.com");
    assert!(client.is_ok());
}

#[tokio::test]
async fn test_send_email() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/emails"))
        .and(header("Authorization", "Bearer sk_test_123"))
        .and(body_partial_json(serde_json::json!({
            "from": "sender@example.com",
            "to": ["recipient@example.com"],
            "subject": "Test Email",
            "html": "<p>Hello</p>"
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": "email-uuid-123"})),
        )
        .mount(&server)
        .await;

    let client = SendKit::with_base_url("sk_test_123", &server.uri()).unwrap();
    let result = client
        .emails
        .send(
            &client,
            &SendEmailParams {
                from: "sender@example.com".into(),
                to: vec!["recipient@example.com".into()],
                subject: "Test Email".into(),
                html: Some("<p>Hello</p>".into()),
                text: None,
                cc: None,
                bcc: None,
                reply_to: None,
                headers: None,
                tags: None,
                scheduled_at: None,
                attachments: None,
            },
        )
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().id, "email-uuid-123");
}

#[test]
fn test_send_email_params_new() {
    let params = SendEmailParams::new("sender@example.com", "recipient@example.com", "Hello");

    assert_eq!(params.from, "sender@example.com");
    assert_eq!(params.to, vec!["recipient@example.com"]);
    assert_eq!(params.subject, "Hello");
    assert!(params.html.is_none());
    assert!(params.text.is_none());
}

#[test]
fn test_send_email_params_with_reply_to() {
    let params = SendEmailParams::new("from@example.com", "to@example.com", "Hello")
        .with_reply_to("reply@example.com");
    assert_eq!(params.reply_to, Some(vec!["reply@example.com".to_string()]));
}

#[test]
fn test_send_email_params_new_display_name() {
    let params = SendEmailParams::new(
        "Support <sender@example.com>",
        "Bob <recipient@example.com>",
        "Hello",
    );

    assert_eq!(params.from, "Support <sender@example.com>");
    assert_eq!(params.to, vec!["Bob <recipient@example.com>"]);
}

#[tokio::test]
async fn test_send_with_optional_fields() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/emails"))
        .and(body_partial_json(serde_json::json!({
            "from": "sender@example.com",
            "to": ["recipient@example.com"],
            "subject": "Test",
            "html": "<p>Hi</p>",
            "reply_to": ["reply@example.com"],
            "scheduled_at": "2026-03-01T10:00:00Z"
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": "email-uuid-456"})),
        )
        .mount(&server)
        .await;

    let client = SendKit::with_base_url("sk_test_123", &server.uri()).unwrap();
    let result = client
        .emails
        .send(
            &client,
            &SendEmailParams {
                from: "sender@example.com".into(),
                to: vec!["recipient@example.com".into()],
                subject: "Test".into(),
                html: Some("<p>Hi</p>".into()),
                text: None,
                cc: None,
                bcc: None,
                reply_to: Some(vec!["reply@example.com".into()]),
                headers: None,
                tags: None,
                scheduled_at: Some("2026-03-01T10:00:00Z".into()),
                attachments: None,
            },
        )
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_send_mime_email() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/emails/mime"))
        .and(body_partial_json(serde_json::json!({
            "envelope_from": "sender@example.com",
            "envelope_to": "recipient@example.com",
            "raw_message": "From: sender@example.com\r\nTo: recipient@example.com\r\n\r\nHello"
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": "mime-uuid-789"})),
        )
        .mount(&server)
        .await;

    let client = SendKit::with_base_url("sk_test_123", &server.uri()).unwrap();
    let result = client
        .emails
        .send_mime(
            &client,
            &SendMimeEmailParams {
                envelope_from: "sender@example.com".into(),
                envelope_to: "recipient@example.com".into(),
                raw_message: "From: sender@example.com\r\nTo: recipient@example.com\r\n\r\nHello"
                    .into(),
            },
        )
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().id, "mime-uuid-789");
}

#[tokio::test]
async fn test_api_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/emails"))
        .and(body_partial_json(serde_json::json!({
            "from": "sender@example.com",
            "to": [],
            "subject": "Test"
        })))
        .respond_with(ResponseTemplate::new(422).set_body_json(serde_json::json!({
            "name": "validation_error",
            "message": "The to field is required.",
            "statusCode": 422
        })))
        .mount(&server)
        .await;

    let client = SendKit::with_base_url("sk_test_123", &server.uri()).unwrap();
    let result = client
        .emails
        .send(
            &client,
            &SendEmailParams {
                from: "sender@example.com".into(),
                to: vec![],
                subject: "Test".into(),
                html: Some("<p>Hi</p>".into()),
                text: None,
                cc: None,
                bcc: None,
                reply_to: None,
                headers: None,
                tags: None,
                scheduled_at: None,
                attachments: None,
            },
        )
        .await;

    assert!(result.is_err());
    if let Err(Error::Api(err)) = result {
        assert_eq!(err.name, "validation_error");
        assert_eq!(err.status_code, Some(422));
        assert_eq!(err.message, "The to field is required.");
    } else {
        panic!("Expected Api error");
    }
}

#[tokio::test]
async fn test_send_email_multiple_recipients() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/emails"))
        .and(header("Authorization", "Bearer sk_test_123"))
        .and(body_partial_json(serde_json::json!({
            "from": "sender@example.com",
            "to": ["alice@example.com", "bob@example.com", "charlie@example.com"],
            "subject": "Team Update"
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": "email-multi-001"})),
        )
        .mount(&server)
        .await;

    let client = SendKit::with_base_url("sk_test_123", &server.uri()).unwrap();
    let result = client
        .emails
        .send(
            &client,
            &SendEmailParams {
                from: "sender@example.com".into(),
                to: vec![
                    "alice@example.com".into(),
                    "bob@example.com".into(),
                    "charlie@example.com".into(),
                ],
                subject: "Team Update".into(),
                html: Some("<p>Hello team</p>".into()),
                text: None,
                cc: None,
                bcc: None,
                reply_to: None,
                headers: None,
                tags: None,
                scheduled_at: None,
                attachments: None,
            },
        )
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().id, "email-multi-001");
}

#[tokio::test]
async fn test_send_email_with_cc_and_bcc() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/emails"))
        .and(header("Authorization", "Bearer sk_test_123"))
        .and(body_partial_json(serde_json::json!({
            "from": "sender@example.com",
            "to": ["recipient@example.com"],
            "subject": "CC/BCC Test",
            "cc": ["cc1@example.com", "cc2@example.com"],
            "bcc": ["bcc1@example.com"]
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": "email-ccbcc-002"})),
        )
        .mount(&server)
        .await;

    let client = SendKit::with_base_url("sk_test_123", &server.uri()).unwrap();
    let result = client
        .emails
        .send(
            &client,
            &SendEmailParams {
                from: "sender@example.com".into(),
                to: vec!["recipient@example.com".into()],
                subject: "CC/BCC Test".into(),
                html: Some("<p>Hello</p>".into()),
                text: None,
                cc: Some(vec!["cc1@example.com".into(), "cc2@example.com".into()]),
                bcc: Some(vec!["bcc1@example.com".into()]),
                reply_to: None,
                headers: None,
                tags: None,
                scheduled_at: None,
                attachments: None,
            },
        )
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().id, "email-ccbcc-002");
}

#[tokio::test]
async fn test_send_email_with_attachments() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/emails"))
        .and(header("Authorization", "Bearer sk_test_123"))
        .and(body_partial_json(serde_json::json!({
            "from": "sender@example.com",
            "to": ["recipient@example.com"],
            "subject": "With Attachments",
            "attachments": [
                {
                    "filename": "report.pdf",
                    "content": "base64encodedcontent",
                    "content_type": "application/pdf"
                },
                {
                    "filename": "notes.txt",
                    "content": "cGxhaW4gdGV4dA=="
                }
            ]
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": "email-attach-003"})),
        )
        .mount(&server)
        .await;

    let client = SendKit::with_base_url("sk_test_123", &server.uri()).unwrap();
    let result = client
        .emails
        .send(
            &client,
            &SendEmailParams {
                from: "sender@example.com".into(),
                to: vec!["recipient@example.com".into()],
                subject: "With Attachments".into(),
                html: Some("<p>See attached</p>".into()),
                text: None,
                cc: None,
                bcc: None,
                reply_to: None,
                headers: None,
                tags: None,
                scheduled_at: None,
                attachments: Some(vec![
                    Attachment {
                        filename: "report.pdf".into(),
                        content: "base64encodedcontent".into(),
                        content_type: Some("application/pdf".into()),
                    },
                    Attachment {
                        filename: "notes.txt".into(),
                        content: "cGxhaW4gdGV4dA==".into(),
                        content_type: None,
                    },
                ]),
            },
        )
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().id, "email-attach-003");
}

#[tokio::test]
async fn test_send_email_with_tags() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/emails"))
        .and(header("Authorization", "Bearer sk_test_123"))
        .and(body_partial_json(serde_json::json!({
            "from": "sender@example.com",
            "to": ["recipient@example.com"],
            "subject": "Tagged Email",
            "tags": [
                {"name": "category", "value": "welcome"},
                {"name": "campaign", "value": "onboarding"}
            ]
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": "email-tags-004"})),
        )
        .mount(&server)
        .await;

    let client = SendKit::with_base_url("sk_test_123", &server.uri()).unwrap();
    let result = client
        .emails
        .send(
            &client,
            &SendEmailParams {
                from: "sender@example.com".into(),
                to: vec!["recipient@example.com".into()],
                subject: "Tagged Email".into(),
                html: Some("<p>Welcome!</p>".into()),
                text: None,
                cc: None,
                bcc: None,
                reply_to: None,
                headers: None,
                tags: Some(vec![
                    Tag { name: "category".into(), value: "welcome".into() },
                    Tag { name: "campaign".into(), value: "onboarding".into() },
                ]),
                scheduled_at: None,
                attachments: None,
            },
        )
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().id, "email-tags-004");
}

#[tokio::test]
async fn test_send_email_with_text() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/emails"))
        .and(header("Authorization", "Bearer sk_test_123"))
        .and(body_partial_json(serde_json::json!({
            "from": "sender@example.com",
            "to": ["recipient@example.com"],
            "subject": "Text Email",
            "text": "Plain text content"
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": "email-text-005"})),
        )
        .mount(&server)
        .await;

    let client = SendKit::with_base_url("sk_test_123", &server.uri()).unwrap();
    let result = client
        .emails
        .send(
            &client,
            &SendEmailParams {
                from: "sender@example.com".into(),
                to: vec!["recipient@example.com".into()],
                subject: "Text Email".into(),
                html: None,
                text: Some("Plain text content".into()),
                cc: None,
                bcc: None,
                reply_to: None,
                headers: None,
                tags: None,
                scheduled_at: None,
                attachments: None,
            },
        )
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().id, "email-text-005");
}

#[tokio::test]
async fn test_send_email_with_headers() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/emails"))
        .and(header("Authorization", "Bearer sk_test_123"))
        .and(body_partial_json(serde_json::json!({
            "from": "sender@example.com",
            "to": ["recipient@example.com"],
            "subject": "Headers Email",
            "headers": {
                "X-Custom": "value"
            }
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"id": "email-headers-006"})),
        )
        .mount(&server)
        .await;

    let client = SendKit::with_base_url("sk_test_123", &server.uri()).unwrap();
    let result = client
        .emails
        .send(
            &client,
            &SendEmailParams {
                from: "sender@example.com".into(),
                to: vec!["recipient@example.com".into()],
                subject: "Headers Email".into(),
                html: Some("<p>Hello</p>".into()),
                text: None,
                cc: None,
                bcc: None,
                reply_to: None,
                headers: Some(HashMap::from([("X-Custom".into(), "value".into())])),
                tags: None,
                scheduled_at: None,
                attachments: None,
            },
        )
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().id, "email-headers-006");
}

#[tokio::test]
async fn test_send_email_null_fields_omitted() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/emails"))
        .and(header("Authorization", "Bearer sk_test_123"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"id": "email-minimal-007"})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = SendKit::with_base_url("sk_test_123", &server.uri()).unwrap();
    let result = client
        .emails
        .send(
            &client,
            &SendEmailParams {
                from: "sender@example.com".into(),
                to: vec!["recipient@example.com".into()],
                subject: "Minimal Email".into(),
                html: Some("<p>Hello</p>".into()),
                text: None,
                cc: None,
                bcc: None,
                reply_to: None,
                headers: None,
                tags: None,
                scheduled_at: None,
                attachments: None,
            },
        )
        .await;

    assert!(result.is_ok());

    // Verify the serialized JSON does not contain optional None fields
    let serialized = serde_json::to_value(&SendEmailParams {
        from: "sender@example.com".into(),
        to: vec!["recipient@example.com".into()],
        subject: "Minimal Email".into(),
        html: Some("<p>Hello</p>".into()),
        text: None,
        cc: None,
        bcc: None,
        reply_to: None,
        headers: None,
        tags: None,
        scheduled_at: None,
        attachments: None,
    })
    .unwrap();

    let obj = serialized.as_object().unwrap();
    let omitted_keys = [
        "text",
        "cc",
        "bcc",
        "reply_to",
        "headers",
        "tags",
        "scheduled_at",
        "attachments",
    ];
    for key in omitted_keys {
        assert!(
            !obj.contains_key(key),
            "Expected key '{}' to be omitted from JSON, but it was present",
            key
        );
    }
}

#[test]
fn test_send_email_params_with_cc() {
    let params = SendEmailParams::new("from@example.com", "to@example.com", "Hello")
        .with_cc("cc@example.com");
    assert_eq!(params.cc, Some(vec!["cc@example.com".to_string()]));
}

#[test]
fn test_send_email_params_with_bcc() {
    let params = SendEmailParams::new("from@example.com", "to@example.com", "Hello")
        .with_bcc("bcc@example.com");
    assert_eq!(params.bcc, Some(vec!["bcc@example.com".to_string()]));
}

#[tokio::test]
async fn test_send_email_with_cc_bcc_builder_serialization() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/emails"))
        .and(header("Authorization", "Bearer sk_test_123"))
        .and(body_partial_json(serde_json::json!({
            "from": "sender@example.com",
            "to": ["recipient@example.com"],
            "subject": "Builder CC/BCC",
            "cc": ["cc@example.com"],
            "bcc": ["bcc@example.com"]
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"id": "email-builder-cc-bcc"})),
        )
        .mount(&server)
        .await;

    let client = SendKit::with_base_url("sk_test_123", &server.uri()).unwrap();
    let params = SendEmailParams::new("sender@example.com", "recipient@example.com", "Builder CC/BCC")
        .with_cc("cc@example.com")
        .with_bcc("bcc@example.com");
    let mut params = params;
    params.html = Some("<p>Hello</p>".into());

    let result = client.emails.send(&client, &params).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().id, "email-builder-cc-bcc");
}
