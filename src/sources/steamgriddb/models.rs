use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Response<T> {
    pub success: bool,
    pub data: T,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GameData {
    pub id: i64,
    pub name: String,
    pub release_date: Option<i64>,
    pub types: Option<Vec<String>>,
    pub verified: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ImageData {
    pub id: i64,
    pub score: i32,
    pub style: String,
    pub url: String,
    pub thumb: String,
    pub tags: Vec<String>,
    pub author: Author,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Author {
    pub name: String,
    pub steam64: String,
    pub avatar: String,
}
