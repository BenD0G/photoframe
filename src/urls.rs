const BASE_URL: &str = "https://eapi.pcloud.com";

pub enum EndPoint {
    GetZip,
    ListFolder,
}

impl EndPoint {
    pub fn get_url(&self) -> String {
        let token = std::env::var("PHOTOFRAME_OAUTH_TOKEN").unwrap();
        let method_name = match self {
            EndPoint::GetZip => "getzip",
            EndPoint::ListFolder => "listfolder",
        };
        format!("{}/{}?access_token={}", BASE_URL, method_name, token)
    }
}
