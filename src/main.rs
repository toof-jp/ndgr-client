use std::env;

use ndgr_client::{fetch_program_info, websocket::fetch_ndgr_view_uri};

#[tokio::main]
async fn main() {
    let url = env::args().nth(1).expect("URL is required");

    let info = fetch_program_info(&url).await.unwrap();
    let view_uri = fetch_ndgr_view_uri(&info.site.relive.web_socket_url)
        .await
        .unwrap();

    println!("{}", view_uri);
}
