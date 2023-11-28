use crate::Result;
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

impl BlockType {
    pub fn new(id: u8, name: String, color: Color) -> Self {
        Self { id, name, color }
    }

    pub fn save(types: Vec<Self>) -> Result<()> {
        if !std::path::Path::new("blocktypes.json").exists() {
            std::fs::File::create("blocktypes.json")?;
            let system_block = Self::new(0, "System".to_string(), Color { r: 0, g: 0, b: 255 });
            serde_json::to_writer_pretty(
                std::fs::File::create("blocktypes.json")?,
                &[system_block],
            )?;
        }
        if Self::check_identical(&types) {
            return Err("Blocktype already exists".into());
        }
        serde_json::to_writer_pretty(std::fs::File::create("blocktypes.json")?, &types)?;
        Ok(())
    }

    pub fn load() -> Result<Vec<Self>> {
        if !std::path::Path::new("blocktypes.json").exists() {
            std::fs::File::create("blocktypes.json")?;
            return Ok(Vec::new());
        }
        let blocktypes =
            serde_json::from_str::<Vec<Self>>(&std::fs::read_to_string("blocktypes.json")?)?;
        Ok(blocktypes)
    }

    fn check_identical(blocktypes: &[Self]) -> bool {
        (1..blocktypes.len()).any(|i| blocktypes[i..].contains(&blocktypes[i - 1]))
    }
}
