use super::client::SocketEvent;
use crate::socket::client::Client;
use bevy::log::prelude::*;
use ezsockets::ClientConfig;
use std::env;
use std::future::Future;
use tokio::sync::mpsc;
use url::Url;

const DEFAULT_URL: &str = "wss://chat.haunted.host";
const DEV_URL: &str = "ws://localhost:4000";

pub fn create_channel() -> (mpsc::Sender<SocketEvent>, mpsc::Receiver<SocketEvent>) {
    mpsc::channel::<SocketEvent>(32)
}

pub async fn connect_socket(
    tx: mpsc::Sender<SocketEvent>,
) -> (
    ezsockets::Client<Client>,
    impl Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>>,
) {
    let socket_url = get_socket_url();
    info!("connecting to {} ...", socket_url);

    let config = ClientConfig::new(socket_url.clone());
    ezsockets::connect(|handle| Client::new(handle, tx), config).await
}

#[allow(dead_code)]
pub fn close_socket(handle: ezsockets::Client<Client>) -> std::io::Result<()> {
    info!("closing websocket");

    match handle.close(None) {
        Ok(_) => {
            info!("websocket closed");
            Ok(())
        }
        Err(e) => {
            error!("failed to close socket={:?}", e);
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("failed to close socket={:?}", e),
            ))
        }
    }
}

pub fn get_socket_url() -> Url {
    let base_url = if env::var("DEV").unwrap_or_default() == "true" {
        DEV_URL.to_string()
    } else if let Ok(custom_url) = env::var("URL") {
        custom_url
    } else {
        DEFAULT_URL.to_string()
    };

    let mut url = Url::parse(&base_url).expect("Error parsing URL");

    url.set_path("/socket/websocket");
    url.set_query(Some("vsn=2.0.0"));

    // default to secure wss if not specified
    match url.scheme() {
        "ws" | "wss" => (),
        _ => url.set_scheme("wss").unwrap(),
    }

    url
}
