use crate::domain::portfolio::Portfolio;
use crate::ports::storage::StoragePort;
use anyhow::Result;
use std::path::PathBuf;

pub struct JsonStorage {
    path: PathBuf,
}

impl JsonStorage {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl StoragePort for JsonStorage {
    fn load(&self) -> Result<Portfolio> {
        if !self.path.exists() {
            return Ok(Portfolio {
                assets: std::collections::HashMap::new(),
            });
        }

        let content = std::fs::read_to_string(&self.path)?;
        let portfolio: Portfolio = serde_json::from_str(&content)?;
        Ok(portfolio)
    }

    fn save(&self, portfolio: &Portfolio) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(portfolio)?;
        std::fs::write(&self.path, content)?;
        Ok(())
    }
}
