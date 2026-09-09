//! Project registry and scoped record reads.
use crate::{
    model::{Entry, Project, validate_project},
    store::Store,
};
use anyhow::{Context, Result, ensure};
use rusqlite::{OptionalExtension, params};
use std::path::Path;
impl Store {
    pub fn add_project(&self, id: &str, name: &str, root: &Path) -> Result<Project> {
        validate_project(id)?;
        ensure!(!name.trim().is_empty(), "project name is required");
        ensure!(root.is_dir(), "project root must be an existing directory");
        let root = root.canonicalize()?.to_string_lossy().into_owned();
        let tx = self.conn.unchecked_transaction()?;
        let existing: Option<String> = tx
            .query_row("SELECT project FROM roots WHERE path=?1", [&root], |r| {
                r.get(0)
            })
            .optional()?;
        ensure!(
            existing.as_deref().is_none_or(|p| p == id),
            "root is already assigned to a different project"
        );
        tx.execute("INSERT INTO projects(id,name) VALUES (?1,?2) ON CONFLICT(id) DO UPDATE SET name=excluded.name", params![id,name])?;
        tx.execute(
            "INSERT OR IGNORE INTO roots VALUES (?1,?2)",
            params![root, id],
        )?;
        tx.commit()?;
        Ok(self.projects()?.into_iter().find(|p| p.id == id).unwrap())
    }
    pub fn projects(&self) -> Result<Vec<Project>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id,name FROM projects ORDER BY id")?;
        let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
        let mut projects = Vec::new();
        for row in rows {
            let (id, name) = row?;
            let mut roots = self
                .conn
                .prepare("SELECT path FROM roots WHERE project=?1 ORDER BY path")?;
            let roots = roots
                .query_map([&id], |r| r.get(0))?
                .collect::<rusqlite::Result<Vec<String>>>()?;
            projects.push(Project { id, name, roots });
        }
        Ok(projects)
    }
    pub fn require_project(&self, project: &str) -> Result<()> {
        validate_project(project)?;
        ensure!(
            self.conn.query_row(
                "SELECT count(*) FROM projects WHERE id=?1",
                [project],
                |r| r.get::<_, i64>(0)
            )? == 1,
            "unknown project; register its root first"
        );
        Ok(())
    }
    pub fn get(&self, project: &str, id: &str) -> Result<Entry> {
        self.require_project(project)?;
        let result: Option<String> = self
            .conn
            .query_row(
                "SELECT json FROM entries WHERE project=?1 AND id=?2",
                params![project, id],
                |r| r.get(0),
            )
            .optional()?;
        serde_json::from_str(&result.context("entry not found in this project")?)
            .map_err(Into::into)
    }
    pub fn entries(&self, project: &str) -> Result<Vec<Entry>> {
        self.require_project(project)?;
        let mut stmt = self
            .conn
            .prepare("SELECT json FROM entries WHERE project=?1 ORDER BY created_at,id")?;
        let rows = stmt
            .query_map([project], |r| r.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        rows.into_iter()
            .map(|s| serde_json::from_str(&s).map_err(Into::into))
            .collect()
    }
    pub fn history(&self, project: &str, id: &str) -> Result<Vec<Entry>> {
        self.get(project, id)?;
        let mut stmt = self
            .conn
            .prepare("SELECT json FROM revisions WHERE project=?1 AND id=?2 ORDER BY revision")?;
        let rows = stmt
            .query_map(params![project, id], |r| r.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        rows.into_iter()
            .map(|s| serde_json::from_str(&s).map_err(Into::into))
            .collect()
    }
}
