# SendKit Rust SDK

## Project Overview

Rust SDK for the SendKit email API. Uses reqwest for HTTP, serde for JSON.

## Architecture

```
src/
├── lib.rs      # Public exports
├── client.rs   # SendKit struct: holds reqwest Client, post() method
├── emails.rs   # Emails struct (send, send_mime) + param/response types
└── error.rs    # Error enum + ErrorResponse
```

- `SendKit::new()` creates client with API key
- `client.emails.send(&client, &params)` for structured emails
- `client.emails.send_mime(&client, &params)` for MIME emails
- All methods are async, return `Result<T, Error>`
- Uses `thiserror` for error handling
- `POST /v1/emails` for structured emails, `POST /v1/emails/mime` for raw MIME

## Testing

- Tests use `wiremock` for mock HTTP servers
- Run tests: `cargo test`
- Integration tests in `tests/integration_test.rs`

## Releasing

- Tags use numeric format: `1.0.0` (no `v` prefix)
- CI runs tests on stable Rust
- Pushing a tag creates GitHub Release + publishes to crates.io

## Git

- NEVER add `Co-Authored-By` lines to commit messages
