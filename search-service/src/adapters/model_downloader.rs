use std::fs;
use std::io::{Read, Write};
use std::path::Path;

const TOKENIZER_URL: &str =
    "https://huggingface.co/Xenova/all-MiniLM-L6-v2/resolve/main/tokenizer.json";
const MODEL_URL: &str =
    "https://huggingface.co/Xenova/all-MiniLM-L6-v2/resolve/main/onnx/model_quantized.onnx";

pub fn download_model(model_dir: &str) -> Result<(), String> {
    fs::create_dir_all(model_dir).map_err(|e| e.to_string())?;

    download_file(TOKENIZER_URL, &format!("{model_dir}/tokenizer.json"))?;
    download_file(MODEL_URL, &format!("{model_dir}/model.onnx"))?;

    Ok(())
}

fn download_file(url: &str, dest: &str) -> Result<(), String> {
    if Path::new(dest).exists() {
        tracing::info!(dest, "skipping download (file already exists)");
        return Ok(());
    }

    tracing::info!(url, dest, "downloading model file");
    let response = ureq::get(url)
        .call()
        .map_err(|e| format!("request to {url} failed: {e}"))?;

    let mut bytes = Vec::new();
    response
        .into_reader()
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;

    let mut file = fs::File::create(dest).map_err(|e| e.to_string())?;
    file.write_all(&bytes).map_err(|e| e.to_string())?;

    tracing::info!(dest, bytes = bytes.len(), "model file saved");
    Ok(())
}
