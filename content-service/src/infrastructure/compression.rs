pub const ALGORITHM: &str = "zstd";
pub const LEVEL: i32 = 3;

pub struct CompressedBody {
    pub data: Vec<u8>,
    pub original_length: usize,
}

pub fn compress(body: &str) -> CompressedBody {
    let data = zstd::encode_all(body.as_bytes(), LEVEL).expect("zstd compression failed");
    CompressedBody {
        data,
        original_length: body.len(),
    }
}

pub fn decompress(data: &[u8]) -> String {
    let bytes = zstd::decode_all(data).expect("zstd decompression failed");
    String::from_utf8(bytes).expect("decompressed body is not valid utf-8")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_preserves_original_body() {
        let body = "hello forum body ".repeat(50);
        let compressed = compress(&body);

        assert_eq!(compressed.original_length, body.len());
        assert!(compressed.data.len() < body.len());
        assert_eq!(decompress(&compressed.data), body);
    }

    #[test]
    fn round_trip_handles_empty_body() {
        let compressed = compress("");
        assert_eq!(compressed.original_length, 0);
        assert_eq!(decompress(&compressed.data), "");
    }
}
