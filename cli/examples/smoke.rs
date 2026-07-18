use futures::{StreamExt, pin_mut};
use ndgr_client::websocket::WebSocketClient;
use ndgr_client::{fetch_program_info, stream_chunked_message};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let url = std::env::args().nth(1).expect("URL is required");

    let info = fetch_program_info(&url).await?;
    println!("web_socket_url: {}", info.site.relive.web_socket_url);

    let client = WebSocketClient::new(&info.site.relive.web_socket_url).await?;
    let view_uri = client.view_uri();
    println!("view_uri: {}", view_uri);

    let stream = stream_chunked_message(view_uri).await;
    pin_mut!(stream);

    let mut count = 0;
    while let Some(message) = stream.next().await {
        println!("{:?}", message);
        count += 1;
        if count >= 5 {
            break;
        }
    }

    println!("received {count} messages, OK");
    Ok(())
}
