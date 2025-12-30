use iced::widget::image;

/// Downloads an image from the provided URL and produces an Iced image handle.
pub async fn download_image(url: String) -> Result<image::Handle, String> {
    let bytes = reqwest::get(&url)
        .await
        .map_err(|e| e.to_string())?
        .bytes()
        .await
        .map_err(|e| e.to_string())?;

    Ok(image::Handle::from_bytes(bytes))
}
