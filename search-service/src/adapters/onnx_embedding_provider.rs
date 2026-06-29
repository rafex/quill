use std::sync::Mutex;

use ort::session::Session;
use ort::value::Tensor;
use tokenizers::Tokenizer;

use crate::ports::EmbeddingProvider;

pub const DIMENSION: usize = 384;
const MODEL_NAME: &str = "sentence-transformers/all-MiniLM-L6-v2";
const MODEL_VERSION: &str = "onnx-quantized-1";

pub struct OnnxEmbeddingProvider {
    tokenizer: Tokenizer,
    session: Mutex<Session>,
}

impl OnnxEmbeddingProvider {
    pub fn load(model_dir: &str) -> Result<Self, String> {
        let tokenizer = Tokenizer::from_file(format!("{model_dir}/tokenizer.json"))
            .map_err(|e| format!("failed to load tokenizer: {e}"))?;

        let session = Session::builder()
            .map_err(|e| e.to_string())?
            .commit_from_file(format!("{model_dir}/model.onnx"))
            .map_err(|e| e.to_string())?;

        Ok(Self {
            tokenizer,
            session: Mutex::new(session),
        })
    }
}

impl EmbeddingProvider for OnnxEmbeddingProvider {
    fn model_name(&self) -> &str {
        MODEL_NAME
    }

    fn model_version(&self) -> &str {
        MODEL_VERSION
    }

    fn dimension(&self) -> usize {
        DIMENSION
    }

    fn embed(&self, text: &str) -> Vec<f32> {
        let encoding = self
            .tokenizer
            .encode(text, true)
            .expect("tokenization failed");

        let ids: Vec<i64> = encoding.get_ids().iter().map(|&id| id as i64).collect();
        let mask: Vec<i64> = encoding
            .get_attention_mask()
            .iter()
            .map(|&m| m as i64)
            .collect();
        let type_ids: Vec<i64> = encoding
            .get_type_ids()
            .iter()
            .map(|&t| t as i64)
            .collect();
        let seq_len = ids.len();
        let shape = vec![1i64, seq_len as i64];

        let input_ids = Tensor::from_array((shape.clone(), ids)).expect("input_ids tensor");
        let attention_mask =
            Tensor::from_array((shape.clone(), mask.clone())).expect("attention_mask tensor");
        let token_type_ids =
            Tensor::from_array((shape, type_ids)).expect("token_type_ids tensor");

        let mut session = self.session.lock().unwrap();
        let outputs = session
            .run(ort::inputs![
                "input_ids" => input_ids,
                "attention_mask" => attention_mask,
                "token_type_ids" => token_type_ids,
            ])
            .expect("onnx inference failed");

        let (shape, last_hidden_state) = outputs[0]
            .try_extract_tensor::<f32>()
            .expect("unexpected onnx output shape");
        let hidden_size = shape[2] as usize;

        mean_pool(last_hidden_state, &mask, seq_len, hidden_size)
    }
}

fn mean_pool(hidden_states: &[f32], attention_mask: &[i64], seq_len: usize, hidden_size: usize) -> Vec<f32> {
    let mut pooled = vec![0f32; hidden_size];
    let mut valid_tokens = 0f32;

    for token_index in 0..seq_len {
        if attention_mask[token_index] == 0 {
            continue;
        }
        valid_tokens += 1.0;
        let offset = token_index * hidden_size;
        for dim in 0..hidden_size {
            pooled[dim] += hidden_states[offset + dim];
        }
    }

    if valid_tokens > 0.0 {
        for value in pooled.iter_mut() {
            *value /= valid_tokens;
        }
    }

    let norm: f32 = pooled.iter().map(|v| v * v).sum::<f32>().sqrt();
    if norm > 0.0 {
        for value in pooled.iter_mut() {
            *value /= norm;
        }
    }

    pooled
}
