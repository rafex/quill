mod stub_embedding_provider;

pub use stub_embedding_provider::StubEmbeddingProvider;

#[cfg(feature = "onnx-embeddings")]
mod model_downloader;
#[cfg(feature = "onnx-embeddings")]
mod onnx_embedding_provider;

#[cfg(feature = "onnx-embeddings")]
pub use model_downloader::download_model;
#[cfg(feature = "onnx-embeddings")]
pub use onnx_embedding_provider::OnnxEmbeddingProvider;
