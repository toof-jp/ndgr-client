use std::env;

use futures::{pin_mut, StreamExt};
use ndgr_client::{
    fetch_chunked_entry, fetch_chunked_message, fetch_program_info, websocket::fetch_ndgr_view_uri,
    ViewQuery,
};
use protobuf::chat::service::edge::chunked_entry::Entry;

#[tokio::main]
async fn main() {
    let url = env::args().nth(1).expect("URL is required");

    let info = fetch_program_info(&url).await.unwrap();
    let view_uri = fetch_ndgr_view_uri(&info.site.relive.web_socket_url)
        .await
        .unwrap();

    println!("{}", view_uri);

    let mut view_query = ViewQuery::Now;

    loop {
        let stream = fetch_chunked_entry(&view_uri, &view_query).await;
        pin_mut!(stream);

        while let Some(Ok(message)) = stream.next().await {
            println!("Received ChunkedEntry: {:?}", message);

            if let Some(entry) = message.entry {
                match entry {
                    Entry::Next(next) => {
                        println!("Next: {:?}", next.at);
                        view_query = ViewQuery::At(next.at);
                    }
                    Entry::Segment(segment) => {
                        let stream = fetch_chunked_message(&segment.uri).await;
                        pin_mut!(stream);

                        while let Some(message_result) = stream.next().await {
                            println!("Received ChunkedMessage: {:?}", message_result);
                        }
                    }
                    _ => (),
                }
            }
        }
    }
}
