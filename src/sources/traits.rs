use anyhow::Result;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct GameImage {
    pub url: String,
    pub thumb: String,
    pub kind: ImageKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageKind {
    Cover,
    Banner,
    Icon,
    Hero,
    Logo,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub name: String,
    pub release_date: Option<String>,
    pub image_url: Option<String>,
}

#[async_trait]
pub trait ImageSource: Send + Sync {
    fn name(&self) -> &'static str;
    async fn search(&self, query: &str) -> Result<Vec<SearchResult>>;
    async fn get_images(&self, game_id: &str) -> Result<Vec<GameImage>>;
    async fn get_thumbnail(&self, game_id: &str) -> Result<Option<String>>;
}
