use serde::{de::DeserializeOwned, Serialize};
use std::io::Read;

const ZSTD_LEVEL: i32 = 3;

/// Serialize JSON và nén bằng Zstd.
pub fn compress_zstd_json<T>(value: &T) -> Result<Vec<u8>, String>
where
    T: Serialize,
{
    let json = serde_json::to_vec(value).map_err(|e| format!("JSON serialize error: {e}"))?;

    zstd::encode_all(&json[..], ZSTD_LEVEL).map_err(|e| format!("Zstd compress JSON error: {e}"))
}

/// Giải nén Zstd và deserialize JSON.
pub fn decompress_zstd_json<T>(bytes: &[u8]) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let mut decoder = zstd::Decoder::new(bytes).map_err(|e| format!("Zstd decoder error: {e}"))?;

    let mut json = Vec::new();

    decoder
        .read_to_end(&mut json)
        .map_err(|e| format!("Zstd decompress JSON error: {e}"))?;

    serde_json::from_slice(&json).map_err(|e| format!("JSON deserialize error: {e}"))
}

/// Nén chuỗi XML bằng Zstd.
pub fn compress_zstd_xml(xml: &str) -> Result<Vec<u8>, String> {
    zstd::encode_all(xml.as_bytes(), ZSTD_LEVEL)
        .map_err(|e| format!("Zstd compress XML error: {e}"))
}

/// Giải nén Zstd thành chuỗi XML.
pub fn decompress_zstd_xml(bytes: &[u8]) -> Result<String, String> {
    let mut decoder = zstd::Decoder::new(bytes).map_err(|e| format!("Zstd decoder error: {e}"))?;

    let mut xml = String::new();

    decoder
        .read_to_string(&mut xml)
        .map_err(|e| format!("Zstd decompress XML error: {e}"))?;

    Ok(xml)
}

pub fn decompress_zstd_json_text(bytes: &[u8]) -> Result<String, String> {
    zstd::decode_all(bytes)
        .map_err(|e| e.to_string())
        .and_then(|data| String::from_utf8(data).map_err(|e| e.to_string()))
}
