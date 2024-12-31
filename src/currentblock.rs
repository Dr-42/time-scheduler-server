use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Serialize, Deserialize)]
pub struct CurrentBlock {
    pub block_type_id: u8,
    pub current_block_name: String,
}

pub enum CurrentBlockError {
    Tokio(tokio::io::Error),
    Serde(serde_json::Error),
}

impl Display for CurrentBlockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CurrentBlockError::Tokio(e) => write!(f, "Tokio error: {}", e),
            CurrentBlockError::Serde(e) => write!(f, "Serde error: {}", e),
        }
    }
}

impl From<tokio::io::Error> for CurrentBlockError {
    fn from(e: tokio::io::Error) -> Self {
        CurrentBlockError::Tokio(e)
    }
}

impl From<serde_json::Error> for CurrentBlockError {
    fn from(e: serde_json::Error) -> Self {
        CurrentBlockError::Serde(e)
    }
}

impl CurrentBlock {
    pub async fn get() -> Result<Self, CurrentBlockError> {
        let currrent_data_file = tokio::fs::read_to_string("currentblock.json").await?;
        let res = serde_json::from_str(&currrent_data_file)?;
        Ok(res)
    }

    pub async fn save(&self) -> Result<(), CurrentBlockError> {
        let data = serde_json::to_string(self)?;
        tokio::fs::write("currentblock.json", data).await?;
        Ok(())
    }
}
