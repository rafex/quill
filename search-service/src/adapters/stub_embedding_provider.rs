use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::infrastructure::EMBEDDING_DIMENSION as DIMENSION;
use crate::ports::EmbeddingProvider;

pub struct StubEmbeddingProvider;

impl EmbeddingProvider for StubEmbeddingProvider {
    fn model_name(&self) -> &str {
        "stub-hash-embedding"
    }

    fn model_version(&self) -> &str {
        "v1"
    }

    fn dimension(&self) -> usize {
        DIMENSION
    }

    fn embed(&self, text: &str) -> Vec<f32> {
        let mut vector = vec![0f32; DIMENSION];

        for word in text.split_whitespace() {
            let mut hasher = DefaultHasher::new();
            word.to_lowercase().hash(&mut hasher);
            let index = (hasher.finish() as usize) % DIMENSION;
            vector[index] += 1.0;
        }

        normalize(&mut vector);
        vector
    }
}

fn normalize(vector: &mut [f32]) {
    let norm: f32 = vector.iter().map(|v| v * v).sum::<f32>().sqrt();
    if norm > 0.0 {
        for v in vector.iter_mut() {
            *v /= norm;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedding_has_expected_dimension() {
        let provider = StubEmbeddingProvider;
        assert_eq!(provider.embed("hello world").len(), DIMENSION);
    }

    #[test]
    fn same_text_produces_same_embedding() {
        let provider = StubEmbeddingProvider;
        assert_eq!(
            provider.embed("rust and sqlite"),
            provider.embed("rust and sqlite")
        );
    }

    #[test]
    fn different_text_produces_different_embedding() {
        let provider = StubEmbeddingProvider;
        assert_ne!(
            provider.embed("rust and sqlite"),
            provider.embed("python and postgres")
        );
    }

    #[test]
    fn non_empty_text_produces_unit_vector() {
        let provider = StubEmbeddingProvider;
        let vector = provider.embed("hello world");
        let norm: f32 = vector.iter().map(|v| v * v).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-5);
    }
}
