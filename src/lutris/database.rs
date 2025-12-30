use anyhow::Context;
use rusqlite::Connection;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Game {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub runner: String,
    pub installed: bool,
}

#[derive(Clone)]
pub struct LutrisDatabase {
    pub path: std::path::PathBuf,
}

impl LutrisDatabase {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn get_games(&self) -> anyhow::Result<Vec<Game>> {
        let conn = Connection::open(&self.path).context("Failed to open Lutris database")?;
        let mut stmt = conn.prepare("SELECT id, slug, name, runner, installed FROM games WHERE installed = 1 ORDER BY name ASC")?;

        let games_iter = stmt.query_map([], |row| {
            Ok(Game {
                id: row.get(0)?,
                slug: row.get(1)?,
                name: row.get(2)?,
                runner: row.get(3)?,
                installed: row.get(4)?,
            })
        })?;

        let mut games = Vec::new();
        for game in games_iter {
            games.push(game?);
        }
        Ok(games)
    }
}
