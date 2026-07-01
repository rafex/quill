use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn validate_slug(slug: &str) -> bool {
    !slug.is_empty()
        && slug.len() <= 120
        && slug
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !slug.starts_with('-')
        && !slug.ends_with('-')
}

#[wasm_bindgen]
pub fn generate_slug(title: &str) -> String {
    let mut slug = String::with_capacity(title.len());
    let mut prev_dash = false;

    for c in title.chars() {
        if c.is_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash && !slug.is_empty() {
            slug.push('-');
            prev_dash = true;
        }
    }

    let slug = slug.trim_end_matches('-').to_string();
    if slug.len() > 120 { slug[..120].to_string() } else { slug }
}

#[wasm_bindgen]
pub fn validate_email(email: &str) -> bool {
    let parts: Vec<&str> = email.splitn(2, '@').collect();
    if parts.len() != 2 { return false; }
    let local = parts[0];
    let domain = parts[1];
    !local.is_empty() && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.')
}

#[wasm_bindgen]
pub fn compute_hybrid_score(vector_score: f32, bm25_score: f32) -> f32 {
    0.60 * vector_score + 0.40 * bm25_score
}

#[wasm_bindgen]
pub fn truncate_body(body: &str, max_chars: usize) -> String {
    if body.chars().count() <= max_chars {
        return body.to_string();
    }
    let truncated: String = body.chars().take(max_chars).collect();
    format!("{}…", truncated.trim_end())
}
