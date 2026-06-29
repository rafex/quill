pub trait EmbeddingProvider: Send + Sync {
    fn model_name(&self) -> &str;
    fn model_version(&self) -> &str;
    fn dimension(&self) -> usize;
    fn embed(&self, text: &str) -> Vec<f32>;
}
