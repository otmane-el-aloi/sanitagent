pub mod diff;
pub mod stage1;
pub mod stage2;
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

    // Stage 1: Deterministic Rule-Based Cleaner (<1ms)
    let stage1_output = stage1::clean_stage1(&raw_text);

    // Stage 2: High-Speed Local LLM Distillation (with 3s hard timeout & fallback)
    let (sanitized_output, is_distilled) = stage2::distill_stage2(&stage1_output).await;

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
