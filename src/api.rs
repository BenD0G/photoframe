use crate::urls::EndPoint;

pub async fn list_top_folder() -> serde_json::Value {
    let url = EndPoint::ListFolder.get_url();
    let url = format!("{url}&folderid=0");
    let response = reqwest::get(&url).await.unwrap();
    let text = response.text().await.unwrap();
    let foo = serde_json::from_str::<serde_json::Value>(&text).unwrap();
    foo
}
