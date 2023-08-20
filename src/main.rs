mod urls;
use photoframe::api::get_file_ids_in_folder;

#[tokio::main]
async fn main() {
    let folder_id: u64 = 6211910250;
    let file_ids = get_file_ids_in_folder(folder_id).await;
    println!("{file_ids:?}");
    let zip = photoframe::api::get_zip(&file_ids).await;
    println!("{:#?}", zip);
}
