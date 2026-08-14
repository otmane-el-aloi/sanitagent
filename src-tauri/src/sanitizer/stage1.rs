use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    // 1. ANSI escape codes
    static ref ANSI_REGEX: Regex = Regex::new(r"\x1b\[[0-9;?]*[a-zA-Z]").unwrap();

    // 2. High-entropy secrets and API keys
    // OpenAI API keys (sk-proj-..., sk-admin-..., sk-svc-..., sk-...)
    static ref OPENAI_KEY_REGEX: Regex = Regex::new(r"sk-(?:proj-|admin-|svc-|[a-zA-Z0-9]{20,})[a-zA-Z0-9_\-]{20,}").unwrap();

    // AWS Access Key ID
    static ref AWS_KEY_REGEX: Regex = Regex::new(r"(?:AKIA|ASIA|ABIA|ACCA)[0-9A-Z]{16}").unwrap();

    // AWS Secret Access Key
    static ref AWS_SECRET_REGEX: Regex = Regex::new(r#"(?i)(aws_secret_access_key|aws_secret|secret_key)\s*[:=]\s*['"]?([A-Za-z0-9/+=]{40})['"]?"#).unwrap();

    // GitHub Tokens
    static ref GITHUB_TOKEN_REGEX: Regex = Regex::new(r"(?:ghp|gho|ghu|ghs|ghr)_[a-zA-Z0-9]{36}|github_pat_[a-zA-Z0-9_]{20,80}").unwrap();

    // Bearer Tokens
    static ref BEARER_REGEX: Regex = Regex::new(r"(?i)Bearer\s+[a-zA-Z0-9\-_~+/]{20,}=*").unwrap();

    // Private Keys (PEM blocks)
    static ref PRIVATE_KEY_REGEX: Regex = Regex::new(r"-----BEGIN (?:RSA|DSA|EC|OPENSSH|PGP|PRIVATE)? KEY-----[\s\S]*?-----END (?:RSA|DSA|EC|OPENSSH|PGP|PRIVATE)? KEY-----").unwrap();

    // Slack Webhooks
    static ref SLACK_WEBHOOK_REGEX: Regex = Regex::new(r"https://hooks\.slack\.com/services/T[a-zA-Z0-9_]+/B[a-zA-Z0-9_]+/[a-zA-Z0-9_]+").unwrap();

    // Generic API Key / Secret assignments in JSON, YAML, ENV, or key=val
    static ref GENERIC_SECRET_REGEX: Regex = Regex::new(r#"(?i)("(?:api[_-]?key|secret|password|passwd|auth[_-]?token|private[_-]?key|access[_-]?token)"\s*:\s*)"[^"]{8,}"#).unwrap();
    static ref GENERIC_SECRET_KV_REGEX: Regex = Regex::new(r#"(?i)\b(api[_-]?key|secret|password|passwd|auth[_-]?token|access[_-]?token)=['"]?[a-zA-Z0-9_\-.~+/@]{8,}['"]?"#).unwrap();

    // Database Connection URIs
    static ref DB_URI_REGEX: Regex = Regex::new(r"(?i)(postgres|postgresql|mysql|mongodb|mongodb\+srv|redis)://([^:\s]+):([^@\s]+)@").unwrap();

    // 3. Timestamps & Memory Addresses
    // ISO Timestamps & Common log dates
    static ref ISO_TIMESTAMP_REGEX: Regex = Regex::new(r"\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:?\d{2})?").unwrap();
    static ref TIME_ONLY_REGEX: Regex = Regex::new(r"\b\d{2}:\d{2}:\d{2}(?:\.\d{3,6})?\b").unwrap();

    // Hex Memory Pointers (e.g., 0x7ffee1234567 or 0x00007f9a12b3)
    static ref HEX_PTR_REGEX: Regex = Regex::new(r"0x[0-9a-fA-F]{8,16}\b").unwrap();

    // 4. Heavy Data URIs & JWTs
    static ref BASE64_DATA_URI_REGEX: Regex = Regex::new(r"data:image/[a-zA-Z0-9\+\-]+;base64,[a-zA-Z0-9+/=]{30,}").unwrap();
    static ref JWT_REGEX: Regex = Regex::new(r"eyJ[a-zA-Z0-9_-]{10,}\.eyJ[a-zA-Z0-9_-]{10,}\.[a-zA-Z0-9_-]{10,}").unwrap();
}

/// Stage 1 Deterministic Rule-Based Cleaner (<1ms execution target)
pub fn clean_stage1(input: &str) -> String {
    if input.trim().is_empty() {
        return String::new();
    }

    // Step 1: Strip ANSI Escape sequences
    let mut text = ANSI_REGEX.replace_all(input, "").to_string();

    // Step 2: Redact High-Entropy Secrets & API Keys
    text = PRIVATE_KEY_REGEX.replace_all(&text, "[REDACTED_PRIVATE_KEY]").to_string();
    text = OPENAI_KEY_REGEX.replace_all(&text, "[REDACTED_OPENAI_KEY]").to_string();
    text = AWS_KEY_REGEX.replace_all(&text, "[REDACTED_AWS_KEY]").to_string();
    text = AWS_SECRET_REGEX.replace_all(&text, "$1: \"[REDACTED_AWS_SECRET]\"").to_string();
    text = GITHUB_TOKEN_REGEX.replace_all(&text, "[REDACTED_GITHUB_TOKEN]").to_string();
    text = BEARER_REGEX.replace_all(&text, "Bearer [REDACTED_BEARER_TOKEN]").to_string();
    text = SLACK_WEBHOOK_REGEX.replace_all(&text, "[REDACTED_SLACK_WEBHOOK]").to_string();
    text = GENERIC_SECRET_REGEX.replace_all(&text, "$1\"[REDACTED_SECRET]\"").to_string();
    text = GENERIC_SECRET_KV_REGEX.replace_all(&text, "$1=[REDACTED_SECRET]").to_string();
    text = DB_URI_REGEX.replace_all(&text, "$1://$2:[REDACTED_PASS]@").to_string();

    // Step 3: Truncate Heavy Data URIs & JWTs
    text = BASE64_DATA_URI_REGEX.replace_all(&text, "<BASE64_DATA truncated>").to_string();
    text = JWT_REGEX.replace_all(&text, "<JWT truncated>").to_string();

    // Step 4: Normalize ISO Timestamps & Hex Memory Pointers
    text = ISO_TIMESTAMP_REGEX.replace_all(&text, "<TIME>").to_string();
    text = TIME_ONLY_REGEX.replace_all(&text, "<TIME>").to_string();
    text = HEX_PTR_REGEX.replace_all(&text, "<ADDR>").to_string();

    // Step 5: Prune internal framework/stdlib stack trace frames
    let lines: Vec<&str> = text.lines().collect();
    let pruned_lines = prune_stack_frames(&lines);

    // Step 6: Collapse duplicate repetitive log lines
    collapse_duplicate_lines(&pruned_lines)
}

/// Prunes non-essential internal framework/stdlib stack frames
fn prune_stack_frames(lines: &[&str]) -> Vec<String> {
    let internal_patterns = [
        "node_modules/",
        "node:internal/",
        "java.base/",
        "site-packages/",
        "/usr/lib/",
        "/rustc/",
        "v8/src/",
        "vendor/bundle/",
        "System.Private.CoreLib",
        "internal/process/",
    ];

    let mut result = Vec::new();
    let mut skipped_count = 0;

    for line in lines {
        let trimmed = line.trim();

        // Check if line looks like a stack trace frame
        let is_stack_frame = trimmed.starts_with("at ")
            || trimmed.starts_with("by ")
            || trimmed.contains(".js:")
            || trimmed.contains(".py:")
            || trimmed.contains(".java:")
            || trimmed.contains(".rs:");

        let matches_internal = internal_patterns.iter().any(|pattern| line.contains(pattern));

        if is_stack_frame && matches_internal {
            skipped_count += 1;
        } else {
            if skipped_count > 0 {
                result.push(format!("    ... [Pruned {} framework stack frames]", skipped_count));
                skipped_count = 0;
            }
            result.push(line.to_string());
        }
    }

    if skipped_count > 0 {
        result.push(format!("    ... [Pruned {} framework stack frames]", skipped_count));
    }

    result
}

/// Collapses consecutive duplicate log lines
fn collapse_duplicate_lines(lines: &[String]) -> String {
    if lines.is_empty() {
        return String::new();
    }

    let mut collapsed = Vec::new();
    let mut current_line = &lines[0];
    let mut count = 1;

    for line in lines.iter().skip(1) {
        if line == current_line {
            count += 1;
        } else {
            if count > 1 {
                collapsed.push(format!("{} (Repeated {}x)", current_line, count));
            } else {
                collapsed.push(current_line.clone());
            }
            current_line = line;
            count = 1;
        }
    }

    if count > 1 {
        collapsed.push(format!("{} (Repeated {}x)", current_line, count));
    } else {
        collapsed.push(current_line.clone());
    }

    collapsed.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ansi_stripping() {
        let input = "\x1b[31mError:\x1b[0m Failed to connect";
        assert_eq!(clean_stage1(input), "Error: Failed to connect");
    }

    #[test]
    fn test_openai_key_redaction() {
        let input = "env OPENAI_API_KEY=sk-proj-1234567890abcdef1234567890abcdef for prod";
        let cleaned = clean_stage1(input);
        assert!(cleaned.contains("[REDACTED_OPENAI_KEY]"));
        assert!(!cleaned.contains("sk-proj"));
    }

    #[test]
    fn test_aws_key_redaction() {
        let input = "AWS Access Key: AKIAIOSFODNN7EXAMPLE";
        let cleaned = clean_stage1(input);
        assert!(cleaned.contains("[REDACTED_AWS_KEY]"));
    }

    #[test]
    fn test_github_token_redaction() {
        let input = "token: ghp_1234567890abcdef1234567890abcdef1234";
        let cleaned = clean_stage1(input);
        assert!(cleaned.contains("[REDACTED_GITHUB_TOKEN]"));
    }

    #[test]
    fn test_pointer_and_timestamp_normalization() {
        let input = "2026-08-14T16:28:46Z Segfault at 0x7ffee1234567 in main";
        let cleaned = clean_stage1(input);
        assert_eq!(cleaned, "<TIME> Segfault at <ADDR> in main");
    }

    #[test]
    fn test_repetitive_line_collapse() {
        let input = vec![
            "Error downloading resource".to_string(),
            "Error downloading resource".to_string(),
            "Error downloading resource".to_string(),
        ];
        let collapsed = collapse_duplicate_lines(&input);
        assert_eq!(collapsed, "Error downloading resource (Repeated 3x)");
    }
}
