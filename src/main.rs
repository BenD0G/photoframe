mod urls;
use photoframe::api::list_top_folder;

#[tokio::main]
async fn main() {
    let foo = list_top_folder().await;
    println!("{:#?}", foo);
}
