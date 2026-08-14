use std::time::Duration;
use serde_json::Value;
use tokio::time::timeout;

const OLLAMA_BASE_URL: &str = "http://127.0.0.1:11434";
const PRIMARY_MODEL: &str = "qwen2.5-coder";

/// High-Speed Local LLM Distillation (Ollama Endpoint Bridge)
/// Attempts local distillation via Ollama with a 10-second hard timeout.
/// Fallback: If model inference fails, times out (>10s), or Ollama is offline,
/// gracefully falls back to the Stage 1 rule-cleaned text.
pub async fn distill_stage2(stage1_text: &str) -> (String, bool) {
    let trimmed = stage1_text.trim();
    if trimmed.is_empty() {
        return (String::new(), false);
    }

    // Fast path: if input is short (< 30 chars or single line < 60 chars),
    // distillation adds no value and we avoid wasting LLM inference overhead.
    if trimmed.len() < 30 || (!trimmed.contains('\n') && trimmed.len() < 60) {
        return (stage1_text.to_string(), false);
    }

    // Attempt local LLM distillation with a 10000ms timeout
    let timeout_duration = Duration::from_millis(10000);

    match timeout(timeout_duration, query_ollama_distillation(stage1_text)).await {
        Ok(Ok(distilled)) if !distilled.trim().is_empty() => {
            let cleaned = clean_llm_output(&distilled);
            if !cleaned.is_empty() && cleaned != stage1_text {
                (cleaned, true)
            } else {
                (stage1_text.to_string(), false)
            }
        }
        _ => {
            // Graceful fallback to Stage 1 cleaned text
            (stage1_text.to_string(), false)
        }
    }
}

/// Queries local Ollama endpoint (http://127.0.0.1:11434)
async fn query_ollama_distillation(input: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(9500))
        .build()?;

    let prompt = format!(
        "Compress the following error log into a minimal, essential error summary. Remove all boilerplate, timestamps, and redundant stack frames. Output ONLY the core error details.\n\nLOG:\n{}",
        input
    );

    // 1. Determine target model using fast tag lookup
    let model_name = get_working_ollama_model().await;

    // 2. Query Ollama native /api/generate endpoint
    let ollama_payload = serde_json::json!({
        "model": model_name,
        "prompt": prompt,
        "stream": false,
        "options": {
            "num_predict": 256,
            "temperature": 0.1
        }
    });

    let generate_url = format!("{}/api/generate", OLLAMA_BASE_URL);
    if let Ok(res) = client.post(&generate_url).json(&ollama_payload).send().await {
        if res.status().is_success() {
            if let Ok(json) = res.json::<Value>().await {
                if let Some(text) = json.get("response").and_then(|v| v.as_str()) {
                    if !text.trim().is_empty() {
                        return Ok(text.trim().to_string());
                    }
                }
            }
        }
    }

    // 3. Fallback: Query Ollama's OpenAI-compatible /v1/chat/completions endpoint
    let chat_payload = serde_json::json!({
        "model": model_name,
        "messages": [
            {"role": "system", "content": "You are a log sanitizer context extractor. Extract only the essential error message and minimal reproduction log. No markdown explanations or conversational text."},
            {"role": "user", "content": input}
        ],
        "max_tokens": 256,
        "temperature": 0.1
    });

    let chat_url = format!("{}/v1/chat/completions", OLLAMA_BASE_URL);
    if let Ok(res) = client.post(&chat_url).json(&chat_payload).send().await {
        if res.status().is_success() {
            if let Ok(json) = res.json::<Value>().await {
                if let Some(text) = json["choices"][0]["message"]["content"].as_str() {
                    if !text.trim().is_empty() {
                        return Ok(text.trim().to_string());
                    }
                }
            }
        }
    }

    Err("Ollama distillation query failed or returned empty response".into())
}

/// Discovers installed Ollama models via /api/tags using a fast 800ms client
async fn get_working_ollama_model() -> String {
    let fast_client = match reqwest::Client::builder()
        .timeout(Duration::from_millis(800))
        .build()
    {
        Ok(c) => c,
        Err(_) => return PRIMARY_MODEL.to_string(),
    };

    let tags_url = format!("{}/api/tags", OLLAMA_BASE_URL);
    if let Ok(res) = fast_client.get(&tags_url).send().await {
        if res.status().is_success() {
            if let Ok(json) = res.json::<Value>().await {
                if let Some(models) = json.get("models").and_then(|m| m.as_array()) {
                    // Check if primary model is installed
                    for model in models {
                        if let Some(name) = model.get("name").and_then(|n| n.as_str()) {
                            if name.contains(PRIMARY_MODEL) {
                                return name.to_string();
                            }
                        }
                    }
                    // Primary model not found; fallback to first available installed model
                    if let Some(first) = models.first() {
                        if let Some(name) = first.get("name").and_then(|n| n.as_str()) {
                            return name.to_string();
                        }
                    }
                }
            }
        }
    }

    PRIMARY_MODEL.to_string()
}

/// Cleans raw LLM outputs (removes wrapping ```codeblocks``` and leading/trailing whitespace)
pub fn clean_llm_output(output: &str) -> String {
    let mut text = output.trim();

    // Strip leading ``` (e.g. ```text or ```log)
    if text.starts_with("```") {
        if let Some(first_line_end) = text.find('\n') {
            text = text[first_line_end + 1..].trim_start();
        } else {
            text = text.trim_start_matches('`');
        }
    }

    // Strip trailing ```
    if text.ends_with("```") {
        if let Some(last_fence) = text.rfind("```") {
            text = text[..last_fence].trim_end();
        } else {
            text = text.trim_end_matches('`');
        }
    }

    text.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_llm_output_markdown_fence() {
        let raw = "```text\nError: connection failed\n    at main.js:10\n```";
        assert_eq!(clean_llm_output(raw), "Error: connection failed\n    at main.js:10");

        let raw2 = "```log\n[ERROR] Database timeout\n```";
        assert_eq!(clean_llm_output(raw2), "[ERROR] Database timeout");
    }

    #[test]
    fn test_clean_llm_output_plain() {
        let raw = "Error: Invalid API token";
        assert_eq!(clean_llm_output(raw), "Error: Invalid API token");
    }

    #[tokio::test]
    async fn test_distill_empty_input() {
        let (output, is_distilled) = distill_stage2("   ").await;
        assert_eq!(output, "");
        assert!(!is_distilled);
    }

    #[tokio::test]
    async fn test_distill_short_input_fastpath() {
        let input = "Error 404: Not Found";
        let (output, is_distilled) = distill_stage2(input).await;
        assert_eq!(output, input);
        assert!(!is_distilled);
    }

    #[tokio::test]
    async fn test_distill_offline_fallback() {
        let long_input = "2026-08-14T16:00:00Z [ERROR] Failed connection to database\nLine 2 info\nLine 3 trace details";
        let (output, is_distilled) = distill_stage2(long_input).await;
        assert_eq!(output, long_input);
        assert!(!is_distilled);
    }
}

