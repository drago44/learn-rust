use crate::domain::portfolio::Portfolio;
use anyhow::Result;

pub trait StoragePort {
    fn load(&self) -> Result<Portfolio>;
    fn save(&self, portfolio: &Portfolio) -> Result<()>;
}
