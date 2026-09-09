//! Typed project-local entry relationships.
use crate::store::Store;
use anyhow::{Result, ensure};
use rusqlite::params;
impl Store {
    pub fn link(&self, project: &str, from: &str, to: &str, relation: &str) -> Result<()> {
        ensure!(
            ["relates_to", "caused_by", "solved_by", "supersedes"].contains(&relation),
            "invalid relation"
        );
        ensure!(from != to, "self links are not allowed");
        self.get(project, from)?;
        self.get(project, to)?;
        self.conn.execute(
            "INSERT OR IGNORE INTO links VALUES (?1,?2,?3,?4)",
            params![project, from, to, relation],
        )?;
        Ok(())
    }
    pub fn neighbors(&self, project: &str, id: &str) -> Result<serde_json::Value> {
        self.get(project, id)?;
        let mut stmt=self.conn.prepare("SELECT source,target,relation FROM links WHERE project=?1 AND (source=?2 OR target=?2) ORDER BY source,target,relation")?;
        let rows = stmt
            .query_map(params![project, id], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(serde_json::Value::Array(rows.into_iter().map(|(from,to,relation)|serde_json::json!({"from":from,"to":to,"relation":relation})).collect()))
    }
}
