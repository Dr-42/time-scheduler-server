use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::{err::Error, err_with_context};

#[derive(Debug, Serialize, Deserialize)]
pub struct CurrentBlock {
    pub block_type_id: u8,
    pub current_block_name: String,
}

impl CurrentBlock {
    pub async fn get() -> Result<Self, Error> {
        if !Path::new("currentblock.json").exists() {
            return Ok(CurrentBlock {
                block_type_id: 0,
                current_block_name: "Hello for first setup".to_string(),
            });
        }
        let currrent_data_file = tokio::fs::read_to_string("currentblock.json")
            .await
            .map_err(|e| err_with_context!(e, "Reading currentblock.json"))?;
        let res = serde_json::from_str(&currrent_data_file)
            .map_err(|e| err_with_context!(e, "Deserializing currentblock.json"))?;
        Ok(res)
    }

    pub async fn save(&self) -> Result<(), Error> {
        let data = serde_json::to_string(self)
            .map_err(|e| err_with_context!(e, "Serializing currentblock.json"))?;
        tokio::fs::write("currentblock.json", data)
            .await
            .map_err(|e| err_with_context!(e, "Writing currentblock.json"))?;
        Ok(())
    }
}
