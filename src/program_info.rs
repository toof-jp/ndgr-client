use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ProgramInfo {
    pub site: Site,
}

#[derive(Debug, Deserialize)]
pub struct Site {
    pub relive: Relive,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Relive {
    pub web_socket_url: String,
}
