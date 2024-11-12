use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio_tungstenite::{connect_async, tungstenite::Message};

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
            if response.r#type == "messageServer" {
                let data: MessageServerData = serde_json::from_str(&response.data.to_string())?;
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
struct ResponseMessage {
    r#type: String,
    data: Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MessageServerData {
    view_uri: String,
}
