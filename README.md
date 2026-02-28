# SendKit Rust SDK

Official Rust SDK for the [SendKit](https://sendkit.com) email API.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
sendkit = "1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

## Usage

### Create a Client

```rust
use sendkit::SendKit;

let client = SendKit::new("sk_your_api_key").unwrap();
```

### Send an Email

```rust
use sendkit::{SendKit, SendEmailParams};

#[tokio::main]
async fn main() {
    let client = SendKit::new("sk_your_api_key").unwrap();

    let response = client.emails.send(&client, &SendEmailParams {
        from: "you@example.com".into(),
        to: vec!["recipient@example.com".into()],
        subject: "Hello from SendKit".into(),
        html: Some("<h1>Welcome!</h1>".into()),
        ..Default::default()
    }).await.unwrap();

    println!("{}", response.id);
}
```

### Send a MIME Email

```rust
use sendkit::{SendKit, SendMimeEmailParams};

let response = client.emails.send_mime(&client, &SendMimeEmailParams {
    envelope_from: "you@example.com".into(),
    envelope_to: "recipient@example.com".into(),
    raw_message: mime_string,
}).await.unwrap();
```

### Error Handling

```rust
use sendkit::{SendKit, Error};

match client.emails.send(&client, &params).await {
    Ok(response) => println!("Sent: {}", response.id),
    Err(Error::Api(err)) => {
        println!("API error: {} ({})", err.name, err.status_code.unwrap_or(0));
        println!("Message: {}", err.message);
    }
    Err(err) => println!("Error: {}", err),
}
```

### Configuration

```rust
// Read API key from SENDKIT_API_KEY environment variable
let client = SendKit::new("").unwrap();

// Custom base URL
let client = SendKit::with_base_url("sk_...", "https://custom.api.com").unwrap();
```
