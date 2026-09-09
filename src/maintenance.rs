//! SQLite backup and operational health reporting.
use crate::{embedding::MODEL, store::Store};
use anyhow::{Result, ensure};
use std::{fs, path::Path};
impl Store {
    pub fn backup(&self, output: &Path) -> Result<()> {
        ensure!(!output.exists(), "backup destination already exists");
        // Reserve with create_new so another process cannot replace an existing backup.
        let mut opts = fs::OpenOptions::new();
        opts.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        let _file = opts.open(output)?;
        self.conn.backup("main", output, None)?;
        Ok(())
    }
    pub fn doctor(&self) -> Result<serde_json::Value> {
        let check: String = self
            .conn
            .query_row("PRAGMA quick_check", [], |r| r.get(0))?;
        let entries: i64 = self
            .conn
            .query_row("SELECT count(*) FROM entries", [], |r| r.get(0))?;
        let vectors: i64 = self
            .conn
            .query_row("SELECT count(*) FROM vector_chunks", [], |r| r.get(0))?;
        let missing: i64 = self.conn.query_row(
            "SELECT count(*) FROM entries WHERE id NOT IN (SELECT entry_id FROM vector_chunks)",
            [],
            |r| r.get(0),
        )?;
        Ok(
            serde_json::json!({"integrity":check,"entries":entries,"vector_chunks":vectors,"unindexed_entries":missing,"embedding_model":MODEL,"schema_version":1,"vector_search":"exact cosine KNN (not ANN)"}),
        )
    }
}
