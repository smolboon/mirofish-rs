//! Text chunking for graph building
//!
//! Splits large texts into overlapping chunks suitable for knowledge graph construction.

/// Split text into overlapping chunks of approximately the given size.
///
/// This function respects paragraph and sentence boundaries when possible,
/// falling back to character-level splitting if needed.
pub fn split_text(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    let mut start = 0;
    let chars: Vec<char> = text.chars().collect();
    let total_len = chars.len();

    while start < total_len {
        let mut end = std::cmp::min(start + chunk_size, total_len);

        // Try to find a good break point (paragraph boundary)
        if end < total_len {
            if let Some(para_end) = find_paragraph_boundary(&chars[start..end]) {
                end = start + para_end;
            } else if let Some(sent_end) = find_sentence_boundary(&chars[start..end]) {
                end = start + sent_end;
            }
        }

        let chunk: String = chars[start..end].iter().collect();
        if !chunk.trim().is_empty() {
            chunks.push(chunk.trim().to_string());
        }

        // Move start with overlap
        start = if end >= overlap { end - overlap } else { end };

        // Prevent infinite loop
        if start >= end {
            start = end;
        }
    }

    chunks
}

/// Find the last paragraph boundary (double newline) within the text
fn find_paragraph_boundary(chars: &[char]) -> Option<usize> {
    for i in (chars.len().saturating_sub(100)..chars.len()).rev() {
        if i + 1 < chars.len() && chars[i] == '\n' && chars[i + 1] == '\n' {
            return Some(i + 2);
        }
    }
    None
}

/// Find the last sentence boundary (.!? followed by space) within the text
fn find_sentence_boundary(chars: &[char]) -> Option<usize> {
    for i in (chars.len().saturating_sub(50)..chars.len()).rev() {
        if i + 1 < chars.len()
            && (chars[i] == '.' || chars[i] == '!' || chars[i] == '?')
            && (chars.get(i + 1) == Some(&' ') || chars.get(i + 1) == Some(&'\n'))
        {
            return Some(i + 2);
        }
    }
    None
}

/// Preprocess text: normalize whitespace, remove control characters
pub fn preprocess_text(text: &str) -> String {
    text
        .replace("\r\n", "\n")
        .replace("\r", "\n")
        .split('\n')
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
        .chars()
        .filter(|c| c.is_whitespace() || !c.is_control())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_text_basic() {
        let text = "This is sentence one. This is sentence two. This is sentence three.";
        let chunks = split_text(text, 30, 5);
        assert!(!chunks.is_empty());
        assert!(chunks.len() >= 1);
    }

    #[test]
    fn test_split_text_empty() {
        let chunks = split_text("", 100, 10);
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_split_text_small() {
        let text = "Hello world";
        let chunks = split_text(text, 100, 10);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "Hello world");
    }

    #[test]
    fn test_split_text_with_paragraphs() {
        let text = "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.";
        let chunks = split_text(text, 20, 5);
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_preprocess_text() {
        let text = "  Hello  \r\n  World  \r\n  \r\n  Test  ";
        let processed = preprocess_text(text);
        assert!(processed.contains("Hello"));
        assert!(processed.contains("World"));
        assert!(processed.contains("Test"));
    }
}