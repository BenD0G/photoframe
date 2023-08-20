use std::{fs::File, io::{Read, Cursor}};

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
pub async fn get_auth_token() -> String {
    let username = std::env::var("PHOTOFRAME_USERNAME").unwrap();
    let password = std::env::var("PHOTOFRAME_PASSWORD").unwrap();
    let digest = get_digest().await;
    let password_digest = make_password_digest(&username, &password, &digest);

    let url = EndPoint::UserInfo.get_url();
    let url = format!(
        "{}?getauth=1&authexpire=63072000&username={}&digest={}&passworddigest={}",
        url, username, digest, password_digest
    );
    let response = reqwest::get(&url).await.unwrap();
    let text = response.text().await.unwrap();
    let foo = serde_json::from_str::<serde_json::Value>(&text).unwrap();
    foo["auth"].as_str().unwrap().to_string()
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
    let url = EndPoint::ListFolder.get_url_with_oauth_token();
    let url = format!("{url}&folderid=0");
    let response = reqwest::get(&url).await.unwrap();
    let text = response.text().await.unwrap();
    let foo = serde_json::from_str::<serde_json::Value>(&text).unwrap();
    foo
}

// Full possible output for a file is:
// "category": Number(1),
// "comments": Number(0),
// "contenttype": String("image/jpeg"),
// "created": String("Mon, 17 Apr 2023 17:22:04 +0000"),
// "exifdatetime": Number(1681755723),
// "fileid": Number(26016926125),
// "hash": Number(9722675623775271780),
// "height": Number(4000),
// "icon": String("image"),
// "id": String("f26016926125"),
// "isfolder": Bool(false),
// "ismine": Bool(true),
// "isshared": Bool(false),
// "modified": String("Mon, 17 Apr 2023 17:22:04 +0000"),
// "name": String("IMG20230417182203.jpg"),
// "parentfolderid": Number(6211910250),
// "size": Number(2991548),
// "thumb": Bool(true),
// "width": Number(3008),
pub async fn get_file_ids_in_folder(folder_id: u64) -> Vec<u64> {
    let url = EndPoint::ListFolder.get_url_with_oauth_token();
    let url = format!("{url}&folderid={folder_id}&filterfilemeta=fileid");
    let response = reqwest::get(&url).await.unwrap();
    let text = response.text().await.unwrap();
    let foo = serde_json::from_str::<serde_json::Value>(&text).unwrap();
    foo["metadata"]["contents"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x["fileid"].as_u64().unwrap())
        .collect()
}

fn unzip_and_save<R: Read + std::io::Seek>(reader: R) -> zip::result::ZipResult<()> {
    let mut archive = zip::ZipArchive::new(reader)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = file.enclosed_name().unwrap();

        // if let Some(p) = outpath.parent() {
        //     if !p.exists() {
        //         std::fs::create_dir_all(&p)?;
        //     }
        // }
        let mut outfile = File::create(&outpath)?;
        std::io::copy(&mut file, &mut outfile)?;
    }
    Ok(())
}

/// Download the file ID's to the current directory.
/// This requires token-based auth and can't use OAuth for some undocumented reason.
/// An additional constraint is that we don't know which file corresponds to which file ID.
pub async fn get_zip(file_ids: &[u64], token: &str) {
    let url = EndPoint::GetZip.get_url();
    let file_ids = file_ids
        .iter()
        .map(|x| format!("{x}"))
        .collect::<Vec<String>>()
        .join(",");
    let url = format!("{url}?auth={token}&fileids={file_ids}");
    let response = reqwest::get(&url).await.unwrap();
    let bytes = response.bytes().await.unwrap();
    let reader = Cursor::new(bytes);
    unzip_and_save(reader).unwrap();
    // let foo = serde_json::from_str::<serde_json::Value>(&text).unwrap();
    // println!("{:#?}", text);
}
