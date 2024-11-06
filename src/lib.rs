use anyhow::Result;

use crate::program_info::ProgramInfo;

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
