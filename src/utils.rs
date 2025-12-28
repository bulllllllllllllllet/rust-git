use sha2::{Digest, Sha256};
use std::path::Path;
use std::fs;
use glob::Pattern;

pub fn hash_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

pub struct GitIgnore {
    patterns: Vec<Pattern>,
}

impl GitIgnore {
    pub fn new() -> Self {
        let mut patterns = Vec::new();
        if let Ok(content) = fs::read_to_string(".gitignore") {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                // Handle basic directory matching by appending /*
                let pattern_str = if line.ends_with('/') {
                    format!("{}*", line) // simple assumption
                } else {
                    line.to_string()
                };
                
                if let Ok(p) = Pattern::new(&pattern_str) {
                    patterns.push(p);
                }
            }
        }
        Self { patterns }
    }
    
    pub fn is_ignored(&self, path: &str) -> bool {
        // Always ignore .git
        if path.starts_with(".git") || path.contains("/.git") {
            return true;
        }
        
        for pattern in &self.patterns {
            // Check if path matches pattern
            // Note: glob patterns usually match the whole string or filename.
            // A proper implementation is complex. 
            // Here we check if the path (relative to root) matches the pattern.
            if pattern.matches(path) {
                return true;
            }
            // Also check if any parent directory matches (if pattern ends with /** or similar? No, simple glob)
            // If we have "target" in gitignore, we want "target/foo" to match.
            // Pattern::new("target").matches("target/foo") is false.
            // So we might need to be smarter.
            
            // Hacky support for directory ignores:
            // If pattern is "target", check if path starts with "target/"
            let p_str = pattern.as_str();
            if path.starts_with(&format!("{}/", p_str)) || path == p_str {
                return true;
            }
        }
        false
    }
}
