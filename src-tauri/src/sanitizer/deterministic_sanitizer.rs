//! Multi-stage prompt/log sanitizer.
//!
//! Stage 1: deterministic secret redaction + log noise cleanup (regex + aho-corasick + entropy fallback)
//! Stage 2: PII detection/redaction (validated with dedicated crates, not just regex shape-matching)
//! Stage 3: profanity filtering (rustrict)

use aho_corasick::AhoCorasick;
use regex::Regex;
use std::sync::LazyLock;

// =====================================================================================
// STAGE 1 — Secrets & log-noise cleaning
// =====================================================================================

// ---- existing patterns, kept as-is -------------------------------------------------
static ANSI_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\x1b\[[0-9;?]*[a-zA-Z]").unwrap());

static OPENAI_KEY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"sk-(?:proj-|admin-|svc-|[a-zA-Z0-9]{20,})[a-zA-Z0-9_\-]{20,}").unwrap()
});

static AWS_KEY_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:AKIA|ASIA|ABIA|ACCA)[0-9A-Z]{16}").unwrap());

static AWS_SECRET_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)(aws_secret_access_key|aws_secret|secret_key)\s*[:=]\s*['"]?([A-Za-z0-9/+=]{40})['"]?"#).unwrap()
});

static GITHUB_TOKEN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:ghp|gho|ghu|ghs|ghr)_[a-zA-Z0-9]{36}|github_pat_[a-zA-Z0-9_]{20,80}").unwrap()
});

static BEARER_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)Bearer\s+[a-zA-Z0-9\-_~+/]{20,}=*").unwrap());

static PRIVATE_KEY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"-----BEGIN (?:RSA|DSA|EC|OPENSSH|PGP|PRIVATE)? KEY-----[\s\S]*?-----END (?:RSA|DSA|EC|OPENSSH|PGP|PRIVATE)? KEY-----").unwrap()
});

static SLACK_WEBHOOK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https://hooks\.slack\.com/services/T[a-zA-Z0-9_]+/B[a-zA-Z0-9_]+/[a-zA-Z0-9_]+").unwrap()
});

static GENERIC_SECRET_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)("(?:api[_-]?key|secret|password|passwd|auth[_-]?token|private[_-]?key|access[_-]?token)"\s*:\s*)"[^"]{8,}"#).unwrap()
});

static GENERIC_SECRET_KV_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)\b(api[_-]?key|secret|password|passwd|auth[_-]?token|access[_-]?token)=['"]?[a-zA-Z0-9_\-.~+/@]{8,}['"]?"#).unwrap()
});

static DB_URI_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(postgres|postgresql|mysql|mongodb|mongodb\+srv|redis)://([^:\s]+):([^@\s]+)@").unwrap()
});

static ISO_TIMESTAMP_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:?\d{2})?").unwrap()
});

static TIME_ONLY_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b\d{2}:\d{2}:\d{2}(?:\.\d{3,6})?\b").unwrap());

static HEX_PTR_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"0x[0-9a-fA-F]{8,16}\b").unwrap());

static BASE64_DATA_URI_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"data:image/[a-zA-Z0-9\+\-]+;base64,[a-zA-Z0-9+/=]{30,}").unwrap()
});

static JWT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"eyJ[a-zA-Z0-9_-]{10,}\.eyJ[a-zA-Z0-9_-]{10,}\.[a-zA-Z0-9_-]{10,}").unwrap()
});

// ---- new patterns: broaden vendor coverage without hand-maintaining every regex ----
static SLACK_TOKEN_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"xox[baprs]-[0-9a-zA-Z\-]{10,72}").unwrap());

static STRIPE_KEY_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:sk|rk)_(?:live|test)_[0-9a-zA-Z]{20,247}").unwrap());

static TWILIO_KEY_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"SK[0-9a-fA-F]{32}").unwrap());

static SENDGRID_KEY_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"SG\.[a-zA-Z0-9_\-]{22}\.[a-zA-Z0-9_\-]{43}").unwrap());

static GOOGLE_API_KEY_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"AIza[0-9A-Za-z_\-]{35}").unwrap());

static NPM_TOKEN_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"npm_[a-zA-Z0-9]{36}").unwrap());

static AZURE_CONN_STRING_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)DefaultEndpointsProtocol=https?;AccountName=[^;]+;AccountKey=[a-zA-Z0-9+/=]{20,}").unwrap()
});

// Internal framework/stdlib stack-frame path fragments, matched with Aho-Corasick
static INTERNAL_FRAME_PATTERNS: &[&str] = &[
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

static INTERNAL_FRAME_MATCHER: LazyLock<AhoCorasick> =
    LazyLock::new(|| AhoCorasick::new(INTERNAL_FRAME_PATTERNS).unwrap());

/// Catches high-entropy tokens that don't match any known vendor prefix
fn shannon_entropy(s: &str) -> f64 {
    use std::collections::HashMap;
    let mut counts: HashMap<char, usize> = HashMap::new();
    for c in s.chars() {
        *counts.entry(c).or_insert(0) += 1;
    }
    let len = s.chars().count() as f64;
    counts
        .values()
        .map(|&c| {
            let p = c as f64 / len;
            -p * p.log2()
        })
        .sum()
}

static CANDIDATE_TOKEN_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[A-Za-z0-9+/=_\-]{24,}").unwrap());

const ENTROPY_THRESHOLD: f64 = 4.0;

fn redact_high_entropy_fallback(text: &str) -> String {
    CANDIDATE_TOKEN_REGEX
        .replace_all(text, |caps: &regex::Captures| {
            let token = &caps[0];
            if shannon_entropy(token) >= ENTROPY_THRESHOLD {
                "[REDACTED_HIGH_ENTROPY]".to_string()
            } else {
                token.to_string()
            }
        })
        .to_string()
}

/// Stage 1 — Deterministic rule-based cleaner (<1ms execution target).
pub fn clean_stage1(input: &str) -> String {
    if input.trim().is_empty() {
        return String::new();
    }

    let mut text = ANSI_REGEX.replace_all(input, "").to_string();

    // Known-vendor secrets
    text = PRIVATE_KEY_REGEX.replace_all(&text, "[REDACTED_PRIVATE_KEY]").to_string();
    text = OPENAI_KEY_REGEX.replace_all(&text, "[REDACTED_OPENAI_KEY]").to_string();
    text = AWS_KEY_REGEX.replace_all(&text, "[REDACTED_AWS_KEY]").to_string();
    text = AWS_SECRET_REGEX.replace_all(&text, "$1: \"[REDACTED_AWS_SECRET]\"").to_string();
    text = GITHUB_TOKEN_REGEX.replace_all(&text, "[REDACTED_GITHUB_TOKEN]").to_string();
    text = BEARER_REGEX.replace_all(&text, "Bearer [REDACTED_BEARER_TOKEN]").to_string();
    text = SLACK_WEBHOOK_REGEX.replace_all(&text, "[REDACTED_SLACK_WEBHOOK]").to_string();
    text = SLACK_TOKEN_REGEX.replace_all(&text, "[REDACTED_SLACK_TOKEN]").to_string();
    text = STRIPE_KEY_REGEX.replace_all(&text, "[REDACTED_STRIPE_KEY]").to_string();
    text = TWILIO_KEY_REGEX.replace_all(&text, "[REDACTED_TWILIO_KEY]").to_string();
    text = SENDGRID_KEY_REGEX.replace_all(&text, "[REDACTED_SENDGRID_KEY]").to_string();
    text = GOOGLE_API_KEY_REGEX.replace_all(&text, "[REDACTED_GOOGLE_API_KEY]").to_string();
    text = NPM_TOKEN_REGEX.replace_all(&text, "[REDACTED_NPM_TOKEN]").to_string();
    text = AZURE_CONN_STRING_REGEX.replace_all(&text, "[REDACTED_AZURE_CONN_STRING]").to_string();
    text = GENERIC_SECRET_REGEX.replace_all(&text, "$1\"[REDACTED_SECRET]\"").to_string();
    text = GENERIC_SECRET_KV_REGEX.replace_all(&text, "$1=[REDACTED_SECRET]").to_string();
    text = DB_URI_REGEX.replace_all(&text, "$1://$2:[REDACTED_PASS]@").to_string();

    // Heavy/opaque blobs
    text = BASE64_DATA_URI_REGEX.replace_all(&text, "<BASE64_DATA truncated>").to_string();
    text = JWT_REGEX.replace_all(&text, "<JWT truncated>").to_string();

    // Fallback pass for secrets that don't match a known vendor shape.
    text = redact_high_entropy_fallback(&text);

    // Normalize timestamps & memory addresses
    text = ISO_TIMESTAMP_REGEX.replace_all(&text, "<TIME>").to_string();
    text = TIME_ONLY_REGEX.replace_all(&text, "<TIME>").to_string();
    text = HEX_PTR_REGEX.replace_all(&text, "<ADDR>").to_string();

    let lines: Vec<&str> = text.lines().collect();
    let pruned_lines = prune_stack_frames(&lines);
    collapse_duplicate_lines(&pruned_lines)
}

fn prune_stack_frames(lines: &[&str]) -> Vec<String> {
    let mut result = Vec::new();
    let mut skipped_count = 0;

    for line in lines {
        let trimmed = line.trim();

        let is_stack_frame = trimmed.starts_with("at ")
            || trimmed.starts_with("by ")
            || trimmed.contains(".js:")
            || trimmed.contains(".py:")
            || trimmed.contains(".java:")
            || trimmed.contains(".rs:");

        let matches_internal = INTERNAL_FRAME_MATCHER.is_match(line);

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

// =====================================================================================
// STAGE 2 — PII detection & redaction
// =====================================================================================

static EMAIL_CANDIDATE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}").unwrap()
});

static PHONE_CANDIDATE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\+?\d[\d\s().\-]{7,16}\d").unwrap()
});

static CARD_CANDIDATE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(?:\d[ \-]?){13,19}\b").unwrap());

static IBAN_CANDIDATE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b[A-Z]{2}\d{2}[A-Z0-9]{10,30}\b").unwrap());

static SSN_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap());

pub fn redact_pii(input: &str, default_region: phonenumber::country::Id) -> String {
    let mut text = input.to_string();

    text = SSN_REGEX.replace_all(&text, "[REDACTED_SSN]").to_string();

    text = EMAIL_CANDIDATE_REGEX
        .replace_all(&text, |caps: &regex::Captures| {
            let candidate = &caps[0];
            if email_address::EmailAddress::is_valid(candidate) {
                "[REDACTED_EMAIL]".to_string()
            } else {
                candidate.to_string()
            }
        })
        .to_string();

    text = CARD_CANDIDATE_REGEX
        .replace_all(&text, |caps: &regex::Captures| {
            let candidate = &caps[0];
            match card_validate::Validate::from(candidate) {
                Ok(_) => "[REDACTED_CARD]".to_string(),
                Err(_) => candidate.to_string(),
            }
        })
        .to_string();

    text = IBAN_CANDIDATE_REGEX
        .replace_all(&text, |caps: &regex::Captures| {
            let candidate = &caps[0];
            match candidate.parse::<iban::Iban>() {
                Ok(_) => "[REDACTED_IBAN]".to_string(),
                Err(_) => candidate.to_string(),
            }
        })
        .to_string();

    text = PHONE_CANDIDATE_REGEX
        .replace_all(&text, |caps: &regex::Captures| {
            let candidate = &caps[0];
            match phonenumber::parse(Some(default_region), candidate) {
                Ok(number) if phonenumber::is_valid(&number) => "[REDACTED_PHONE]".to_string(),
                _ => candidate.to_string(),
            }
        })
        .to_string();

    text
}

// =====================================================================================
// STAGE 3 — Profanity filtering
// =====================================================================================

pub fn censor_profanity(input: &str) -> (String, bool) {
    use rustrict::{Censor, Type};

    let (censored, analysis) = Censor::from_str(input)
        .with_censor_threshold(Type::INAPPROPRIATE)
        .censor_and_analyze();

    (censored, analysis.is(Type::INAPPROPRIATE))
}

// =====================================================================================
// Combined pipeline
// =====================================================================================

#[allow(dead_code)]
pub struct SanitizeReport {
    pub text: String,
    pub flagged_profanity: bool,
}

pub fn sanitize(input: &str, region: phonenumber::country::Id) -> SanitizeReport {
    let stage1 = clean_stage1(input);
    let stage2 = redact_pii(&stage1, region);
    let (stage3, flagged_profanity) = censor_profanity(&stage2);

    SanitizeReport {
        text: stage3,
        flagged_profanity,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phonenumber::country::Id as Region;

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
    fn test_high_entropy_fallback() {
        let input = "internal_token: aZ8kQ2mN9pR4vT7xW1yB6cD3eF5gH0jK2lM4nO";
        let cleaned = clean_stage1(input);
        assert!(cleaned.contains("[REDACTED_HIGH_ENTROPY]"));
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

    #[test]
    fn test_email_redaction() {
        let input = "contact me at otmane@example.com for details";
        let cleaned = redact_pii(input, Region::FR);
        assert!(cleaned.contains("[REDACTED_EMAIL]"));
        assert!(!cleaned.contains("otmane@"));
    }

    #[test]
    fn test_card_number_luhn_check() {
        let valid = "card: 4532015112830366";
        let cleaned_valid = redact_pii(valid, Region::FR);
        assert!(cleaned_valid.contains("[REDACTED_CARD]"));

        let invalid = "ref: 1234567890123456";
        let cleaned_invalid = redact_pii(invalid, Region::FR);
        assert!(!cleaned_invalid.contains("[REDACTED_CARD]"));
    }

    #[test]
    fn test_ssn_redaction() {
        let input = "SSN on file: 123-45-6789";
        let cleaned = redact_pii(input, Region::FR);
        assert!(cleaned.contains("[REDACTED_SSN]"));
    }

    #[test]
    fn test_profanity_flagging() {
        let (_censored, flagged) = censor_profanity("this is fucking broken");
        assert!(flagged);

        let (_clean, not_flagged) = censor_profanity("this is completely fine");
        assert!(!not_flagged);
    }
}
