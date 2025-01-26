use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::{err_with_context, error::Error};

#[derive(Debug, Serialize, Deserialize)]
pub struct CurrentBlock {
    pub block_type_id: u8,
    pub current_block_name: String,
}

impl CurrentBlock {
    pub async fn get(data_dir: &Path) -> Result<Self, Error> {
        let current_block_file = data_dir.join("currentblock.json");
        if !current_block_file.exists() {
            return Ok(CurrentBlock {
                block_type_id: 0,
                current_block_name: "Hello for first setup".to_string(),
            });
        }
        let currrent_data_file = tokio::fs::read_to_string(&current_block_file)
            .await
            .map_err(|e| err_with_context!(e, "Reading {}", current_block_file.display()))?;
        let res = serde_json::from_str(&currrent_data_file)
            .map_err(|e| err_with_context!(e, "Deserializing {}", current_block_file.display()))?;
        Ok(res)
    }

    pub async fn save(&self, data_dir: &Path) -> Result<(), Error> {
        let current_block_file = data_dir.join("currentblock.json");
        let data = serde_json::to_string(self)
            .map_err(|e| err_with_context!(e, "Serializing {}", current_block_file.display()))?;
        tokio::fs::write(&current_block_file, data)
            .await
            .map_err(|e| err_with_context!(e, "Writing {}", current_block_file.display()))?;
        Ok(())
    }
}
