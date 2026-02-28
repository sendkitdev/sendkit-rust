use sendkit::{Error, SendEmailParams, SendKit, SendMimeEmailParams};
use wiremock::matchers::{header, method, path};
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
        .and(path("/v1/emails"))
        .and(header("Authorization", "Bearer sk_test_123"))
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

#[tokio::test]
async fn test_send_with_optional_fields() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/emails"))
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
                reply_to: Some("reply@example.com".into()),
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
        .and(path("/v1/emails/mime"))
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
        .and(path("/v1/emails"))
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
