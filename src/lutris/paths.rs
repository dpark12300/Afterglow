use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct LutrisPaths {
    root: PathBuf,
    icons_dir: PathBuf,
}

impl LutrisPaths {
    pub const ICON_PREFIX: &'static str = "lutris_";

    pub fn from_root(root: PathBuf, icon_override: Option<PathBuf>) -> Option<Self> {
        let db = root.join("pga.db");
        if db.is_file() {
            let icons_dir = icon_override.unwrap_or_else(|| Self::infer_icons_dir(&root));
            Some(Self { root, icons_dir })
        } else {
            None
        }
    }

    pub fn from_database_path(path: PathBuf, icon_override: Option<PathBuf>) -> Option<Self> {
        if path.is_file() {
            path.parent().map(|parent| {
                let root = parent.to_path_buf();
                let icons_dir = icon_override.unwrap_or_else(|| Self::infer_icons_dir(&root));
                Self { root, icons_dir }
            })
        } else {
            None
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn database_path(&self) -> PathBuf {
        self.root.join("pga.db")
    }

    pub fn covers_dir(&self) -> PathBuf {
        self.root.join("coverart")
    }

    pub fn banners_dir(&self) -> PathBuf {
        self.root.join("banners")
    }

    pub fn icons_dir(&self) -> PathBuf {
        self.icons_dir.clone()
    }

    pub fn default_database_locations() -> Vec<PathBuf> {
        let mut defaults = Vec::new();
        if let Some(dir) = dirs::data_local_dir() {
            defaults.push(dir.join("lutris").join("pga.db"));
        }
        if let Some(home) = dirs::home_dir() {
            defaults.push(
                home.join(".var/app/net.lutris.Lutris/data/lutris")
                    .join("pga.db"),
            );
        }
        defaults
    }

    fn infer_icons_dir(root: &Path) -> PathBuf {
        if let Some(home) = dirs::home_dir() {
            let native_root = home.join(".local/share/lutris");
            if root.starts_with(&native_root) {
                return home.join(".local/share/icons/hicolor/128x128/apps");
            }

            let flatpak_root = home.join(".var/app/net.lutris.Lutris/data/lutris");
            if root.starts_with(&flatpak_root) {
                return home.join(".var/app/net.lutris.Lutris/data/icons/hicolor/128x128/apps");
            }
        }

        root.join("icons")
    }
}
