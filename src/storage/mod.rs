pub mod models;

pub use models::*;

#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Failed to read database: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("Failed to parse database: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("String not found: {0}")]
    StringNotFound(String),

    #[error("Database directory not found")]
    NoDatabaseDir,
}

#[cfg(not(target_arch = "wasm32"))]
pub struct Storage {
    path: PathBuf,
}

#[cfg(not(target_arch = "wasm32"))]
impl Storage {
    pub fn new() -> Result<Self, StorageError> {
        let path = Self::default_path()?;
        Ok(Self { path })
    }

    pub fn with_path(path: PathBuf) -> Self {
        Self { path }
    }

    fn default_path() -> Result<PathBuf, StorageError> {
        let config_dir = dirs::config_dir().ok_or(StorageError::NoDatabaseDir)?;
        let app_dir = config_dir.join("redteamstrings");
        Ok(app_dir.join("database.json"))
    }

    pub fn load(&self) -> Result<Database, StorageError> {
        if !self.path.exists() {
            return Ok(Database::default());
        }

        let contents = fs::read_to_string(&self.path)?;
        let db: Database = serde_json::from_str(&contents)?;
        Ok(db)
    }

    pub fn save(&self, db: &Database) -> Result<(), StorageError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = serde_json::to_string_pretty(db)?;
        fs::write(&self.path, contents)?;
        Ok(())
    }

    pub fn add_string(&self, tracked: TrackedString) -> Result<(), StorageError> {
        let mut db = self.load()?;
        db.strings.push(tracked);
        self.save(&db)
    }

    pub fn find_by_value(&self, value: &str) -> Result<Option<TrackedString>, StorageError> {
        let db = self.load()?;
        Ok(db.strings.into_iter().find(|s| s.value == value))
    }

    pub fn find_by_id(&self, id: &str) -> Result<Option<TrackedString>, StorageError> {
        let db = self.load()?;
        let uuid = uuid::Uuid::parse_str(id).ok();
        Ok(db.strings.into_iter().find(|s| {
            uuid.map(|u| s.id == u).unwrap_or(false) || s.value == id
        }))
    }

    pub fn update_string(&self, tracked: TrackedString) -> Result<(), StorageError> {
        let mut db = self.load()?;
        if let Some(pos) = db.strings.iter().position(|s| s.id == tracked.id) {
            db.strings[pos] = tracked;
            self.save(&db)?;
            Ok(())
        } else {
            Err(StorageError::StringNotFound(tracked.id.to_string()))
        }
    }

    pub fn list_all(&self) -> Result<Vec<TrackedString>, StorageError> {
        let db = self.load()?;
        Ok(db.strings)
    }

    pub fn list_by_tag(&self, tag: &str) -> Result<Vec<TrackedString>, StorageError> {
        let db = self.load()?;
        Ok(db
            .strings
            .into_iter()
            .filter(|s| s.tags.iter().any(|t| t.contains(tag)))
            .collect())
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for Storage {
    fn default() -> Self {
        Self::new().expect("Failed to initialize storage")
    }
}
