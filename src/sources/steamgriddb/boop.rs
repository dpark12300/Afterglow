use reqwest::{header::HeaderValue, Client as HttpClient};

const BOOP_BASE_URL: &str = "https://www.steamgriddb.com/api/sgdboop";
const BOOP_AUTH_HEADER: &str = "Bearer 62696720-6f69-6c79-2070-65656e75733f";
const BOOP_API_VERSION: &str = "3";

#[derive(Debug, Clone)]
pub struct BoopAssetResponse {
    pub app_id: String,
    pub asset_url: String,
    pub asset_type: String,
}

pub async fn fetch_asset(
    asset_type: &str,
    asset_id: &str,
    for_nonsteam: bool,
) -> Result<BoopAssetResponse, String> {
    let mut url = format!("{}/{}/{}", BOOP_BASE_URL, asset_type, asset_id);
    if for_nonsteam {
        url.push_str("?nonsteam=1");
    }

    let client = HttpClient::new();
    let response = client
        .get(&url)
        .header("Authorization", HeaderValue::from_static(BOOP_AUTH_HEADER))
        .header("X-BOOP-API-VER", HeaderValue::from_static(BOOP_API_VERSION))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = response.status();
    let bytes = response.bytes().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        let body = String::from_utf8_lossy(&bytes).trim().to_string();
        return Err(format!("SGDBoop request failed ({}): {}", status, body));
    }

    let payload = String::from_utf8_lossy(&bytes);
    let line = payload
        .lines()
        .next()
        .map(|line| line.trim_matches(|c| c == '\r' || c == '\n' || c == '\0'))
        .unwrap_or("");

    if line.is_empty() {
        return Err("SGDBoop response was empty".to_string());
    }

    let mut columns = line.splitn(5, ',');
    let app_id = columns.next();
    let orientation = columns.next();
    let asset_url = columns.next();
    let asset_type = columns.next();
    let asset_hash = columns.next();

    match (app_id, orientation, asset_url, asset_type, asset_hash) {
        (
            Some(app_id),
            Some(_orientation),
            Some(asset_url),
            Some(asset_type),
            Some(_asset_hash),
        ) => Ok(BoopAssetResponse {
            app_id: app_id.trim().to_string(),
            asset_url: asset_url.trim().to_string(),
            asset_type: asset_type.trim().to_string(),
        }),
        _ => Err(format!("Unexpected SGDBoop response format: {}", line)),
    }
}
