use std::path::Path;

pub fn parse_document(file_path: &str) -> Result<String, String> {
    let path = Path::new(file_path);
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "pdf" => parse_pdf(file_path),
        "md" | "markdown" | "txt" | "text" => parse_text(file_path),
        _ => Err(format!("Unsupported file type: {}", extension)),
    }
}

fn parse_text(file_path: &str) -> Result<String, String> {
    std::fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read text file {}: {}", file_path, e))
}

fn parse_pdf(file_path: &str) -> Result<String, String> {
    let bytes = std::fs::read(file_path)
        .map_err(|e| format!("Failed to read PDF file {}: {}", file_path, e))?;

    let doc = lopdf::Document::load_mem(&bytes)
        .map_err(|e| format!("Failed to parse PDF {}: {}", file_path, e))?;

    let mut text = String::new();
    let pages = doc.get_pages();

    for (page_num, _) in pages.iter() {
        if let Ok(page_text) = doc.extract_text(&[*page_num]) {
            text.push_str(&page_text);
            text.push('\n');
        }
    }

    if text.trim().is_empty() {
        return Err("PDF appears to contain no extractable text (may be scanned/image-based)".to_string());
    }

    Ok(text)
}

pub fn chunk_text(text: &str, max_chunk_size: usize, overlap: usize) -> Vec<String> {
    let splitter = text_splitter::TextSplitter::new(max_chunk_size);
    let chunks: Vec<String> = splitter.chunks(text).map(|c| c.to_string()).collect();

    if overlap > 0 && chunks.len() > 1 {
        let mut overlapped = Vec::new();
        for (i, chunk) in chunks.iter().enumerate() {
            let mut ch = chunk.clone();
            if i > 0 {
                let prev = &chunks[i - 1];
                let prev_words: Vec<&str> = prev.split_whitespace().collect();
                let overlap_words = if prev_words.len() > overlap {
                    &prev_words[prev_words.len() - overlap..]
                } else {
                    &prev_words
                };
                ch = format!("{} {}", overlap_words.join(" "), ch);
            }
            overlapped.push(ch);
        }
        overlapped
    } else {
        chunks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_text_no_overlap() {
        let text = "This is a test document with multiple words that should be split into chunks.";
        let chunks = chunk_text(text, 5, 0);
        assert!(chunks.len() > 1, "Should produce multiple chunks with small max size");
        for chunk in &chunks {
            assert!(!chunk.is_empty(), "Chunks should not be empty");
        }
    }

    #[test]
    fn test_chunk_text_single_chunk() {
        let text = "Short text.";
        let chunks = chunk_text(text, 100, 0);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], text);
    }

    #[test]
    fn test_chunk_text_empty() {
        let chunks = chunk_text("", 100, 0);
        assert_eq!(chunks.len(), 0);
    }

    #[test]
    fn test_chunk_text_with_overlap() {
        let text = "word1 word2 word3 word4 word5 word6 word7 word8 word9 word10";
        let chunks = chunk_text(text, 3, 2);
        assert!(chunks.len() > 1, "Should produce multiple chunks");
        // With overlap, each chunk should have some shared words
        if chunks.len() >= 2 {
            let first_end = chunks[0].split_whitespace().last().unwrap_or("");
            let _second_start = chunks[1].split_whitespace().next().unwrap_or("");
            // With overlap=2, the last words of chunk 0 should appear at start of chunk 1
            assert!(
                chunks[1].contains(first_end),
                "Overlap should cause shared words between chunks"
            );
        }
    }

    #[test]
    fn test_parse_document_unsupported_extension() {
        let result = parse_document("test.xyz");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unsupported file type"));
    }

    #[test]
    fn test_parse_document_text_file() {
        let result = parse_document("Cargo.toml");
        // .toml is unsupported, so we test that it errors correctly
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_document_nonexistent_file() {
        let result = parse_document("nonexistent_file.md");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_document_md_extension() {
        let result = parse_document("nonexistent.md");
        // Should attempt to read and fail because file doesn't exist
        assert!(result.is_err());
    }

    #[test]
    fn test_chunk_text_overlap_larger_than_chunk() {
        let text = "one two three four";
        let chunks = chunk_text(text, 2, 10);
        assert!(!chunks.is_empty());
        // Overlap larger than chunk size should still work
        for chunk in &chunks {
            assert!(!chunk.is_empty());
        }
    }
}
