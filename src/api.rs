use log::warn;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{Cursor, Read},
    path::PathBuf,
};

use sha1::{Digest, Sha1};

const BASE_URL: &str = "https://eapi.pcloud.com";

enum EndPoint {
    GetDigest,
    GetZip,
    ListFolder,
    UserInfo,
}

impl EndPoint {
    fn get_url(&self) -> String {
        let method_name = match self {
            EndPoint::GetDigest => "getdigest",
            EndPoint::GetZip => "getzip",
            EndPoint::ListFolder => "listfolder",
            EndPoint::UserInfo => "userinfo",
        };
        format!("{BASE_URL}/{method_name}")
    }

    fn get_url_with_oauth_token(&self, config: &Config) -> String {
        let token = &config.pcloud_oath_token;
        let method_name = match self {
            EndPoint::GetDigest => "getdigest",
            EndPoint::GetZip => "getzip",
            EndPoint::ListFolder => "listfolder",
            EndPoint::UserInfo => "userinfo",
        };
        format!("{BASE_URL}/{method_name}?access_token={token}")
    }
}

#[derive(Debug)]
pub struct Config {
    pub index_file: PathBuf,
    pub photo_dir: PathBuf,
    pub pcloud_username: String,
    pub pcloud_password: String,
    pub pcloud_oath_token: String,
}

impl Config {
    /// Read from environment variables.
    pub fn new() -> Self {
        Config {
            index_file: std::env::var("PHOTOFRAME_INDEX_FILE")
                .expect("PHOTOFRAME_INDEX_FILE not set")
                .into(),
            photo_dir: std::env::var("PHOTOFRAME_PHOTO_DIR")
                .expect("PHOTOFRAME_PHOTO_DIR not set")
                .into(),
            pcloud_username: std::env::var("PHOTOFRAME_USERNAME")
                .expect("PHOTOFRAME_USERNAME not set"),
            pcloud_password: std::env::var("PHOTOFRAME_PASSWORD")
                .expect("PHOTOFRAME_PASSWORD not set"),
            pcloud_oath_token: std::env::var("PHOTOFRAME_OAUTH_TOKEN")
                .expect("PHOTOFRAME_OAUTH_TOKEN not set"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct FileMetadata {
    file_id: u64,
    file_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileIndex {
    files: Vec<FileMetadata>,
}

impl FileIndex {
    pub fn get_new_file_ids_and_file_names_to_delete(
        &self,
        desired_index: &FileIndex,
    ) -> (Vec<u64>, Vec<String>) {
        let current_file_ids = self
            .files
            .iter()
            .map(|x| x.file_id)
            .collect::<HashSet<u64>>();
        let desired_file_ids = desired_index
            .files
            .iter()
            .map(|x| x.file_id)
            .collect::<HashSet<u64>>();
        let file_ids_to_download = desired_file_ids
            .difference(&current_file_ids)
            .map(|x| *x)
            .collect::<Vec<u64>>();
        let file_id_to_file_name = self
            .files
            .iter()
            .map(|x| (x.file_id, x.file_name.clone()))
            .collect::<HashMap<u64, String>>();
        let file_names_to_delete = current_file_ids
            .difference(&desired_file_ids)
            .map(|x| {
                file_id_to_file_name
                    .get(x)
                    .unwrap_or_else(|| panic!("File ID {} not found in current file index", x))
                    .clone()
            })
            .collect::<Vec<String>>();
        (file_ids_to_download, file_names_to_delete)
    }

    pub fn read(path: &PathBuf) -> FileIndex {
        match File::open(path) {
            Ok(file) => {
                let reader = std::io::BufReader::new(file);
                serde_json::from_reader(reader).unwrap()
            }
            Err(_) => {
                warn!("Failed to read file index at {path:?}, returning empty one.");
                FileIndex { files: vec![] }
            }
        }
    }

    pub fn write(&self, path: &PathBuf) {
        let file = File::create(path).unwrap();
        let writer = std::io::BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self).unwrap();
    }
}

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
pub async fn get_auth_token(config: &Config) -> String {
    let username = &config.pcloud_username;
    let password = &config.pcloud_password;
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

pub async fn get_file_ids_in_folder(folder_id: u64, config: &Config) -> FileIndex {
    let url = EndPoint::ListFolder.get_url_with_oauth_token(config);
    let url = format!("{url}&folderid={folder_id}&filterfilemeta=fileid,name");
    let response = reqwest::get(&url).await.unwrap();
    let text = response.text().await.unwrap();
    let json = serde_json::from_str::<serde_json::Value>(&text).unwrap();
    let file_metas = json["metadata"]["contents"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| FileMetadata {
            file_id: x["fileid"].as_u64().unwrap(),
            file_name: x["name"].as_str().unwrap().to_string(),
        })
        .collect();
    FileIndex { files: file_metas }
}

fn unzip_and_save<R: Read + std::io::Seek>(reader: R, dir: &PathBuf) -> zip::result::ZipResult<()> {
    let mut archive = zip::ZipArchive::new(reader)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let filename = file.enclosed_name().unwrap();

        let path = dir.join(filename);

        let mut outfile = File::create(&path)?;
        std::io::copy(&mut file, &mut outfile)?;
    }
    Ok(())
}

/// Download the file ID's to the current directory.
/// This requires token-based auth and can't use OAuth for some undocumented reason.
/// An additional constraint is that we don't know which file corresponds to which file ID.
pub async fn get_zip(file_ids: &[u64], token: &str, dir: &PathBuf) {
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
    unzip_and_save(reader, dir).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_new_file_ids_and_file_names_to_delete() {
        let downloaded_files = FileIndex {
            files: vec![
                FileMetadata {
                    file_id: 1,
                    file_name: "foo".to_string(),
                },
                FileMetadata {
                    file_id: 2,
                    file_name: "bar".to_string(),
                },
                FileMetadata {
                    file_id: 3,
                    file_name: "baz".to_string(),
                },
            ],
        };
        let new_index = FileIndex {
            files: vec![
                FileMetadata {
                    file_id: 1,
                    file_name: "foo".to_string(),
                },
                FileMetadata {
                    file_id: 2,
                    file_name: "bar".to_string(),
                },
                FileMetadata {
                    file_id: 4,
                    file_name: "poop".to_string(),
                },
            ],
        };
        let (file_ids_to_download, file_names_to_delete) =
            downloaded_files.get_new_file_ids_and_file_names_to_delete(&new_index);
        assert_eq!(file_ids_to_download, vec![4]);
        assert_eq!(file_names_to_delete, vec!["baz"]);
    }
}
