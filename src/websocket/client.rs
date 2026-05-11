use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};

use crate::auth::ApprovalKey;
use crate::error::{Error, Result};
use crate::websocket::{IncomingFrame, Subscription, SubscriptionBook, SubscriptionMessage};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WebSocketClient {
    url: String,
}

impl WebSocketClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            url: base_url.into(),
        }
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub async fn connect(&self) -> Result<WebSocketSession> {
        let (stream, _) = connect_async(&self.url)
            .await
            .map_err(|error| Error::http(format!("websocket connect failed: {error}")))?;

        Ok(WebSocketSession { stream })
    }

    pub async fn connect_with_subscriptions(
        &self,
        approval_key: &ApprovalKey,
        subscriptions: &SubscriptionBook,
    ) -> Result<WebSocketSession> {
        let mut session = self.connect().await?;

        for message in subscriptions.messages(approval_key) {
            session.send_message(&message).await?;
        }

        Ok(session)
    }
}

pub struct WebSocketSession {
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl WebSocketSession {
    pub async fn send_subscription(
        &mut self,
        approval_key: &ApprovalKey,
        subscription: Subscription,
    ) -> Result<()> {
        let message = SubscriptionMessage::new(approval_key, subscription);

        self.send_message(&message).await
    }

    pub async fn send_message(&mut self, message: &SubscriptionMessage) -> Result<()> {
        let payload = serde_json::to_string(message).map_err(|error| {
            Error::parse(format!(
                "failed to serialize websocket subscription message: {error}"
            ))
        })?;

        self.stream
            .send(Message::Text(payload.into()))
            .await
            .map_err(|error| Error::http(format!("websocket send failed: {error}")))
    }

    pub async fn send_pong_text(&mut self, payload: &str) -> Result<()> {
        self.stream
            .send(Message::Pong(payload.as_bytes().to_vec().into()))
            .await
            .map_err(|error| Error::http(format!("websocket pong failed: {error}")))
    }

    pub async fn next_frame(&mut self) -> Result<Option<IncomingFrame>> {
        while let Some(message) = self.stream.next().await {
            let message = message
                .map_err(|error| Error::http(format!("websocket receive failed: {error}")))?;

            match message {
                Message::Text(text) => {
                    return IncomingFrame::parse(&text).map(Some);
                }
                Message::Ping(payload) => {
                    self.stream
                        .send(Message::Pong(payload))
                        .await
                        .map_err(|error| Error::http(format!("websocket pong failed: {error}")))?;
                }
                Message::Close(_) => return Ok(None),
                Message::Pong(_) => {}
                Message::Binary(_) => {
                    return Err(Error::parse("websocket binary frame is not supported"));
                }
                Message::Frame(_) => {}
            }
        }

        Ok(None)
    }

    pub async fn close(mut self) -> Result<()> {
        self.stream
            .close(None)
            .await
            .map_err(|error| Error::http(format!("websocket close failed: {error}")))
    }
}
