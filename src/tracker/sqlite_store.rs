use anyhow::Result;
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Mutex;

use super::Checkpoint;

pub struct CheckpointStore {
    conn: Mutex<Connection>,
}

impl CheckpointStore {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        let store = Self {
            conn: Mutex::new(conn),
        };
        store.initialize_schema()?;
        Ok(store)
    }

    pub fn open_or_create(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Self::open(path)
    }

    fn initialize_schema(&self) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS checkpoints (
                id TEXT PRIMARY KEY,
                timestamp INTEGER NOT NULL,
                block_count INTEGER NOT NULL,
                tracking_method TEXT NOT NULL,
                created_at TEXT DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS dirty_blocks (
                block_number INTEGER NOT NULL,
                checkpoint_id TEXT NOT NULL,
                hash TEXT NOT NULL,
                PRIMARY KEY (block_number, checkpoint_id),
                FOREIGN KEY (checkpoint_id) REFERENCES checkpoints(id)
            );

            CREATE INDEX IF NOT EXISTS idx_dirty_blocks_checkpoint
                ON dirty_blocks(checkpoint_id);",
        )?;
        Ok(())
    }

    pub fn save_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        conn.execute(
            "INSERT OR REPLACE INTO checkpoints (id, timestamp, block_count, tracking_method)
             VALUES (?1, ?2, ?3, ?4)",
            params![checkpoint.id, checkpoint.timestamp, checkpoint.block_count, checkpoint.tracking_method],
        )?;
        Ok(())
    }

    pub fn get_latest_checkpoint(&self) -> Result<Option<Checkpoint>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, block_count, tracking_method
             FROM checkpoints ORDER BY timestamp DESC LIMIT 1",
        )?;

        let result = stmt.query_row([], |row| {
            Ok(Checkpoint {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                block_count: row.get(2)?,
                tracking_method: row.get(3)?,
            })
        });

        match result {
            Ok(cp) => Ok(Some(cp)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn record_dirty_block(&self, block_number: u64, checkpoint_id: &str, hash: &str) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        conn.execute(
            "INSERT OR REPLACE INTO dirty_blocks (block_number, checkpoint_id, hash)
             VALUES (?1, ?2, ?3)",
            params![block_number, checkpoint_id, hash],
        )?;
        Ok(())
    }

    pub fn get_dirty_blocks(&self, checkpoint_id: &str) -> Result<Vec<u64>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT block_number FROM dirty_blocks WHERE checkpoint_id = ?1 ORDER BY block_number",
        )?;

        let blocks: Vec<u64> = stmt
            .query_map(params![checkpoint_id], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(blocks)
    }

    pub fn get_all_checkpoints(&self) -> Result<Vec<Checkpoint>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, block_count, tracking_method
             FROM checkpoints ORDER BY timestamp ASC",
        )?;

        let checkpoints: Vec<Checkpoint> = stmt
            .query_map([], |row| {
                Ok(Checkpoint {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    block_count: row.get(2)?,
                    tracking_method: row.get(3)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(checkpoints)
    }

    pub fn clear_checkpoints(&self) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        conn.execute_batch("DELETE FROM dirty_blocks; DELETE FROM checkpoints;")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_checkpoint_persistence() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("checkpoints.db");
        let store = CheckpointStore::open(&path).unwrap();

        let cp = Checkpoint {
            id: "test-1".to_string(),
            timestamp: 1000,
            block_count: 42,
            tracking_method: "bitmap".to_string(),
        };
        store.save_checkpoint(&cp).unwrap();

        let loaded = store.get_latest_checkpoint().unwrap().unwrap();
        assert_eq!(loaded.id, "test-1");
        assert_eq!(loaded.block_count, 42);
    }

    #[test]
    fn test_dirty_blocks() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("dirty.db");
        let store = CheckpointStore::open(&path).unwrap();

        let cp = Checkpoint {
            id: "cp1".to_string(),
            timestamp: 1000,
            block_count: 0,
            tracking_method: "bitmap".to_string(),
        };
        store.save_checkpoint(&cp).unwrap();

        store.record_dirty_block(100, "cp1", "hash1").unwrap();
        store.record_dirty_block(200, "cp1", "hash2").unwrap();

        let blocks = store.get_dirty_blocks("cp1").unwrap();
        assert_eq!(blocks, vec![100, 200]);
    }
}
