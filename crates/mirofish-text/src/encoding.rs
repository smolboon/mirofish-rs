//! Character encoding detection and decoding

use encoding_rs::{Encoding, UTF_8, WINDOWS_1252, SHIFT_JIS, EUC_JP, GBK, BIG5, EUC_KR, UTF_16LE, UTF_16BE, GB18030};
use mirofish_core::FileError;

/// Detect encoding and decode bytes to string
pub fn decode_with_fallback(bytes: &[u8]) -> anyhow::Result<String> {
    // Try UTF-8 first
    if let Ok(s) = std::str::from_utf8(bytes) {
        return Ok(s.to_string());
    }

    // Use chardetng to detect encoding
    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(bytes, false);
    let hint = detector.guess(None, true);

    // Map chardetng encoding name to encoding_rs Encoding
    let encoding = encoding_name_to_encoding_rs(hint.name());

    // Decode using encoding_rs
    let (cow, _enc, _had_errors) = encoding.decode(bytes);
    Ok(cow.into_owned())
}

/// Read a text file with automatic encoding detection
pub fn read_text_file(path: &std::path::Path) -> anyhow::Result<String> {
    let bytes = std::fs::read(path)
        .map_err(|e| FileError::ReadError(format!("Failed to read file: {}", e)))?;

    decode_with_fallback(&bytes)
}

/// Detect encoding name from bytes
pub fn detect_encoding(bytes: &[u8]) -> &'static str {
    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(bytes, false);
    detector.guess(None, true).name()
}

/// Map encoding name string to encoding_rs Encoding
fn encoding_name_to_encoding_rs(name: &str) -> &'static Encoding {
    let name = name.to_ascii_lowercase();
    match name.as_str() {
        "utf-8" | "utf8" => UTF_8,
        "windows-1252" | "cp1252" => WINDOWS_1252,
        "iso-8859-1" | "latin1" => WINDOWS_1252, // Closest match in encoding_rs
        "shift_jis" | "sjis" => SHIFT_JIS,
        "euc-jp" => EUC_JP,
        "gbk" | "gb2312" => GBK,
        "big5" => BIG5,
        "euc-kr" => EUC_KR,
        "utf-16le" => UTF_16LE,
        "utf-16be" => UTF_16BE,
        "gb18030" => GB18030,
        _ => UTF_8, // Safe fallback - use UTF-8 replacement
    }
}
