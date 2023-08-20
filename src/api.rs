use crate::urls::EndPoint;

pub async fn list_top_folder() -> serde_json::Value {
    let url = EndPoint::ListFolder.get_url();
    let url = format!("{url}&folderid=0");
    let response = reqwest::get(&url).await.unwrap();
    let text = response.text().await.unwrap();
    let foo = serde_json::from_str::<serde_json::Value>(&text).unwrap();
    foo
}

pub async fn get_file_ids_in_folder(folder_id: u64) -> Vec<u64> {
    let url = EndPoint::ListFolder.get_url();
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

pub async fn get_zip(file_ids: &[u64]) {
    let url = EndPoint::GetZip.get_url();
    let file_ids = file_ids
        .iter()
        .map(|x| format!("fileids[]={}", x))
        .collect::<Vec<String>>()
        .join("&");
    let url = format!("{url}&{file_ids}");
    let response = reqwest::get(&url).await.unwrap();
    let text = response.text().await.unwrap();
    let foo = serde_json::from_str::<serde_json::Value>(&text).unwrap();
    println!("{:#?}", foo);
}
