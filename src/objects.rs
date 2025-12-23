use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub enum GitObject {
    Blob(String), // Content
    Tree(Vec<TreeEntry>),
    Commit(Commit),
}

impl GitObject {
    pub fn save(&self) -> Result<String> {
        let content = serde_json::to_string(self)?;
        let hash = crate::utils::hash_content(&content);
        
        let git_dir = Path::new(".git");
        let objects_dir = git_dir.join("objects");
        let prefix = &hash[..2];
        let suffix = &hash[2..];
        let dir = objects_dir.join(prefix);
        
        if !dir.exists() {
            fs::create_dir(&dir)?;
        }
        
        fs::write(dir.join(suffix), content)?;
        Ok(hash)
    }

    pub fn load(hash: &str) -> Result<Self> {
        let git_dir = Path::new(".git");
        let objects_dir = git_dir.join("objects");
        let prefix = &hash[..2];
        let suffix = &hash[2..];
        let file_path = objects_dir.join(prefix).join(suffix);
        
        let content = fs::read_to_string(file_path)?;
        let obj = serde_json::from_str(&content)?;
        Ok(obj)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TreeEntry {
    pub mode: String,
    pub name: String,
    pub hash: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Commit {
    pub tree: String,
    pub parents: Vec<String>,
    pub author: String,
    pub message: String,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Index {
    pub entries: HashMap<String, String>, // path -> hash
}

impl Index {
    pub fn load() -> Result<Self> {
        let index_path = Path::new(".git/index");
        if index_path.exists() {
            let content = fs::read_to_string(index_path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(Index::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let index_path = Path::new(".git/index");
        let content = serde_json::to_string(self)?;
        fs::write(index_path, content)?;
        Ok(())
    }
}
