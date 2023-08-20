use sha1::{Digest, Sha1};

use crate::urls::EndPoint;

/// Return a digest that must be used within 30s.
pub async fn get_digest() -> String {
    let url = EndPoint::GetDigest.get_url();
    let text = reqwest::get(&url).await.unwrap().text().await.unwrap();
    let foo = serde_json::from_str::<serde_json::Value>(&text).unwrap();
    foo["digest"].as_str().unwrap().to_string()
}

fn make_sha1(x: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(x);
    let result = hasher.finalize();
    format!("{:x}", result)
}

fn make_password_digest(username: &str, password: &str, digest: &str) -> String {
    let username_sha1 = make_sha1(username);
    make_sha1(format!("{}{}{}", password, username_sha1, digest).as_str())
}

/// Generate an auth token.
pub async fn get_auth_token() -> serde_json::Value {
    let username = std::env::var("PHOTOFRAME_USERNAME")
        .unwrap()
        .replace("@", "%40");
    let password = std::env::var("PHOTOFRAME_PASSWORD").unwrap();
    let digest = get_digest().await;
    let password_digest = make_password_digest(&username, &password, &digest);
    println!(
        "{}\n{}\n{}\n{}",
        username, password, digest, password_digest
    );

    let url = EndPoint::UserInfo.get_url();
    let url = format!(
        "{}?authexpire=63072000&username={}&digest={}&passworddigest={}",
        url,
        username, //.replace("@", "%40"),
        digest,
        password_digest
    );
    let response = reqwest::get(&url).await.unwrap();
    println!("{:#?} {}", response, response.status());
    let headers = response.headers();
    println!("{:#?}", headers);
    let text = response.text().await.unwrap();
    let foo = serde_json::from_str::<serde_json::Value>(&text).unwrap();
    // foo["auth"].as_str().unwrap().to_string()
    foo
}

pub async fn get_oauth_token() -> serde_json::Value {
    let url = EndPoint::Oauth2Token.get_url();
    let client_id = std::env::var("PHOTOFRAME_CLIENT_ID").unwrap();
    let client_secret = std::env::var("PHOTOFRAME_CLIENT_SECRET").unwrap();
    let url = format!("{url}?client_id={client_id}&client_secret={client_secret}");
    let response = reqwest::get(&url).await.unwrap();
    println!("{:#?} {}", response, response.status());
    let text = response.text().await.unwrap();
    let foo = serde_json::from_str::<serde_json::Value>(&text).unwrap();
    // foo

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let bar = client.get(url).send().await.unwrap();
    let text = bar.text().await.unwrap();
    let baz = serde_json::from_str::<serde_json::Value>(&text).unwrap();

    baz
}

pub async fn list_top_folder() -> serde_json::Value {
    let url = EndPoint::ListFolder.get_url();
    let url = format!("{url}&folderid=0");
    let response = reqwest::get(&url).await.unwrap();
    let text = response.text().await.unwrap();
    let foo = serde_json::from_str::<serde_json::Value>(&text).unwrap();
    foo
}
