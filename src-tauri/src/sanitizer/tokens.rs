use serde::{Deserialize, Serialize};
use tiktoken_rs::cl100k_base;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenStats {
    pub raw_tokens: usize,
    pub sanitized_tokens: usize,
    pub saved_tokens: usize,
    pub reduction_percent: f64,
}

pub fn calculate_stats(raw_text: &str, sanitized_text: &str) -> TokenStats {
    let raw_tokens = count_tokens(raw_text);
    let sanitized_tokens = count_tokens(sanitized_text);

    let saved_tokens = if raw_tokens > sanitized_tokens {
        raw_tokens - sanitized_tokens
    } else {
        0
    };

    let reduction_percent = if raw_tokens > 0 {
        ((saved_tokens as f64) / (raw_tokens as f64)) * 100.0
    } else {
        0.0
    };

    TokenStats {
        raw_tokens,
        sanitized_tokens,
        saved_tokens,
        reduction_percent: (reduction_percent * 10.0).round() / 10.0, // 1 decimal place
    }
}

pub fn count_tokens(text: &str) -> usize {
    if text.trim().is_empty() {
        return 0;
    }

    match cl100k_base() {
        Ok(bpe) => bpe.encode_with_special_tokens(text).len(),
        Err(_) => {
            // Fallback word/subword heuristic
            let words = text.split_whitespace().count();
            (words as f64 * 1.3) as usize
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_reduction_calculation() {
        let raw = "2026-08-14T16:28:46Z Error segfault at 0x7ffee123 with key sk-proj-1234567890abcdef1234567890abcdef";
        let sanitized = "<TIME> Error segfault at <ADDR> with key [REDACTED_OPENAI_KEY]";

        let stats = calculate_stats(raw, sanitized);
        assert!(stats.raw_tokens > stats.sanitized_tokens);
        assert!(stats.saved_tokens > 0);
        assert!(stats.reduction_percent > 0.0);
    }
}
