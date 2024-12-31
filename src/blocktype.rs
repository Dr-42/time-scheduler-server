use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum BlockTypeError {
    Identical,
    Tokio(tokio::io::Error),
    Serde(serde_json::Error),
}

impl std::fmt::Display for BlockTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockTypeError::Identical => write!(f, "Block types are identical"),
            BlockTypeError::Tokio(e) => write!(f, "Tokio error: {}", e),
            BlockTypeError::Serde(e) => write!(f, "Serde error: {}", e),
        }
    }
}

impl From<std::io::Error> for BlockTypeError {
    fn from(e: std::io::Error) -> Self {
        BlockTypeError::Tokio(e)
    }
}

impl From<serde_json::Error> for BlockTypeError {
    fn from(e: serde_json::Error) -> Self {
        BlockTypeError::Serde(e)
    }
}

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
    pub async fn save(types: Vec<Self>) -> Result<(), BlockTypeError> {
        if Self::check_identical(&types) {
            return Err(BlockTypeError::Identical);
        }
        let contents = serde_json::to_string_pretty(&types)?;
        tokio::fs::write("blocktypes.json", contents).await?;
        Ok(())
    }

    pub async fn load() -> Result<Vec<Self>, BlockTypeError> {
        if !std::path::Path::new("blocktypes.json").exists() {
            std::fs::File::create("blocktypes.json")?;
            return Ok(Vec::new());
        }
        let content = tokio::fs::read_to_string("blocktypes.json").await?;
        let blocktypes = serde_json::from_str::<Vec<Self>>(&content)?;
        Ok(blocktypes)
    }

    fn check_identical(blocktypes: &[Self]) -> bool {
        blocktypes.windows(2).any(|w| w[0] == w[1])
    }
}
