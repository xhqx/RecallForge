//! Lazy, local E5 inference. Model identifiers are persisted to prevent mixed vector spaces.
use anyhow::{Result, ensure};
use fastembed::{
    InitOptionsUserDefined, Pooling, TextEmbedding, TokenizerFiles, UserDefinedEmbeddingModel,
};
use std::path::PathBuf;

pub const DIMENSIONS: usize = 384;
pub const MODEL_REVISION: &str = "614241f622f53c4eeff9890bdc4f31cfecc418b3";
pub const MODEL: &str = "intfloat/multilingual-e5-small@614241f622f53c4eeff9890bdc4f31cfecc418b3:fastembed-6.0.3:passage-query:chars800-overlap200:v1";

pub struct Embedder {
    cache: PathBuf,
    model: Option<TextEmbedding>,
}
impl Embedder {
    pub fn new(cache: PathBuf) -> Self {
        let cache = std::env::var_os("RECALLFORGE_MODEL_CACHE")
            .map(PathBuf::from)
            .unwrap_or(cache);
        Self { cache, model: None }
    }
    fn encode(&mut self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        if self.model.is_none() {
            eprintln!(
                "RecallForge: loading local multilingual E5 (first use downloads model weights)"
            );
            let repo = hf_hub::api::sync::ApiBuilder::new()
                .with_cache_dir(self.cache.clone())
                .with_progress(false)
                .build()?
                .repo(hf_hub::Repo::with_revision(
                    "intfloat/multilingual-e5-small".into(),
                    hf_hub::RepoType::Model,
                    MODEL_REVISION.into(),
                ));
            let read = |name: &str| -> Result<Vec<u8>> { Ok(std::fs::read(repo.get(name)?)?) };
            let tokenizer = TokenizerFiles {
                tokenizer_file: read("tokenizer.json")?,
                config_file: read("config.json")?,
                special_tokens_map_file: read("special_tokens_map.json")?,
                tokenizer_config_file: read("tokenizer_config.json")?,
            };
            let model = UserDefinedEmbeddingModel::new(read("onnx/model.onnx")?, tokenizer)
                .with_pooling(Pooling::Mean);
            self.model = Some(TextEmbedding::try_new_from_user_defined(
                model,
                InitOptionsUserDefined::new(),
            )?);
        }
        let vectors = self.model.as_mut().unwrap().embed(texts, Some(16))?;
        for v in &vectors {
            validate_vector(v)?;
        }
        Ok(vectors)
    }
    pub fn query(&mut self, text: &str) -> Result<Vec<f32>> {
        Ok(self.encode(vec![format!("query: {text}")])?.remove(0))
    }
    /// Chunk long notes so the model's token limit does not silently discard the solution.
    pub fn document(&mut self, text: &str) -> Result<Vec<Vec<f32>>> {
        self.encode(
            chunks(text)
                .into_iter()
                .map(|s| format!("passage: {s}"))
                .collect(),
        )
    }
}
pub fn validate_vector(vector: &[f32]) -> Result<()> {
    ensure!(
        vector.len() == DIMENSIONS && vector.iter().all(|v| v.is_finite()),
        "invalid embedding dimensions or non-finite values"
    );
    ensure!(
        vector.iter().any(|v| *v != 0.0),
        "zero embedding is invalid"
    );
    Ok(())
}
pub fn chunks(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return vec![String::new()];
    }
    // Conservatively small Unicode chunks, including overlap for boundary context.
    (0..chars.len())
        .step_by(600)
        .map(|start| {
            chars[start..(start + 800).min(chars.len())]
                .iter()
                .collect()
        })
        .collect()
}
