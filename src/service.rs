//! One application boundary shared by CLI and MCP. No client writes SQL directly.
use crate::{embedding::Embedder, model::EntryInput, store::Store};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct Service {
    pub store: Store,
    embedder: Embedder,
    pub data_dir: PathBuf,
}
impl Service {
    pub fn open(dir: PathBuf) -> Result<Self> {
        #[cfg(unix)]
        let existed = dir.exists();
        fs::create_dir_all(&dir)?;
        #[cfg(unix)]
        if !existed {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&dir, fs::Permissions::from_mode(0o700))?;
        }
        Ok(Self {
            store: Store::open(&dir.join("memory.db"))?,
            embedder: Embedder::new(
                directories::ProjectDirs::from("dev", "RecallForge", "RecallForge")
                    .map(|d| d.cache_dir().join("models"))
                    .unwrap_or_else(|| dir.join("models")),
            ),
            data_dir: dir,
        })
    }
    pub fn call(&mut self, name: &str, args: Value) -> Result<Value> {
        ensure!(args.is_object(), "arguments must be an object");
        match name {
            "project_list" => Ok(serde_json::to_value(self.store.projects()?)?),
            "memory_doctor" => self.store.doctor(),
            "memory_get" => Ok(serde_json::to_value(
                self.store
                    .get(required(&args, "project")?, required(&args, "id")?)?,
            )?),
            "memory_history" => Ok(serde_json::to_value(
                self.store
                    .history(required(&args, "project")?, required(&args, "id")?)?,
            )?),
            "memory_neighbors" => self
                .store
                .neighbors(required(&args, "project")?, required(&args, "id")?),
            "memory_link" => {
                self.store.link(
                    required(&args, "project")?,
                    required(&args, "from")?,
                    required(&args, "to")?,
                    required(&args, "relation")?,
                )?;
                Ok(json!({"linked":true}))
            }
            "memory_save" => {
                let project = required(&args, "project")?;
                self.store.require_project(project)?;
                let entry: EntryInput = serde_json::from_value(
                    args.get("entry").context("entry is required")?.clone(),
                )?;
                entry.validate()?;
                // Embed before acquiring a write transaction; failed inference never partially saves.
                let vectors = if boolean(&args, "lexical_only", false)? {
                    None
                } else {
                    Some(self.embedder.document(&entry.search_text())?)
                };
                Ok(serde_json::to_value(
                    self.store.save(project, entry, vectors)?,
                )?)
            }
            "memory_search" => {
                let project = required(&args, "project")?;
                self.store.require_project(project)?;
                let query = required(&args, "query")?;
                ensure!(
                    !query.trim().is_empty() && query.chars().count() <= 2000,
                    "query must contain 1..2000 characters"
                );
                let mode = args
                    .get("mode")
                    .map(|v| v.as_str().context("mode must be a string"))
                    .transpose()?
                    .unwrap_or("hybrid");
                ensure!(
                    ["lexical", "semantic", "hybrid"].contains(&mode),
                    "invalid search mode"
                );
                let limit = args
                    .get("limit")
                    .map(|v| v.as_u64().context("limit must be a positive integer"))
                    .transpose()?
                    .unwrap_or(5);
                ensure!((1..=50).contains(&limit), "limit must be 1..50");
                let vector = if mode == "lexical" {
                    None
                } else {
                    Some(self.embedder.query(query)?)
                };
                Ok(serde_json::to_value(self.store.search(
                    project,
                    query,
                    mode,
                    limit as usize,
                    boolean(&args, "include_inactive", false)?,
                    vector.as_deref(),
                )?)?)
            }
            "memory_reindex" => {
                let project = required(&args, "project")?;
                let entries = self.store.entries(project)?;
                let count = entries.len();
                for entry in entries {
                    let vectors = self.embedder.document(&entry.data.search_text())?;
                    self.store.index_entry(&entry, vectors)?;
                }
                Ok(json!({"indexed":count,"project":project}))
            }
            "memory_export" => {
                let project = required(&args, "project")?;
                let format = args
                    .get("format")
                    .map(|v| v.as_str().context("format must be a string"))
                    .transpose()?
                    .unwrap_or("json");
                let entries = self.store.entries(project)?;
                match format {
                    "json" => {
                        let mut links = Vec::new();
                        for entry in &entries {
                            if let Some(edges) =
                                self.store.neighbors(project, &entry.id)?.as_array()
                            {
                                for edge in edges {
                                    if !links.contains(edge) {
                                        links.push(edge.clone());
                                    }
                                }
                            }
                        }
                        Ok(
                            json!({"schema_version":1,"project":project,"entries":entries,"links":links}),
                        )
                    }
                    "markdown" => {
                        let mut text = format!("# Project memory: {project}\n\n");
                        for e in entries {
                            text.push_str(&format!("## {}\n\nID: {} · revision {} · {}\n\n**Problem:** {}\n\n**Cause:** {}\n\n**Solution:** {}\n\n**Verification:** {}\n\n**Applies to:** {}\n\n**Failed attempts:** {}\n\n**Sources:** {}\n\n",e.data.title,e.id,e.revision,e.data.status,e.data.problem,e.data.cause,e.data.solution,e.data.verification,e.data.applies_to,e.data.failed_attempts.join("; "),serde_json::to_string(&e.data.sources)?));
                        }
                        Ok(json!({"markdown":text}))
                    }
                    _ => anyhow::bail!("format must be json or markdown"),
                }
            }
            _ => anyhow::bail!("unknown tool: {name}"),
        }
    }
    pub fn backup(&self, path: &Path) -> Result<()> {
        self.store.backup(path)
    }
}
pub fn required<'a>(args: &'a Value, key: &str) -> Result<&'a str> {
    args.get(key)
        .and_then(Value::as_str)
        .with_context(|| format!("{key} must be a string"))
}
fn boolean(args: &Value, key: &str, default: bool) -> Result<bool> {
    args.get(key)
        .map(|v| {
            v.as_bool()
                .with_context(|| format!("{key} must be boolean"))
        })
        .transpose()
        .map(|v| v.unwrap_or(default))
}
