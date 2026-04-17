use crate::sources::traits::{GameImage, ImageKind, ImageSource, SearchResult};
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use steamgriddb_api::{
    query_parameters::{GridDimentions, GridQueryParameters, IconQueryParameters, MimeTypeIcon},
    Client, QueryType,
};

#[derive(Clone)]
pub struct SteamGridDB {
    client: Arc<Client>,
}

impl SteamGridDB {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Arc::new(Client::new(api_key)),
        }
    }
}

#[async_trait]
impl ImageSource for SteamGridDB {
    fn name(&self) -> &'static str {
        "SteamGridDB"
    }

    async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        let results = self
            .client
            .search(query)
            .await
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        Ok(results
            .into_iter()
            .map(|g| SearchResult {
                id: g.id.to_string(),
                name: g.name,
                release_date: g.release_date.map(|d| d.to_string()),
                image_url: None,
            })
            .collect())
    }

    async fn get_images(&self, game_id: &str) -> Result<Vec<GameImage>> {
        let id: usize = game_id.parse()?;
        let mut images = vec![];

        // Grids
        if let Ok(grids) = self
            .client
            .get_images_for_id(id, &QueryType::Grid(None))
            .await
        {
            for grid in grids {
                images.push(GameImage {
                    url: grid.url.clone(),
                    thumb: grid.thumb.clone(),
                    kind: ImageKind::Cover,
                });
            }
        }

        // Banners (landscape grids)
        if let Ok(grids) = self
            .client
            .get_images_for_id(
                id,
                &QueryType::Grid(Some(GridQueryParameters {
                    dimentions: Some(&[GridDimentions::D460x215, GridDimentions::D920x430]),
                    ..Default::default()
                })),
            )
            .await
        {
            for grid in grids {
                images.push(GameImage {
                    url: grid.url.clone(),
                    thumb: grid.thumb.clone(),
                    kind: ImageKind::Banner,
                });
            }
        }

        // Logos
        if let Ok(logos) = self
            .client
            .get_images_for_id(id, &QueryType::Logo(None))
            .await
        {
            for logo in logos {
                images.push(GameImage {
                    url: logo.url.clone(),
                    thumb: logo.thumb.clone(),
                    kind: ImageKind::Logo,
                });
            }
        }

        // Icons
        if let Ok(icons) = self
            .client
            .get_images_for_id(
                id,
                &QueryType::Icon(Some(IconQueryParameters {
                    mimes: Some(&[MimeTypeIcon::Png]),
                    ..Default::default()
                })),
            )
            .await
        {
            for icon in icons {
                images.push(GameImage {
                    url: icon.url.clone(),
                    thumb: icon.thumb.clone(),
                    kind: ImageKind::Icon,
                });
            }
        }

        Ok(images)
    }

    async fn get_thumbnail(&self, game_id: &str) -> Result<Option<String>> {
        let id: usize = game_id.parse()?;
        if let Ok(grids) = self
            .client
            .get_images_for_id(id, &QueryType::Grid(None))
            .await
        {
            Ok(grids.first().map(|g| g.thumb.clone()))
        } else {
            Ok(None)
        }
    }
}
