use anyhow::Result;
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Mutex;

use super::layout::BackupSnapshot;
use crate::tracker::Checkpoint;

pub struct IndexManager {
    conn: Mutex<Connection>,
}

impl IndexManager {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        let manager = Self {
            conn: Mutex::new(conn),
        };
        manager.initialize()?;
        Ok(manager)
    }

    fn initialize(&self) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS snapshots (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                backup_type TEXT NOT NULL,
                block_count INTEGER NOT NULL DEFAULT 0,
                total_size INTEGER NOT NULL DEFAULT 0,
                label TEXT NOT NULL DEFAULT '',
                parent_id TEXT,
                created_at TEXT DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS checkpoints (
                id TEXT PRIMARY KEY,
                timestamp INTEGER NOT NULL,
                block_count INTEGER NOT NULL DEFAULT 0,
                tracking_method TEXT NOT NULL DEFAULT 'bitmap',
                created_at TEXT DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS block_index (
                block_number INTEGER NOT NULL,
                snapshot_id TEXT NOT NULL,
                offset INTEGER NOT NULL,
                size INTEGER NOT NULL,
                hash TEXT NOT NULL,
                PRIMARY KEY (block_number, snapshot_id),
                FOREIGN KEY (snapshot_id) REFERENCES snapshots(id)
            );

            CREATE INDEX IF NOT EXISTS idx_block_index_snapshot
                ON block_index(snapshot_id);

            CREATE INDEX IF NOT EXISTS idx_snapshots_timestamp
                ON snapshots(timestamp);",
        )?;
        Ok(())
    }

    pub fn record_snapshot(&self, snapshot: &BackupSnapshot) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        conn.execute(
            "INSERT OR REPLACE INTO snapshots (id, timestamp, backup_type, block_count, total_size, label, parent_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                snapshot.id,
                snapshot.timestamp,
                snapshot.backup_type,
                snapshot.block_count,
                snapshot.total_size,
                snapshot.label,
                snapshot.parent_id,
            ],
        )?;
        Ok(())
    }

    pub fn get_snapshots(&self) -> Result<Vec<BackupSnapshot>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, backup_type, block_count, total_size, label, parent_id
             FROM snapshots ORDER BY timestamp ASC",
        )?;

        let snapshots = stmt
            .query_map([], |row| {
                Ok(BackupSnapshot {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    backup_type: row.get(2)?,
                    block_count: row.get(3)?,
                    total_size: row.get(4)?,
                    label: row.get(5)?,
                    parent_id: row.get(6)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(snapshots)
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

    pub fn get_last_checkpoint(&self) -> Result<Option<Checkpoint>> {
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

    pub fn record_block(&self, block_number: u64, snapshot_id: &str, offset: u64, size: u32, hash: &str) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        conn.execute(
            "INSERT OR REPLACE INTO block_index (block_number, snapshot_id, offset, size, hash)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![block_number, snapshot_id, offset, size, hash],
        )?;
        Ok(())
    }

    pub fn get_blocks_for_snapshot(&self, snapshot_id: &str) -> Result<Vec<(u64, u64, u32, String)>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT block_number, offset, size, hash
             FROM block_index WHERE snapshot_id = ?1 ORDER BY block_number",
        )?;

        let blocks = stmt
            .query_map(params![snapshot_id], |row| {
                Ok((
                    row.get::<_, u64>(0)?,
                    row.get::<_, u64>(1)?,
                    row.get::<_, u32>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(blocks)
    }

    pub fn get_statistics(&self) -> Result<IndexStatistics> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;

        let total_snapshots: u64 = conn.query_row(
            "SELECT COUNT(*) FROM snapshots",
            [],
            |row| row.get(0),
        )?;

        let total_blocks: u64 = conn.query_row(
            "SELECT COALESCE(SUM(block_count), 0) FROM snapshots",
            [],
            |row| row.get(0),
        )?;

        Ok(IndexStatistics {
            total_snapshots,
            total_blocks,
        })
    }
}

pub struct IndexStatistics {
    pub total_snapshots: u64,
    pub total_blocks: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_index_basics() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("index.db");
        let index = IndexManager::open(&path).unwrap();

        let snapshot = BackupSnapshot {
            id: "test-1".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            backup_type: "full".to_string(),
            block_count: 100,
            total_size: 409600,
            label: "test".to_string(),
            parent_id: None,
        };

        index.record_snapshot(&snapshot).unwrap();
        let snapshots = index.get_snapshots().unwrap();
        assert_eq!(snapshots.len(), 1);

        let stats = index.get_statistics().unwrap();
        assert_eq!(stats.total_snapshots, 1);
    }
}
