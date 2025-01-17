use std::path::Path;

use crate::{
    err::{Error, ErrorType},
    err_with_context,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BlockType {
    pub id: u8,
    pub name: String,
    pub color: Color,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewBlockType {
    name: String,
    color: Color,
}

pub trait PushNew<T> {
    fn push_new(&mut self, new: T);
}

impl PushNew<NewBlockType> for Vec<BlockType> {
    fn push_new(&mut self, new: NewBlockType) {
        let id = self.iter().map(|b| b.id).max().unwrap_or(0) + 1;
        let blocktype = BlockType {
            id,
            name: new.name,
            color: new.color,
        };
        self.push(blocktype);
    }
}

impl BlockType {
    pub async fn save(data_dir: &Path, types: &[Self]) -> Result<(), Error> {
        if Self::check_identical(types) {
            return Err(Error {
                error_type: ErrorType::IdenticalBlockType,
                file: file!(),
                line: line!(),
                column: column!(),
                additional: None,
            });
        }
        let blocktypes_path = data_dir.join("blocktypes.json");
        let contents = serde_json::to_string_pretty(&types)
            .map_err(|e| err_with_context!(e, "Serializing to {}", blocktypes_path.display()))?;
        tokio::fs::write(&blocktypes_path, contents)
            .await
            .map_err(|e| err_with_context!(e, "Writing to blocktypes.json"))?;
        Ok(())
    }

    pub async fn load(data_dir: &Path) -> Result<Vec<Self>, Error> {
        let blocktypes_path = data_dir.join("blocktypes.json");
        if !blocktypes_path.exists() {
            let blocktypes = vec![BlockType {
                id: 0,
                name: "System".to_string(),
                color: Color { r: 0, g: 0, b: 255 },
            }];
            BlockType::save(data_dir, &blocktypes).await?;
            return Ok(blocktypes);
        }
        let content = tokio::fs::read_to_string(&blocktypes_path)
            .await
            .map_err(|e| err_with_context!(e, "Reading {}", blocktypes_path.display()))?;
        let blocktypes = serde_json::from_str::<Vec<Self>>(&content)
            .map_err(|e| err_with_context!(e, "Deserializing {}", blocktypes_path.display()))?;
        Ok(blocktypes)
    }

    fn check_identical(blocktypes: &[Self]) -> bool {
        blocktypes.windows(2).any(|w| w[0] == w[1])
    }
}
