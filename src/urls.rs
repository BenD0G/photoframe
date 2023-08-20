use std::f32::consts::E;

const BASE_URL: &str = "https://eapi.pcloud.com";

pub enum EndPoint {
    GetDigest,
    ListFolder,
    Oauth2Token,
    UserInfo,
}

impl EndPoint {
    pub fn get_url(&self) -> String {
        let token = std::env::var("PHOTOFRAME_OAUTH_TOKEN").unwrap();
        let method_name = match self {
            EndPoint::GetDigest => "getdigest",
            EndPoint::ListFolder => "listfolder",
            EndPoint::Oauth2Token => "oauth2_token",
            EndPoint::UserInfo => "userinfo",
        };
        format!("{}/{}?access_token={}", BASE_URL, method_name, token)
    }
}
