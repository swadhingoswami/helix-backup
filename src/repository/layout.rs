use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;

use super::manifest::Manifest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSnapshot {
    pub id: String,
    pub timestamp: String,
    pub backup_type: String,
    pub block_count: u64,
    pub total_size: u64,
    pub label: String,
    pub parent_id: Option<String>,
}

pub struct Repository {
    root: PathBuf,
    block_size: u32,
}

impl Repository {
    pub fn open(path: &str) -> Result<Self> {
        let root = PathBuf::from(path);
        if !root.exists() {
            anyhow::bail!("Repository not found at: {}", path);
        }

        let metadata_path = root.join("metadata.json");
        if !metadata_path.exists() {
            anyhow::bail!("Invalid repository: metadata.json not found at {}", path);
        }

        let metadata_content = std::fs::read_to_string(&metadata_path)?;
        let metadata: Metadata = serde_json::from_str(&metadata_content)?;

        Ok(Self {
            root,
            block_size: metadata.block_size,
        })
    }

    pub fn open_or_create(path: &str) -> Result<Self> {
        let root = PathBuf::from(path);
        if root.join("metadata.json").exists() {
            return Self::open(path);
        }
        Self::initialize(path, None, 3)
    }

    pub fn initialize(path: &str, _key: Option<&str>, compression_level: i32) -> Result<Self> {
        let root = PathBuf::from(path);

        if root.exists() {
            let entries = std::fs::read_dir(&root)
                .into_iter()
                .flatten()
                .flatten()
                .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
                .count();
            if entries > 0 {
                anyhow::bail!("Directory not empty: {}", path);
            }
        }

        fs2::ensure_dir_exists(&root)?;
        fs2::ensure_dir_exists(&root.join("Full"))?;
        fs2::ensure_dir_exists(&root.join("Incremental"))?;

        let metadata = Metadata {
            version: 1,
            created_at: chrono::Utc::now().to_rfc3339(),
            block_size: 4096,
            compression_level,
            encrypted: _key.is_some(),
            total_snapshots: 0,
        };

        let metadata_content = serde_json::to_string_pretty(&metadata)?;
        std::fs::write(root.join("metadata.json"), &metadata_content)?;

        // Initialize SQLite index
        let _index = super::index::IndexManager::open(&root.join("index.db"))?;

        log::info!("Repository initialized at {}", path);
        Ok(Self {
            root,
            block_size: 4096,
        })
    }

    pub fn create_snapshot(&self, label: &str, backup_type: &str) -> Result<String> {
        let id = format!("{}-{}", backup_type, uuid::Uuid::new_v4().to_string().split('-').next().unwrap());

        let snapshot_dir = if backup_type == "full" {
            self.root.join("Full").join(&id)
        } else {
            self.root.join("Incremental").join(&id)
        };

        fs2::ensure_dir_exists(&snapshot_dir)?;

        let parent_id = if backup_type == "incremental" {
            self.list_backups()?.into_iter().rev().find(|b| b.backup_type == "full").map(|b| b.id)
        } else {
            None
        };

        let snapshot = BackupSnapshot {
            id: id.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            backup_type: backup_type.to_string(),
            block_count: 0,
            total_size: 0,
            label: label.to_string(),
            parent_id: parent_id.clone(),
        };

        let mut manifest = Manifest::new(&id, backup_type, self.block_size);
        manifest.label = label.to_string();
        manifest.parent_id = parent_id;
        let manifest_content = serde_json::to_string_pretty(&manifest)?;
        std::fs::write(snapshot_dir.join("manifest.json"), &manifest_content)?;

        self.update_metadata(|m| m.total_snapshots += 1)?;

        if let Ok(index) = super::index::IndexManager::open(&self.root.join("index.db")) {
            index.record_snapshot(&snapshot)?;
        }

        Ok(id)
    }

    pub fn finalize_snapshot(&self, snapshot_id: &str) -> Result<()> {
        let manifest_path = self.find_manifest_path(snapshot_id)?;
        let content = std::fs::read_to_string(&manifest_path)?;
        let mut manifest: Manifest = serde_json::from_str(&content)?;
        manifest.completed = true;
        let new_content = serde_json::to_string_pretty(&manifest)?;
        std::fs::write(&manifest_path, new_content)?;
        Ok(())
    }

    pub fn write_blocks(&self, snapshot_id: &str, blocks: &[(u64, Vec<u8>)]) -> Result<()> {
        let snapshot_dir = self.snapshot_dir(snapshot_id)?;
        let data_path = snapshot_dir.join("blocks.dat");

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&data_path)?;

        for (block_num, data) in blocks {
            file.write_all(&block_num.to_le_bytes())?;
            let size = data.len() as u32;
            file.write_all(&size.to_le_bytes())?;
            file.write_all(data)?;
        }

        Ok(())
    }

    pub fn read_blocks_map(&self, snapshot_id: &str) -> Result<HashMap<u64, Vec<u8>>> {
        let snapshot_dir = self.snapshot_dir(snapshot_id)?;
        let data_path = snapshot_dir.join("blocks.dat");

        let mut file = std::fs::File::open(&data_path)?;
        let mut map = HashMap::new();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let mut offset = 0;
        while offset + 12 <= buffer.len() {
            let block_num = u64::from_le_bytes(
                buffer[offset..offset + 8].try_into().unwrap(),
            );
            let data_size = u32::from_le_bytes(
                buffer[offset + 8..offset + 12].try_into().unwrap(),
            ) as usize;
            offset += 12;

            if offset + data_size > buffer.len() {
                break;
            }

            let data = buffer[offset..offset + data_size].to_vec();
            map.insert(block_num, data);
            offset += data_size;
        }

        Ok(map)
    }

    pub fn store_block_hashes(&self, snapshot_id: &str, hashes: &[crate::block::hasher::BlockHash]) -> Result<()> {
        let manifest_path = self.find_manifest_path(snapshot_id)?;
        let content = std::fs::read_to_string(&manifest_path)?;
        let mut manifest: Manifest = serde_json::from_str(&content)?;

        for hash in hashes {
            manifest.block_hashes.push(hash.clone());
        }

        manifest.block_count = manifest.block_hashes.len() as u64;

        let new_content = serde_json::to_string_pretty(&manifest)?;
        std::fs::write(&manifest_path, new_content)?;
        Ok(())
    }

    pub fn load_manifest(&self, snapshot_id: &str) -> Result<Manifest> {
        let manifest_path = self.find_manifest_path(snapshot_id)?;
        let content = std::fs::read_to_string(&manifest_path)?;
        let manifest: Manifest = serde_json::from_str(&content)?;
        Ok(manifest)
    }

    fn snapshot_dir(&self, snapshot_id: &str) -> Result<PathBuf> {
        let candidates = [
            self.root.join("Full").join(snapshot_id),
            self.root.join("Incremental").join(snapshot_id),
        ];
        for dir in &candidates {
            if dir.join("manifest.json").exists() {
                return Ok(dir.clone());
            }
        }
        anyhow::bail!("Snapshot '{}' not found", snapshot_id)
    }

    pub fn list_backups(&self) -> Result<Vec<BackupSnapshot>> {
        let mut backups = Vec::new();

        // List full backups
        let full_dir = self.root.join("Full");
        if full_dir.exists() {
            for entry in std::fs::read_dir(&full_dir)? {
                let entry = entry?;
                let manifest_path = entry.path().join("manifest.json");
                if manifest_path.exists() {
                    let content = std::fs::read_to_string(&manifest_path)?;
                    if let Ok(manifest) = serde_json::from_str::<Manifest>(&content) {
                        backups.push(BackupSnapshot {
                            id: manifest.snapshot_id.clone(),
                            timestamp: manifest.created_at.clone(),
                            backup_type: "full".to_string(),
                            block_count: manifest.block_count,
                            total_size: manifest.total_size,
                            label: manifest.label,
                            parent_id: None,
                        });
                    }
                }
            }
        }

        // List incremental backups
        let inc_dir = self.root.join("Incremental");
        if inc_dir.exists() {
            for entry in std::fs::read_dir(&inc_dir)? {
                let entry = entry?;
                let manifest_path = entry.path().join("manifest.json");
                if manifest_path.exists() {
                    let content = std::fs::read_to_string(&manifest_path)?;
                    if let Ok(manifest) = serde_json::from_str::<Manifest>(&content) {
                        backups.push(BackupSnapshot {
                            id: manifest.snapshot_id.clone(),
                            timestamp: manifest.created_at.clone(),
                            backup_type: "incremental".to_string(),
                            block_count: manifest.block_count,
                            total_size: manifest.total_size,
                            label: manifest.label,
                            parent_id: manifest.parent_id,
                        });
                    }
                }
            }
        }

        backups.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        Ok(backups)
    }

    pub fn build_restore_chain(&self, snapshot_id: &str) -> Result<Vec<String>> {
        let mut chain = Vec::new();
        let mut current_id = Some(snapshot_id.to_string());

        while let Some(id) = current_id {
            chain.push(id.clone());
            let manifest = self.load_manifest(&id)?;
            current_id = manifest.parent_id;
        }

        chain.reverse();
        Ok(chain)
    }

    pub fn validate(&self, _repair: bool) -> Result<ValidationResult> {
        let mut issues = Vec::new();
        let mut ok = true;

        // Check metadata.json
        let metadata_path = self.root.join("metadata.json");
        if !metadata_path.exists() {
            issues.push("metadata.json not found".to_string());
            ok = false;
        }

        // Check all manifest files
        let backups = self.list_backups()?;
        for backup in &backups {
            let manifest_path = if backup.backup_type == "full" {
                self.root.join("Full").join(&backup.id).join("manifest.json")
            } else {
                self.root.join("Incremental").join(&backup.id).join("manifest.json")
            };

            if !manifest_path.exists() {
                issues.push(format!("Missing manifest for snapshot: {}", backup.id));
                ok = false;
            }
        }

        Ok(ValidationResult { ok, issues })
    }

    pub fn last_checkpoint(&self) -> Result<Option<crate::tracker::Checkpoint>> {
        if let Ok(index) = super::index::IndexManager::open(&self.root.join("index.db")) {
            index.get_last_checkpoint()
        } else {
            Ok(None)
        }
    }

    pub fn save_checkpoint(&self, checkpoint: &crate::tracker::Checkpoint) -> Result<()> {
        if let Ok(index) = super::index::IndexManager::open(&self.root.join("index.db")) {
            index.save_checkpoint(checkpoint)?;
        }
        Ok(())
    }

    pub fn block_size(&self) -> u32 {
        self.block_size
    }

    fn find_manifest_path(&self, snapshot_id: &str) -> Result<PathBuf> {
        let candidates = [
            self.root.join("Full").join(snapshot_id).join("manifest.json"),
            self.root.join("Incremental").join(snapshot_id).join("manifest.json"),
        ];

        for path in &candidates {
            if path.exists() {
                return Ok(path.clone());
            }
        }

        anyhow::bail!("Snapshot '{}' not found", snapshot_id)
    }

    fn update_metadata<F>(&self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut Metadata),
    {
        let metadata_path = self.root.join("metadata.json");
        let content = std::fs::read_to_string(&metadata_path)?;
        let mut metadata: Metadata = serde_json::from_str(&content)?;
        updater(&mut metadata);
        let new_content = serde_json::to_string_pretty(&metadata)?;
        std::fs::write(&metadata_path, new_content)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Metadata {
    version: u32,
    created_at: String,
    block_size: u32,
    compression_level: i32,
    encrypted: bool,
    total_snapshots: u64,
}

pub struct ValidationResult {
    ok: bool,
    issues: Vec<String>,
}

impl ValidationResult {
    pub fn is_ok(&self) -> bool {
        self.ok
    }

    pub fn issues(&self) -> &[String] {
        &self.issues
    }
}

// Helper module for directory creation
mod fs2 {
    use anyhow::Result;
    use std::path::Path;

    pub fn ensure_dir_exists(path: &Path) -> Result<()> {
        if !path.exists() {
            std::fs::create_dir_all(path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_initialize_and_open() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("repo");

        Repository::initialize(path.to_str().unwrap(), None, 3).unwrap();
        let repo = Repository::open(path.to_str().unwrap()).unwrap();
        assert_eq!(repo.block_size(), 4096);
    }

    #[test]
    fn test_create_and_list_snapshots() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("repo");
        let repo = Repository::initialize(path.to_str().unwrap(), None, 3).unwrap();

        let id = repo.create_snapshot("test-backup", "full").unwrap();
        repo.finalize_snapshot(&id).unwrap();

        let backups = repo.list_backups().unwrap();
        assert_eq!(backups.len(), 1);
        assert_eq!(backups[0].label, "test-backup");
    }

    #[test]
    fn test_invalid_repository() {
        assert!(Repository::open("/nonexistent/path").is_err());
    }
}
