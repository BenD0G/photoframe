use std::path::PathBuf;

use log::{error, info};
use photoframe::api::{get_auth_token, get_file_ids_in_folder, Config, FileIndex};
use tokio::time::{interval, Duration};

const FOLDER_ID: u64 = 6211910250;

async fn update(config: &Config) {
    let desired_file_index = get_file_ids_in_folder(FOLDER_ID).await;
    let existing_file_index = FileIndex::read(&config.index_file);

    let (file_ids_to_download, file_names_to_delete) =
        existing_file_index.get_new_file_ids_and_file_names_to_delete(&desired_file_index);

    for file_name in file_names_to_delete {
        match std::fs::remove_file(&file_name) {
            Ok(_) => {}
            Err(e) => error!("Failed to delete file {}: {}", file_name, e),
        }
    }

    let token = get_auth_token().await;
    photoframe::api::get_zip(&file_ids_to_download, &token).await;
    desired_file_index.write(&config.index_file);
    info!("Completed update.")
}

#[tokio::main]
async fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();

    let index_file = std::env::var("PHOTOFRAME_INDEX_FILE").unwrap();
    let photo_dir = std::env::var("PHOTOFRAME_PHOTO_DIR").unwrap();

    std::fs::create_dir_all(&photo_dir).unwrap();
    std::fs::create_dir_all(&PathBuf::from(&index_file).parent().unwrap()).unwrap();

    let config = Config {
        index_file: index_file.into(),
        photo_dir: photo_dir.into(),
    };

    let mut interval = interval(Duration::from_secs(30));
    loop {
        interval.tick().await;
        update(&config).await;
    }
}
