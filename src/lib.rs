use anyhow::Result;
use async_stream::{stream, try_stream};
use bytes::BytesMut;
use futures::pin_mut;
use futures_core::stream::Stream;
use futures_util::StreamExt;
use protobuf::chat::service::edge::{chunked_entry::Entry, ChunkedEntry, ChunkedMessage};

use crate::program_info::ProgramInfo;

pub mod comment_buffer;
pub mod program_info;
pub mod websocket;

// TODO 番組終了の場合の処理
pub async fn fetch_program_info(url: &str) -> Result<ProgramInfo> {
    let html = reqwest::Client::new().get(url).send().await?.text().await?;

    let document = scraper::Html::parse_document(&html);
    let selector = scraper::Selector::parse("#embedded-data").unwrap();

    if let Some(element) = document.select(&selector).next() {
        if let Some(data_props) = element.value().attr("data-props") {
            let info: program_info::ProgramInfo = serde_json::from_str(data_props)?;
            return Ok(info);
        }
    }

    Err(anyhow::anyhow!("program info not found"))
}

pub enum ViewQuery {
    Now,
    At(i64),
}

pub async fn fetch_chunked_entry(
    url: &str,
    query: &ViewQuery,
) -> impl Stream<Item = Result<ChunkedEntry>> {
    let at_str = match query {
        ViewQuery::Now => "now".to_string(),
        ViewQuery::At(at) => at.to_string(),
    };
    let url = format!("{url}?at={at_str}");

    fetch_protobuf_stream::<ChunkedEntry>(&url).await
}

pub async fn fetch_chunked_message(url: &str) -> impl Stream<Item = Result<ChunkedMessage>> {
    fetch_protobuf_stream::<ChunkedMessage>(url).await
}

pub async fn fetch_protobuf_stream<T: prost::Message + Default>(
    url: &str,
) -> impl Stream<Item = Result<T>> {
    let response = reqwest::get(url).await;

    try_stream! {
        let mut stream = response?.bytes_stream();
        let mut buffer = BytesMut::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            buffer.extend_from_slice(&chunk);

            while let Ok(message) = T::decode_length_delimited(&mut buffer) {
                yield message;
            }
        }
    }
}

pub async fn stream_chunked_message<'a>(
    view_uri: &'a str,
) -> impl Stream<Item = ChunkedMessage> + 'a {
    stream! {
        let mut view_query = ViewQuery::Now;
        loop {
            let stream = fetch_chunked_entry(view_uri, &view_query).await;
            pin_mut!(stream);

            while let Some(Ok(message)) = stream.next().await {
                if let Some(entry) = message.entry {
                    match entry {
                        Entry::Next(next) => {
                            view_query = ViewQuery::At(next.at);
                        }
                        Entry::Segment(segment) => {
                            let stream = fetch_chunked_message(&segment.uri).await;
                            pin_mut!(stream);

                            while let Some(Ok(message)) = stream.next().await {
                                yield message;
                            }
                        }
                        _ => (),
                    }
                }
            }
        }
    }
}
