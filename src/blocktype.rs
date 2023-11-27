use crate::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockType {
    pub id: u8,
    pub name: String,
    pub color: Color,
}

impl BlockType {
    pub fn new(id: u8, name: String, color: Color) -> Self {
        Self { id, name, color }
    }

    pub fn save(&self) -> Result<()> {
        if !std::path::Path::new("blocktypes.json").exists() {
            std::fs::File::create("blocktypes.json")?;
        }
        let mut blocktypes = Self::load()?;
        if self.check_identical(&blocktypes) {
            return Err("Blocktype already exists".into());
        }
        blocktypes.push(self.clone());
        serde_json::to_writer_pretty(std::fs::File::create("blocktypes.json")?, &blocktypes)?;
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

    fn check_identical(&self, blocktypes: &[Self]) -> bool {
        for blocktype in blocktypes {
            if self.id == blocktype.id {
                eprintln!("Something went wrong: Blocktype ID already exists");
                return true;
            }
            if self.name == blocktype.name {
                return true;
            }
            if self.color == blocktype.color {
                return true;
            }
        }
        false
    }
}
