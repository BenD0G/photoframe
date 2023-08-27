const BASE_URL: &str = "https://eapi.pcloud.com";

pub enum EndPoint {
    GetDigest,
    GetZip,
    ListFolder,
    Oauth2Token,
    UserInfo,
}

impl EndPoint {
    pub fn get_url(&self) -> String {
        let method_name = match self {
            EndPoint::GetDigest => "getdigest",
            EndPoint::GetZip => "getzip",
            EndPoint::ListFolder => "listfolder",
            EndPoint::Oauth2Token => "oauth2_token",
            EndPoint::UserInfo => "userinfo",
        };
        format!("{BASE_URL}/{method_name}")
    }

    pub fn get_url_with_oauth_token(&self) -> String {
        let token = std::env::var("PHOTOFRAME_OAUTH_TOKEN").unwrap();
        let method_name = match self {
            EndPoint::GetDigest => "getdigest",
            EndPoint::GetZip => "getzip",
            EndPoint::ListFolder => "listfolder",
            EndPoint::Oauth2Token => "oauth2_token",
            EndPoint::UserInfo => "userinfo",
        };
        format!("{BASE_URL}/{method_name}?access_token={token}")
    }
}
