use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn init() -> Result<()> {
    let git_dir = Path::new(".git");
    if git_dir.exists() {
        println!("Git repository already initialized");
        return Ok(());
    }

    fs::create_dir(git_dir)?;
    fs::create_dir(git_dir.join("objects"))?;
    fs::create_dir(git_dir.join("refs"))?;
    fs::create_dir(git_dir.join("refs/heads"))?;

    fs::write(git_dir.join("HEAD"), "ref: refs/heads/master\n")?;

    println!("Initialized empty Git repository in .git/");
    Ok(())
}

use crate::objects::{GitObject, Index, Commit, TreeEntry};
use walkdir::WalkDir;
use crate::utils::GitIgnore;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::BTreeMap;

pub fn add(path: &str) -> Result<()> {
    let mut index = Index::load()?;
    let ignore = GitIgnore::new();
    
    for entry in WalkDir::new(path) {
        let entry = entry?;
        let path_str = entry.path().to_str().unwrap();
        
        // Remove "./" prefix if present for cleaner paths
        let clean_path = if path_str.starts_with("./") {
            &path_str[2..]
        } else {
            path_str
        };
        
        if ignore.is_ignored(clean_path) {
            continue;
        }

        if entry.file_type().is_file() {
            if let Ok(content) = fs::read_to_string(path_str) {
                let blob = GitObject::Blob(content);
                let hash = blob.save()?;
                
                let path_string = clean_path.to_string();
                let is_new_or_modified = match index.entries.get(&path_string) {
                    Some(old_hash) => *old_hash != hash,
                    None => true,
                };

                if is_new_or_modified {
                    index.entries.insert(path_string, hash);
                    println!("Added {}", clean_path);
                }
            } else {
                println!("Skipping binary or unreadable file: {}", path_str);
            }
        }
    }
    
    index.save()?;
    Ok(())
}

pub fn commit(message: &str) -> Result<()> {
    let index = Index::load()?;
    let root_hash = build_root_tree(&index)?;
    
    let parents = if let Some(p) = get_head_commit()? {
        vec![p]
    } else {
        vec![]
    };
    
    let commit = Commit {
        tree: root_hash,
        parents,
        author: "User <user@example.com>".to_string(),
        message: message.to_string(),
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
    };
    
    let commit_hash = GitObject::Commit(commit).save()?;
    
    update_head(&commit_hash)?;
    
    println!("Computed commit {}", commit_hash);
    Ok(())
}

fn get_head_commit() -> Result<Option<String>> {
    let git_dir = Path::new(".git");
    let head_path = git_dir.join("HEAD");
    let head_content = fs::read_to_string(head_path)?;
    let head_content = head_content.trim();
    
    if head_content.starts_with("ref: ") {
        let ref_path = git_dir.join(&head_content[5..]);
        if ref_path.exists() {
            let hash = fs::read_to_string(ref_path)?;
            Ok(Some(hash.trim().to_string()))
        } else {
            Ok(None)
        }
    } else {
        Ok(Some(head_content.to_string()))
    }
}

fn update_head(commit_hash: &str) -> Result<()> {
    let git_dir = Path::new(".git");
    let head_path = git_dir.join("HEAD");
    let head_content = fs::read_to_string(&head_path)?;
    let head_content = head_content.trim();
    
    if head_content.starts_with("ref: ") {
        let ref_path = git_dir.join(&head_content[5..]);
        if let Some(parent) = ref_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(ref_path, commit_hash)?;
    } else {
        fs::write(head_path, commit_hash)?;
    }
    Ok(())
}

struct InMemoryTree {
    blobs: BTreeMap<String, String>, 
    trees: BTreeMap<String, InMemoryTree>,
}

impl InMemoryTree {
    fn new() -> Self {
        Self { blobs: BTreeMap::new(), trees: BTreeMap::new() }
    }
    
    fn insert(&mut self, path: &str, hash: &str) {
        if let Some((dir, rest)) = path.split_once('/') {
            // It's in a subdirectory
            self.trees.entry(dir.to_string()).or_insert_with(InMemoryTree::new).insert(rest, hash);
        } else {
            // It's a file in this directory
            self.blobs.insert(path.to_string(), hash.to_string());
        }
    }
    
    fn write(&self) -> Result<String> {
        let mut entries = Vec::new();
        
        for (name, hash) in &self.blobs {
            entries.push(TreeEntry {
                mode: "100644".to_string(),
                name: name.clone(),
                hash: hash.clone(),
            });
        }
        
        for (name, tree) in &self.trees {
            let hash = tree.write()?;
            entries.push(TreeEntry {
                mode: "040000".to_string(),
                name: name.clone(),
                hash,
            });
        }
        
        let tree_obj = GitObject::Tree(entries);
        tree_obj.save()
    }
}

fn build_root_tree(index: &Index) -> Result<String> {
    let mut root = InMemoryTree::new();
    for (path, hash) in &index.entries {
        root.insert(path, hash);
    }
    root.write()
}

pub fn rm(path: &str) -> Result<()> {
    let mut index = Index::load()?;
    index.entries.remove(path);
    index.save()?;
    
    if Path::new(path).exists() {
        fs::remove_file(path)?;
    }
    
    println!("Removed {}", path);
    Ok(())
}

pub fn log() -> Result<()> {
    let mut current_hash = get_head_commit()?;

    while let Some(hash) = current_hash {
        let obj = GitObject::load(&hash)?;
        if let GitObject::Commit(commit) = obj {
            println!("commit {}", hash);
            println!("Author: {}", commit.author);
            println!("Date:   {} (Unix Timestamp)", commit.timestamp);
            println!("\n    {}\n", commit.message);

            current_hash = commit.parents.first().cloned();
        } else {
            break;
        }
    }

    Ok(())
}

pub fn branch(name: &str) -> Result<()> {
    let head = get_head_commit()?;
    if let Some(hash) = head {
        let branch_path = Path::new(".git/refs/heads").join(name);
        if let Some(parent) = branch_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(branch_path, hash)?;
        println!("Created branch {}", name);
    } else {
        println!("No commits yet, cannot create branch");
    }
    Ok(())
}

pub fn checkout(name: &str) -> Result<()> {
    let git_dir = Path::new(".git");
    let branch_path = git_dir.join("refs/heads").join(name);
    
    if !branch_path.exists() {
        println!("Branch {} does not exist", name);
        return Ok(());
    }
    
    // Update HEAD
    fs::write(git_dir.join("HEAD"), format!("ref: refs/heads/{}\n", name))?;
    
    // Restore files
    let commit_hash = fs::read_to_string(branch_path)?;
    let commit_hash = commit_hash.trim();
    
    restore_working_directory(commit_hash)?;
    
    println!("Switched to branch {}", name);
    Ok(())
}

fn restore_working_directory(commit_hash: &str) -> Result<()> {
    let commit: Commit = match GitObject::load(commit_hash)? {
        GitObject::Commit(c) => c,
        _ => return Err(anyhow::anyhow!("Not a commit")),
    };
    
    let tree_hash = commit.tree;
    restore_tree(&tree_hash, Path::new("."))?;
    
    let mut index = Index::default();
    build_index_from_tree(&tree_hash, Path::new(""), &mut index)?;
    index.save()?;
    
    Ok(())
}

fn restore_tree(tree_hash: &str, current_path: &Path) -> Result<()> {
    let entries = match GitObject::load(tree_hash)? {
        GitObject::Tree(e) => e,
        _ => return Err(anyhow::anyhow!("Not a tree")),
    };
    
    for entry in entries {
        let path = current_path.join(&entry.name);
        if entry.mode == "040000" {
            // Directory
            if !path.exists() {
                fs::create_dir(&path)?;
            }
            restore_tree(&entry.hash, &path)?;
        } else {
            // Blob
            let content = match GitObject::load(&entry.hash)? {
                GitObject::Blob(c) => c,
                _ => return Err(anyhow::anyhow!("Not a blob")),
            };
            fs::write(&path, content)?;
        }
    }
    Ok(())
}

fn build_index_from_tree(tree_hash: &str, current_path: &Path, index: &mut Index) -> Result<()> {
    let entries = match GitObject::load(tree_hash)? {
        GitObject::Tree(e) => e,
        _ => return Err(anyhow::anyhow!("Not a tree")),
    };
    
    for entry in entries {
        let path = current_path.join(&entry.name);
        if entry.mode == "040000" {
            build_index_from_tree(&entry.hash, &path, index)?;
        } else {
             let path_str = path.to_str().unwrap().to_string();
             index.entries.insert(path_str, entry.hash.clone());
        }
    }
    Ok(())
}

pub fn merge(branch: &str) -> Result<()> {
    let head = get_head_commit()?;
    let head_hash = head.ok_or_else(|| anyhow::anyhow!("Nothing to merge into"))?;
    
    let branch_path = Path::new(".git/refs/heads").join(branch);
    if !branch_path.exists() {
        return Err(anyhow::anyhow!("Branch {} does not exist", branch));
    }
    let branch_hash = fs::read_to_string(branch_path)?;
    let branch_hash = branch_hash.trim().to_string();
    
    if head_hash == branch_hash {
        println!("Already up to date.");
        return Ok(());
    }
    
    if is_ancestor(&head_hash, &branch_hash)? {
        // Fast-forward
        println!("Fast-forward merge");
        update_head(&branch_hash)?;
        restore_working_directory(&branch_hash)?;
    } else {
        return Err(anyhow::anyhow!("Non-fast-forward merge not supported yet"));
    }
    Ok(())
}

fn is_ancestor(ancestor: &str, descendant: &str) -> Result<bool> {
    if ancestor == descendant {
        return Ok(true);
    }
    let commit = match GitObject::load(descendant)? {
        GitObject::Commit(c) => c,
        _ => return Ok(false),
    };
    for parent in commit.parents {
        if is_ancestor(ancestor, &parent)? {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn status() -> Result<()> {
    let index = Index::load()?;
    // We use a BTreeMap to keep output sorted
    let mut index_entries: BTreeMap<String, String> = index.entries.iter().map(|(k,v)| (k.clone(), v.clone())).collect();
    
    let mut untracked = Vec::new();
    let mut modified = Vec::new();
    let mut deleted = Vec::new();
    
    let ignore = GitIgnore::new();

    // 1. Check for untracked files
    for entry in WalkDir::new(".") {
        let entry = entry?;
        let path_str = entry.path().to_str().unwrap();
        
        // Remove "./" prefix
        let clean_path = if path_str.starts_with("./") {
            &path_str[2..]
        } else {
            path_str
        };

        if ignore.is_ignored(clean_path) {
            continue;
        }
        
        if entry.file_type().is_file() {
            // If not in index, it's untracked
            if !index_entries.contains_key(clean_path) {
                untracked.push(clean_path.to_string());
            }
        }
    }

    // 2. Check for modified and deleted files (iterating over Index)
    let entries_to_check: Vec<(String, String)> = index_entries.iter().map(|(k,v)| (k.clone(), v.clone())).collect();

    for (path, index_hash) in entries_to_check {
        if Path::new(&path).exists() {
             if let Ok(content) = fs::read_to_string(&path) {
                let blob = GitObject::Blob(content);
                // Calculate hash without saving
                let json = serde_json::to_string(&blob)?;
                let current_hash = crate::utils::hash_content(&json);
                
                if current_hash != index_hash {
                    modified.push(path);
                }
            }
        } else {
            deleted.push(path);
        }
    }
    
    if !modified.is_empty() || !deleted.is_empty() {
        println!("Changes not staged for commit:");
        for path in &modified {
            println!("\tmodified: {}", path);
        }
        for path in &deleted {
            println!("\tdeleted:  {}", path);
        }
        println!();
    }
    
    if !untracked.is_empty() {
        println!("Untracked files:");
        for path in &untracked {
            println!("\t{}", path);
        }
        println!();
    }
    
    if modified.is_empty() && deleted.is_empty() && untracked.is_empty() {
        println!("nothing to commit, working tree clean");
    }
    
    Ok(())
}

use similar::{ChangeTag, TextDiff};

pub fn diff() -> Result<()> {
    let index = Index::load()?;
    
    for (path, hash) in &index.entries {
        if Path::new(path).exists() {
             // 1. Get content from Working Directory
            let content_wd = fs::read_to_string(path)?;
            
            // 2. Get content from Index (Blob)
            let obj = GitObject::load(hash)?;
            let content_index = match obj {
                GitObject::Blob(content) => content,
                _ => continue, // Should not happen for file entries in index
            };
            
            // 3. Compare
            if content_wd != content_index {
                println!("diff --git a/{} b/{}", path, path);
                println!("--- a/{}", path);
                println!("+++ b/{}", path);
                
                let diff = TextDiff::from_lines(&content_index, &content_wd);
                
                for change in diff.iter_all_changes() {
                    let sign = match change.tag() {
                        ChangeTag::Delete => "-",
                        ChangeTag::Insert => "+",
                        ChangeTag::Equal => " ",
                    };
                    print!("{}{}", sign, change);
                }
                println!();
            }
        }
    }
    Ok(())
}
