pub mod deterministic_sanitizer;
pub mod diff;
pub mod llm_sanitizer;
pub mod tokens;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizationResult {
    pub raw_text: String,
    pub sanitized_text: String,
    pub stage1_text: String,
    pub is_distilled: bool,
    pub latency_ms: u128,
    pub token_stats: tokens::TokenStats,
    pub diff_lines: Vec<diff::DiffLine>,
}

pub async fn process_pipeline(raw_text: String) -> SanitizationResult {
    let start_time = std::time::Instant::now();

    // Deterministic Multi-stage Cleaner (Secrets + Log Noise + Validated PII + Profanity)
    let deterministic_report =
        deterministic_sanitizer::sanitize(&raw_text, phonenumber::country::Id::FR);
    let stage1_output = deterministic_report.text;

    // Optional Stage: Local LLM distillation with bounded Ollama request
    let (distilled_output, is_distilled) = llm_sanitizer::distill_llm(&stage1_output).await;

    // Safety Pass: Re-run deterministic rules on LLM output to guarantee no leaked secrets or PII
    let sanitized_output = if is_distilled {
        deterministic_sanitizer::sanitize(&distilled_output, phonenumber::country::Id::FR).text
    } else {
        stage1_output.clone()
    };

    // Token Statistics Calculation
    let token_stats = tokens::calculate_stats(&raw_text, &sanitized_output);

    // Unified Diff Generation
    let diff_lines = diff::generate_diff(&raw_text, &sanitized_output);

    let latency_ms = start_time.elapsed().as_millis();

    SanitizationResult {
        raw_text,
        sanitized_text: sanitized_output,
        stage1_text: stage1_output,
        is_distilled,
        latency_ms,
        token_stats,
        diff_lines,
    }
}
