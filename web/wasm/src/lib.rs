use futures_util::{StreamExt, pin_mut};
use ndgr_client::{ViewQuery, fetch_chunked_entry, fetch_chunked_message, fetch_program_info};
use protobuf::chat::data::nicolive_message::Data;
use protobuf::chat::data::{nicoad, simple_notification};
use protobuf::chat::service::edge::ChunkedMessage;
use protobuf::chat::service::edge::chunked_entry::Entry;
use protobuf::chat::service::edge::chunked_message::Payload;
use wasm_bindgen::prelude::*;

fn to_js_err(e: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&e.to_string())
}

fn proxied(proxy_prefix: &str, url: &str) -> String {
    format!("{proxy_prefix}{url}")
}

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

/// 番組ページの HTML から WebSocket URL を取り出す。
/// `proxy_prefix` は CORS 回避用プロキシの前置文字列(不要なら空文字)。
#[wasm_bindgen]
pub async fn fetch_web_socket_url(
    page_url: String,
    proxy_prefix: String,
) -> Result<String, JsValue> {
    let info = fetch_program_info(&proxied(&proxy_prefix, &page_url))
        .await
        .map_err(to_js_err)?;
    Ok(info.site.relive.web_socket_url)
}

/// NDGR メッセージサーバーからコメントをストリーミングし、
/// 1 件ごとに JSON 文字列で `on_message` を呼ぶ。
/// コールバックが `false` を返したら停止する。
#[wasm_bindgen]
pub async fn stream_comments(
    view_uri: String,
    proxy_prefix: String,
    on_message: js_sys::Function,
) -> Result<(), JsValue> {
    let mut view_query = ViewQuery::Now;

    loop {
        let mut got_next = false;

        let stream = fetch_chunked_entry(&proxied(&proxy_prefix, &view_uri), &view_query).await;
        pin_mut!(stream);

        while let Some(entry) = stream.next().await {
            let entry = entry.map_err(to_js_err)?;
            match entry.entry {
                Some(Entry::Next(next)) => {
                    view_query = ViewQuery::At(next.at);
                    got_next = true;
                }
                Some(Entry::Segment(segment)) => {
                    let messages =
                        fetch_chunked_message(&proxied(&proxy_prefix, &segment.uri)).await;
                    pin_mut!(messages);

                    while let Some(message) = messages.next().await {
                        let message = message.map_err(to_js_err)?;
                        if let Some(json) = chunked_message_to_json(&message) {
                            let keep_going = on_message
                                .call1(&JsValue::NULL, &JsValue::from_str(&json.to_string()))?;
                            if keep_going.is_falsy() {
                                return Ok(());
                            }
                        }
                    }
                }
                _ => (),
            }
        }

        if !got_next {
            return Err(JsValue::from_str(
                "stream ended (program may have finished)",
            ));
        }
    }
}

fn chunked_message_to_json(message: &ChunkedMessage) -> Option<serde_json::Value> {
    let at = message
        .meta
        .as_ref()
        .and_then(|meta| meta.at.as_ref())
        .map(|at| at.seconds);

    let Some(Payload::Message(message)) = &message.payload else {
        return None;
    };

    let mut json = match message.data.as_ref()? {
        Data::Chat(chat) | Data::OverflowedChat(chat) => serde_json::json!({
            "type": "chat",
            "content": chat.content,
            "name": chat.name,
            "rawUserId": chat.raw_user_id,
            "hashedUserId": chat.hashed_user_id,
            "premium": chat.account_status == 1,
        }),
        Data::SimpleNotification(notification) => {
            use simple_notification::Message;
            let (kind, content) = match notification.message.as_ref()? {
                Message::Ichiba(s) => ("ichiba", s),
                Message::Quote(s) => ("quote", s),
                Message::Emotion(s) => ("emotion", s),
                Message::Cruise(s) => ("cruise", s),
                Message::ProgramExtended(s) => ("programExtended", s),
                Message::RankingIn(s) => ("rankingIn", s),
                Message::RankingUpdated(s) => ("rankingUpdated", s),
                Message::Visited(s) => ("visited", s),
            };
            serde_json::json!({
                "type": "notification",
                "kind": kind,
                "content": content,
            })
        }
        Data::Gift(gift) => serde_json::json!({
            "type": "gift",
            "advertiserName": gift.advertiser_name,
            "itemName": gift.item_name,
            "point": gift.point,
            "message": gift.message,
        }),
        Data::Nicoad(ad) => {
            let content = match ad.versions.as_ref()? {
                nicoad::Versions::V0(v0) => v0
                    .latest
                    .as_ref()
                    .map(|latest| format!("{} {}pt", latest.advertiser, latest.point))
                    .unwrap_or_default(),
                nicoad::Versions::V1(v1) => v1.message.clone(),
            };
            serde_json::json!({
                "type": "nicoad",
                "content": content,
            })
        }
        _ => return None,
    };

    json["at"] = serde_json::json!(at);
    Some(json)
}
