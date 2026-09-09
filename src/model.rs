//! Public data contracts and validation, independent of transport and storage.
use anyhow::{Result, bail, ensure};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub reference: String,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntryInput {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub expected_revision: Option<i64>,
    pub title: String,
    #[serde(default = "lesson")]
    pub kind: String,
    pub problem: String,
    #[serde(default)]
    pub cause: String,
    #[serde(default)]
    pub solution: String,
    #[serde(default)]
    pub verification: String,
    #[serde(default = "candidate")]
    pub status: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub sources: Vec<Source>,
    #[serde(default)]
    pub applies_to: String,
    #[serde(default)]
    pub failed_attempts: Vec<String>,
}
fn lesson() -> String {
    "lesson".into()
}
fn candidate() -> String {
    "candidate".into()
}

impl EntryInput {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            !self.title.trim().is_empty() && self.title.chars().count() <= 240,
            "title must contain 1..240 characters"
        );
        ensure!(!self.problem.trim().is_empty(), "problem is required");
        ensure!(
            ["lesson", "decision", "constraint", "pattern"].contains(&self.kind.as_str()),
            "invalid kind"
        );
        ensure!(
            ["candidate", "verified", "stale", "superseded"].contains(&self.status.as_str()),
            "invalid status"
        );
        ensure!(
            self.id.is_some() == self.expected_revision.is_some(),
            "updates require both id and expected_revision"
        );
        ensure!(
            self.expected_revision.is_none_or(|r| r > 0),
            "revision must be positive"
        );
        ensure!(
            self.sources.iter().all(|s| !s.reference.trim().is_empty()),
            "source references cannot be empty"
        );
        if self.status == "verified" {
            ensure!(
                !self.verification.trim().is_empty()
                    && !self.sources.is_empty()
                    && !self.solution.trim().is_empty(),
                "verified entries require solution, verification and sources"
            );
        }
        ensure!(
            serde_json::to_vec(self)?.len() <= 64_000,
            "entry exceeds 64 KB; split into focused lessons"
        );
        Ok(())
    }
    /// Stable search text excludes volatile timestamps and record identity.
    pub fn search_text(&self) -> String {
        format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
            self.title,
            self.problem,
            self.cause,
            self.solution,
            self.applies_to,
            self.tags.join(" "),
            self.failed_attempts.join("\n"),
            self.verification
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: String,
    pub project: String,
    pub revision: i64,
    pub created_at: String,
    pub updated_at: String,
    pub indexed: bool,
    pub data: EntryInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub roots: Vec<String>,
}

pub fn validate_project(id: &str) -> Result<()> {
    if id.is_empty()
        || id.len() > 80
        || !id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))
    {
        bail!("project id must be 1..80 ASCII letters, digits, -, _ or .");
    }
    Ok(())
}
