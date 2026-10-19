use std::path::PathBuf;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use protocol::{CloudEvent, types};
use serde_json::json;
use tokio::net::UnixStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::HeaderValue;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let socket: PathBuf = std::env::args()
        .nth(1)
        .or_else(|| std::env::var("OFFICE_SOCKET").ok())
        .unwrap_or_else(|| "/tmp/office.sock".to_owned())
        .into();

    let mut request = "ws://localhost/office".into_client_request()?;
    request.headers_mut().insert(
        "Sec-WebSocket-Protocol",
        HeaderValue::from_static(types::SUBPROTOCOL),
    );

    let stream = UnixStream::connect(&socket).await?;
    let (mut websocket, _) = tokio_tungstenite::client_async(request, stream).await?;
    println!("connected to {}", socket.display());

    for (id, type_, data) in [
        (
            "smoke-sub",
            types::command::SUBSCRIPTION_ADD,
            json!({ "filter": {} }),
        ),
        ("smoke-tools", types::command::TOOL_LIST, json!({})),
        ("smoke-models", types::command::MODEL_LIST, json!({})),
        (
            "smoke-workspaces",
            types::command::WORKSPACE_LIST,
            json!({}),
        ),
    ] {
        send(
            &mut websocket,
            CloudEvent::new(id, "/office/client/smoke", type_, data),
        )
        .await;
    }

    let deadline = tokio::time::Instant::now() + Duration::from_secs(3);
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break;
        }
        match tokio::time::timeout(remaining, websocket.next()).await {
            Ok(Some(Ok(Message::Text(text)))) => {
                print_event(&text);
            }
            Ok(Some(Ok(Message::Binary(bytes)))) => {
                print_event(std::str::from_utf8(&bytes)?);
            }
            Ok(Some(Ok(_))) => {}
            Ok(Some(Err(error))) => return Err(error.into()),
            Ok(None) | Err(_) => break,
        }
    }

    Ok(())
}

async fn send(websocket: &mut tokio_tungstenite::WebSocketStream<UnixStream>, event: CloudEvent) {
    let json = event.to_json().expect("event serializes");
    websocket.send(Message::text(json)).await.expect("send");
}

fn print_event(raw: &str) {
    match CloudEvent::from_json(raw) {
        Ok(event) => println!(
            "  seq={:<4} type={:<45} subject={} data={}",
            event
                .sequence
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".to_owned()),
            event.type_,
            event.subject.as_deref().unwrap_or("-"),
            event.data,
        ),
        Err(error) => println!("  <unparseable frame: {error}>"),
    }
}
