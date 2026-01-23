//! Embedding model implementation for the OpenAI provider.

use crate::{
    core::{
        capabilities::ModelName,
        embedding_model::{EmbeddingModel, EmbeddingModelResponse},
    },
    providers::openai::OpenAI,
};

#[derive(Debug, Clone)]
/// Settings for OpenAI that are specific to embedding models.
pub struct OpenAIEmbeddingModelOptions {}

impl<M: ModelName> EmbeddingModel for OpenAI<M> {
    fn embed(&self) -> EmbeddingModelResponse {
        todo!()
    }
}
