use super::{refs::Refs, response::Response};
use crate::socket::request::Request;
use async_trait::async_trait;
use bevy::log::prelude::*;
use ezsockets::{client::ClientCloseMode, CloseFrame, Error as SocketError, WSError};
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::mpsc;

/// This module contains the `Client` struct and ezsockets client implementation.
/// It handles internal calls and relays messages to the server.

#[derive(Debug)]
pub struct Client {
    pub handle: ezsockets::Client<Self>,
    pub tx: mpsc::Sender<SocketEvent>,
    refs: Refs,
}

impl Client {
    pub fn new(handle: ezsockets::Client<Self>, tx: mpsc::Sender<SocketEvent>) -> Self {
        Self {
            handle,
            tx,
            refs: Refs::default(),
        }
    }

    pub fn next_refs(&self) -> Refs {
        let new_message_ref = self.refs.message_ref.fetch_add(1, Ordering::SeqCst);

        Refs {
            join_ref: self.refs.join_ref.clone(),
            message_ref: AtomicUsize::new(new_message_ref + 1),
        }
    }
}

#[derive(Debug)]
pub enum SocketEvent {
    Close,
    Connect,
    ConnectFail,
    Disconnect,
    Response(Response),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum SocketStatus {
    #[default]
    Closed,
    Connected,
    ConnectFailed,
    Disconnected,
}

#[async_trait]
impl ezsockets::ClientExt for Client {
    type Call = Request;

    async fn on_text(&mut self, text: String) -> Result<(), SocketError> {
        // Relay message from server to channel
        let response = SocketEvent::Response(Response::new_from_json_string(&text));
        if let Err(e) = self.tx.send(response).await {
            error!("error sending message to channel: {e}");
        }

        Ok(())
    }

    async fn on_binary(&mut self, bytes: Vec<u8>) -> Result<(), SocketError> {
        debug!("on_binary={bytes:?}");
        Ok(())
    }

    async fn on_call(&mut self, request: Request) -> Result<(), SocketError> {
        debug!("on_call={:?}", request);

        let request_payload = request.to_payload(self.next_refs());
        debug!("sending request: {request_payload}");

        self.handle
            .text(request_payload)
            .expect("error sending request");

        Ok(())
    }

    async fn on_connect(&mut self) -> Result<(), SocketError> {
        debug!("on_connect");

        if let Err(e) = self.tx.send(SocketEvent::Connect).await {
            error!("error sending message to channel: {e}");
        }

        Ok(())
    }

    async fn on_connect_fail(&mut self, _error: WSError) -> Result<ClientCloseMode, SocketError> {
        error!("on_connect_fail");

        if let Err(e) = self.tx.send(SocketEvent::ConnectFail).await {
            error!("error sending message to channel: {e}");
        }

        Ok(ClientCloseMode::Reconnect)
    }

    async fn on_close(
        &mut self,
        _frame: Option<CloseFrame>,
    ) -> Result<ClientCloseMode, SocketError> {
        error!("on_close");

        if let Err(e) = self.tx.send(SocketEvent::Close).await {
            error!("error sending message to channel: {e}");
        }

        Ok(ClientCloseMode::Reconnect)
    }

    async fn on_disconnect(&mut self) -> Result<ClientCloseMode, SocketError> {
        error!("on_disconnect");

        if let Err(e) = self.tx.send(SocketEvent::Disconnect).await {
            error!("error sending message to channel: {e}");
        }

        Ok(ClientCloseMode::Reconnect)
    }
}
