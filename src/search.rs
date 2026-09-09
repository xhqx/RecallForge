//! Project-filtered lexical and vector retrieval, combined with reciprocal rank fusion.
use crate::{embedding::validate_vector, model::Source, store::Store};
use anyhow::{Result, ensure};
use rusqlite::params;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Serialize)]
pub struct Hit {
    pub id: String,
    pub revision: i64,
    pub title: String,
    pub status: String,
    pub score: f64,
    pub lexical_rank: Option<usize>,
    pub semantic_rank: Option<usize>,
    pub excerpt: String,
    pub sources: Vec<Source>,
}
#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub project: String,
    pub mode: String,
    pub hits: Vec<Hit>,
    pub warnings: Vec<String>,
}

/// Quote terms independently. User input is never executed as an FTS expression.
pub fn literal_query(text: &str) -> String {
    text.split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|s| !s.is_empty())
        .take(40)
        .map(|s| format!("\"{s}\""))
        .collect::<Vec<_>>()
        .join(" OR ")
}

impl Store {
    pub fn search(
        &self,
        project: &str,
        query: &str,
        mode: &str,
        limit: usize,
        inactive: bool,
        vector: Option<&[f32]>,
    ) -> Result<SearchResult> {
        self.require_project(project)?;
        ensure!(
            !query.trim().is_empty() && query.chars().count() <= 2000,
            "query must contain 1..2000 characters"
        );
        ensure!((1..=50).contains(&limit), "limit must be 1..50");
        ensure!(
            ["lexical", "semantic", "hybrid"].contains(&mode),
            "invalid search mode"
        );
        let pool = (limit * 20).min(1000) as i64;
        let mut ranks: BTreeMap<String, (Option<usize>, Option<usize>)> = BTreeMap::new();
        if mode != "semantic" {
            let q = literal_query(query);
            if !q.is_empty() {
                let mut stmt=self.conn.prepare("SELECT e.id FROM entries_fts JOIN entries e ON e.rowid=entries_fts.rowid WHERE entries_fts MATCH ?1 AND e.project=?2 AND (?3 OR e.status NOT IN ('stale','superseded')) ORDER BY bm25(entries_fts,3.0,1.0),e.id LIMIT ?4")?;
                let ids = stmt
                    .query_map(params![q, project, inactive, pool], |r| {
                        r.get::<_, String>(0)
                    })?
                    .collect::<rusqlite::Result<Vec<_>>>()?;
                for (i, id) in ids.into_iter().enumerate() {
                    ranks.entry(id).or_default().0 = Some(i + 1);
                }
            }
        }
        let mut warnings = vec![];
        if mode != "lexical" {
            let vector = vector
                .ok_or_else(|| anyhow::anyhow!("semantic search requires a query embedding"))?;
            validate_vector(vector)?;
            let conditions = if inactive {
                ""
            } else {
                " AND status != 'stale' AND status != 'superseded'"
            };
            let sql = format!(
                "SELECT c.entry_id FROM (SELECT rowid,distance FROM vectors WHERE embedding MATCH ?1 AND k=?2 AND project=?3{conditions} ORDER BY distance) v JOIN vector_chunks c ON c.rowid=v.rowid ORDER BY v.distance,c.entry_id"
            );
            let mut stmt = self.conn.prepare(&sql)?;
            let ids = stmt
                .query_map(
                    params![serde_json::to_string(vector)?, pool, project],
                    |r| r.get::<_, String>(0),
                )?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            let mut rank = 0;
            for id in ids {
                let item = ranks.entry(id).or_default();
                if item.1.is_none() {
                    rank += 1;
                    item.1 = Some(rank);
                }
            }
            let missing:i64=self.conn.query_row("SELECT count(*) FROM entries e WHERE project=?1 AND (?2 OR status NOT IN ('stale','superseded')) AND NOT EXISTS(SELECT 1 FROM vector_chunks c WHERE c.entry_id=e.id)",params![project,inactive],|r|r.get(0))?;
            if missing > 0 {
                warnings.push(format!(
                    "{missing} entries have no vector index; run reindex for this project"
                ));
            }
        }
        let mut sorted: Vec<_> = ranks
            .into_iter()
            .map(|(id, (lex, sem))| {
                let score = lex.map_or(0.0, |r| 1.0 / (60.0 + r as f64))
                    + sem.map_or(0.0, |r| 1.0 / (60.0 + r as f64));
                (id, lex, sem, score)
            })
            .collect();
        sorted.sort_by(|a, b| b.3.total_cmp(&a.3).then(a.0.cmp(&b.0)));
        let mut hits = Vec::new();
        for (id, lexical_rank, semantic_rank, score) in sorted.into_iter().take(limit) {
            let e = self.get(project, &id)?;
            hits.push(Hit {
                id,
                revision: e.revision,
                title: e.data.title,
                status: e.data.status,
                score,
                lexical_rank,
                semantic_rank,
                excerpt: format!("{} → {}", e.data.problem, e.data.solution)
                    .chars()
                    .take(450)
                    .collect(),
                sources: e.data.sources.into_iter().take(3).collect(),
            });
        }
        Ok(SearchResult {
            project: project.into(),
            mode: mode.into(),
            hits,
            warnings,
        })
    }
}
