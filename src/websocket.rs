use std::sync::Arc;

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::{
    mpsc::{self, Sender},
    Mutex,
};
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub struct WebSocketClient {
    tx: Sender<String>,
    view_uri: String,
}

impl WebSocketClient {
    pub async fn new(web_socket_url: &str) -> Result<Self> {
        let (ws_stream, _) = connect_async(web_socket_url).await?;

        let (mut write, mut read) = ws_stream.split();

        let (tx, mut rx) = mpsc::channel(100);

        write
            .send(Message::Text(serde_json::to_string(
                &InitialConnectionMessage {
                    r#type: "startWatching".to_string(),
                    data: InitialConnectionData { reconnect: false },
                },
            )?))
            .await?;

        let mut view_uri = None;
        let mut keep_interval_sec = None;

        while let Some(msg) = read.next().await {
            if let Ok(Message::Text(text)) = msg {
                let response: ResponseMessage = serde_json::from_str(&text)?;
                match response {
                    ResponseMessage::MessageServer { data } => {
                        view_uri = Some(data.view_uri);
                    }
                    ResponseMessage::Seat { data } => {
                        keep_interval_sec = Some(data.keep_interval_sec);
                    }
                    _ => (),
                }
                if view_uri.is_some() && keep_interval_sec.is_some() {
                    break;
                }
            }
        }

        let view_uri = view_uri.unwrap();
        let keep_interval_sec = keep_interval_sec.unwrap();

        let write = Arc::new(Mutex::new(write));

        let write_clone = Arc::clone(&write);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(keep_interval_sec as u64))
                    .await;
                write_clone
                    .lock()
                    .await
                    .send(Message::Text(r#"{"type":"keepSeat"}"#.to_string()))
                    .await
                    .unwrap();
            }
        });

        let write_clone = Arc::clone(&write);
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                write_clone
                    .lock()
                    .await
                    // TODO
                    .send(Message::Text(format!(
                        r#"{{"type":"postComment","data":{{"text":"{}"}}}}"#,
                        message
                    )))
                    .await
                    .unwrap();
            }
        });

        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                if let Ok(Message::Text(text)) = msg {
                    let response: ResponseMessage = serde_json::from_str(&text).unwrap();
                    match response {
                        ResponseMessage::Ping => {
                            write
                                .lock()
                                .await
                                .send(Message::Text(r#"{"type":"pong"}"#.to_string()))
                                .await
                                .unwrap();
                        }
                        ResponseMessage::Reconnect { data } => {
                            // TODO
                        }
                        _ => (),
                    }
                }
            }
        });

        Ok(Self { tx, view_uri })
    }

    pub async fn post(&self, comment: &str) -> Result<()> {
        self.tx.send(comment.to_string()).await?;
        Ok(())
    }

    pub fn view_uri(&self) -> &String {
        &self.view_uri
    }
}

pub async fn fetch_ndgr_view_uri(web_socket_url: &str) -> Result<String> {
    let (ws_stream, _) = connect_async(web_socket_url).await?;

    let (mut write, mut read) = ws_stream.split();

    write
        .send(Message::Text(serde_json::to_string(
            &InitialConnectionMessage {
                r#type: "startWatching".to_string(),
                data: InitialConnectionData { reconnect: false },
            },
        )?))
        .await?;

    while let Some(msg) = read.next().await {
        if let Ok(Message::Text(text)) = msg {
            let response: ResponseMessage = serde_json::from_str(&text)?;
            if let ResponseMessage::MessageServer { data } = response {
                return Ok(data.view_uri);
            }
        }
    }

    Err(anyhow::anyhow!("view uri not found"))
}

#[derive(Debug, Serialize)]
struct InitialConnectionMessage {
    r#type: String,
    data: InitialConnectionData,
}

#[derive(Debug, Serialize)]
struct InitialConnectionData {
    reconnect: bool,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
enum ResponseMessage {
    MessageServer { data: MessageServerData },
    Seat { data: SeatData },
    Ping,
    Reconnect { data: ReconnectData },
    ServerTime,
    Stream,
    Schedule,
    Statistics,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MessageServerData {
    view_uri: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SeatData {
    keep_interval_sec: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReconnectData {
    audience_token: String,
    wait_time_sec: i64,
}
