//! Transactional persistence, schema ownership, project isolation and optimistic revisions.
use crate::{
    embedding::{MODEL, validate_vector},
    model::{Entry, EntryInput},
};
use anyhow::{Result, bail, ensure};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use sha2::{Digest, Sha256};
use std::{fs, path::Path, sync::Once, time::Duration};

pub struct Store {
    pub(crate) conn: Connection,
}
static EXTENSION: Once = Once::new();

impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        EXTENSION.call_once(|| {
            // SAFETY: sqlite-vec documents this ABI-compatible, statically linked entrypoint.
            unsafe {
                rusqlite::ffi::sqlite3_auto_extension(Some(std::mem::transmute::<
                    *const (),
                    unsafe extern "C" fn(
                        *mut rusqlite::ffi::sqlite3,
                        *mut *mut std::ffi::c_char,
                        *const rusqlite::ffi::sqlite3_api_routines,
                    ) -> i32,
                >(
                    sqlite_vec::sqlite3_vec_init as *const (),
                )));
            }
        });
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        // Never adopt or migrate another application's SQLite database by filename alone.
        let application: i64 = conn.query_row("PRAGMA application_id", [], |r| r.get(0))?;
        let tables: i64 = conn.query_row(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
            [],
            |r| r.get(0),
        )?;
        ensure!(
            application == 0x52464F52 || (application == 0 && tables == 0),
            "not a RecallForge database"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
        }
        conn.busy_timeout(Duration::from_secs(10))?;
        conn.execute_batch(
            "PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;",
        )?;
        let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        ensure!(version <= 1, "database schema is newer than this binary");
        conn.execute_batch(include_str!("schema.sql"))?;
        let model: String = conn
            .query_row(
                "SELECT value FROM metadata WHERE key='embedding_model'",
                [],
                |r| r.get(0),
            )
            .optional()?
            .unwrap_or_default();
        ensure!(
            model.is_empty() || model == MODEL,
            "embedding model mismatch; use the matching binary or explicit data migration"
        );
        conn.execute(
            "INSERT OR IGNORE INTO metadata VALUES ('embedding_model',?1)",
            [MODEL],
        )?;
        Ok(Self { conn })
    }
    pub fn save(
        &mut self,
        project: &str,
        mut data: EntryInput,
        vectors: Option<Vec<Vec<f32>>>,
    ) -> Result<Entry> {
        self.require_project(project)?;
        data.validate()?;
        if let Some(vs) = &vectors {
            ensure!(!vs.is_empty(), "empty embedding batch");
            for v in vs {
                validate_vector(v)?;
            }
        }
        let id = data
            .id
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let expected = data.expected_revision;
        data.id = None;
        data.expected_revision = None;
        let fingerprint = format!("{:x}", Sha256::digest(serde_json::to_vec(&data)?));
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if expected.is_none() {
            let duplicate: Option<String> = tx
                .query_row(
                    "SELECT json FROM entries WHERE project=?1 AND fingerprint=?2",
                    params![project, fingerprint],
                    |r| r.get(0),
                )
                .optional()?;
            if let Some(json) = duplicate {
                let mut existing: Entry = serde_json::from_str(&json)?;
                if vectors.is_some() && !existing.indexed {
                    replace_vectors(&tx, project, &existing.id, &existing.data.status, vectors)?;
                    existing.indexed = true;
                    tx.execute(
                        "UPDATE entries SET json=?1 WHERE id=?2",
                        params![serde_json::to_string(&existing)?, existing.id],
                    )?;
                }
                tx.commit()?;
                return Ok(existing);
            }
        }
        let previous: Option<String> = tx
            .query_row(
                "SELECT json FROM entries WHERE project=?1 AND id=?2",
                params![project, id],
                |r| r.get(0),
            )
            .optional()?;
        let previous = previous
            .map(|s| serde_json::from_str::<Entry>(&s))
            .transpose()?;
        match (expected, &previous) {
            (Some(rev), Some(old)) => ensure!(
                rev == old.revision,
                "revision conflict: current revision is {}",
                old.revision
            ),
            (Some(_), None) => bail!("entry not found in this project"),
            (None, Some(_)) => bail!("entry already exists"),
            _ => {}
        }
        let now: String = tx.query_row("SELECT strftime('%Y-%m-%dT%H:%M:%fZ','now')", [], |r| {
            r.get(0)
        })?;
        let entry = Entry {
            id: id.clone(),
            project: project.into(),
            revision: previous.as_ref().map_or(1, |e| e.revision + 1),
            created_at: previous
                .as_ref()
                .map_or(now.clone(), |e| e.created_at.clone()),
            updated_at: now,
            indexed: vectors.is_some(),
            data,
        };
        let json = serde_json::to_string(&entry)?;
        tx.execute("INSERT INTO entries(id,project,revision,status,kind,title,search_text,fingerprint,json,created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10) ON CONFLICT(id) DO UPDATE SET revision=excluded.revision,status=excluded.status,kind=excluded.kind,title=excluded.title,search_text=excluded.search_text,fingerprint=excluded.fingerprint,json=excluded.json",params![id,project,entry.revision,entry.data.status,entry.data.kind,entry.data.title,entry.data.search_text(),fingerprint,json,entry.created_at])?;
        tx.execute(
            "INSERT INTO revisions VALUES (?1,?2,?3,?4)",
            params![project, id, entry.revision, json],
        )?;
        replace_vectors(&tx, project, &id, &entry.data.status, vectors)?;
        tx.commit()?;
        Ok(entry)
    }
    pub fn index_entry(&mut self, entry: &Entry, vectors: Vec<Vec<f32>>) -> Result<()> {
        ensure!(!vectors.is_empty(), "empty embedding batch");
        for v in &vectors {
            validate_vector(v)?;
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let current: i64 = tx.query_row(
            "SELECT revision FROM entries WHERE project=?1 AND id=?2",
            params![entry.project, entry.id],
            |r| r.get(0),
        )?;
        ensure!(
            current == entry.revision,
            "entry changed during reindex; retry"
        );
        replace_vectors(
            &tx,
            &entry.project,
            &entry.id,
            &entry.data.status,
            Some(vectors),
        )?;
        let mut updated = entry.clone();
        updated.indexed = true;
        tx.execute(
            "UPDATE entries SET json=?1 WHERE id=?2",
            params![serde_json::to_string(&updated)?, entry.id],
        )?;
        tx.commit()?;
        Ok(())
    }
}

fn replace_vectors(
    tx: &rusqlite::Transaction<'_>,
    project: &str,
    id: &str,
    status: &str,
    vectors: Option<Vec<Vec<f32>>>,
) -> Result<()> {
    tx.execute(
        "DELETE FROM vectors WHERE rowid IN (SELECT rowid FROM vector_chunks WHERE entry_id=?1)",
        [id],
    )?;
    tx.execute("DELETE FROM vector_chunks WHERE entry_id=?1", [id])?;
    if let Some(vectors) = vectors {
        for v in vectors {
            tx.execute("INSERT INTO vector_chunks(entry_id) VALUES (?1)", [id])?;
            let rowid = tx.last_insert_rowid();
            tx.execute(
                "INSERT INTO vectors(rowid,embedding,project,status) VALUES (?1,?2,?3,?4)",
                params![rowid, serde_json::to_string(&v)?, project, status],
            )?;
        }
    }
    Ok(())
}
