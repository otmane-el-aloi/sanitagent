use std::time::Duration;
use serde_json::Value;
use tokio::time::timeout;

const OLLAMA_BASE_URL: &str = "http://127.0.0.1:11434";
const PRIMARY_MODEL: &str = "qwen2.5:1.5b";
const HEALTHCHECK_TIMEOUT: Duration = Duration::from_secs(1);
const DISTILLATION_TIMEOUT: Duration = Duration::from_secs(15);

/// High-Speed Local LLM Distillation (Ollama Endpoint Bridge)
/// Checks if Ollama is online and attempts local distillation with a bounded timeout.
pub async fn distill_stage2(stage1_text: &str) -> (String, bool) {
    let trimmed = stage1_text.trim();
    if trimmed.is_empty() {
        return (String::new(), false);
    }

    // Fast path 1: if input is short (< 30 chars or single line < 60 chars),
    // distillation adds no value and we avoid wasting LLM inference overhead.
    if trimmed.len() < 30 || (!trimmed.contains('\n') && trimmed.len() < 60) {
        return (stage1_text.to_string(), false);
    }

    let installed_model = match is_ollama_online_and_get_model().await {
        Some(model) => model,
        None => return (stage1_text.to_string(), false),
    };

    match timeout(
        DISTILLATION_TIMEOUT,
        query_ollama_distillation(stage1_text, &installed_model),
    )
    .await
    {
        Ok(Ok(distilled)) if !distilled.trim().is_empty() => {
            let cleaned = clean_llm_output(&distilled);

            // Safety Retention Threshold: Ensure the LLM didn't over-summarize/strip essential context.
            // Distilled text must retain at least 20% of original character length (or at least 30 chars).
            let min_expected_len = (trimmed.len() as f64 * 0.20).max(30.0) as usize;

            if !cleaned.is_empty() && cleaned != stage1_text && cleaned.len() >= min_expected_len {
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
async fn query_ollama_distillation(input: &str, model_name: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .timeout(DISTILLATION_TIMEOUT)
        .build()?;

    let prompt = format!(
        "You are an expert log and code sanitizer. Your job is NOT to summarize or shorten the text arbitrarily. Keep all critical error messages, stack trace lines, exception types, line numbers, and relevant technical context intact. Only remove high-entropy secret noise, redundant repetitive lines, and unneeded framework wrapper stack frames. Do not add markdown explanations or conversational text. Output the sanitized log directly.\n\nLOG:\n{}",
        input
    );

    // 1. Query Ollama native /api/generate endpoint
    let ollama_payload = serde_json::json!({
        "model": model_name,
        "prompt": prompt,
        "stream": false,
        "options": {
            "num_predict": 512,
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

    // 2. Fallback: Query Ollama's OpenAI-compatible /v1/chat/completions endpoint
    let chat_payload = serde_json::json!({
        "model": model_name,
        "messages": [
            {
                "role": "system",
                "content": "You are an expert log and code sanitizer. Your job is NOT to summarize or shorten the text arbitrarily. Keep all critical error messages, stack trace lines, exception types, and technical context intact. Only remove high-entropy noise, redundant lines, and framework wrapper frames. Output the sanitized log directly without conversational filler or markdown wrapper."
            },
            {"role": "user", "content": input}
        ],
        "max_tokens": 512,
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

/// Verifies that Ollama is online and returns an installed model name.
async fn is_ollama_online_and_get_model() -> Option<String> {
    let ping_client = match reqwest::Client::builder()
        .timeout(HEALTHCHECK_TIMEOUT)
        .build()
    {
        Ok(c) => c,
        Err(_) => return None,
    };

    let tags_url = format!("{}/api/tags", OLLAMA_BASE_URL);
    if let Ok(res) = ping_client.get(&tags_url).send().await {
        if res.status().is_success() {
            if let Ok(json) = res.json::<Value>().await {
                if let Some(models) = json.get("models").and_then(|m| m.as_array()) {
                    return select_model(models);
                }
            }
        }
    }

    None
}

fn select_model(models: &[Value]) -> Option<String> {
    models
        .iter()
        .filter_map(|model| model.get("name").and_then(Value::as_str))
        .find(|name| *name == PRIMARY_MODEL)
        .or_else(|| {
            models
                .iter()
                .filter_map(|model| model.get("name").and_then(Value::as_str))
                .next()
        })
        .map(str::to_owned)
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

    #[test]
    fn test_select_model_prefers_primary() {
        let models = vec![
            serde_json::json!({ "name": "gemma4:e4b" }),
            serde_json::json!({ "name": PRIMARY_MODEL }),
        ];

        assert_eq!(select_model(&models).as_deref(), Some(PRIMARY_MODEL));
    }

    #[test]
    fn test_select_model_requires_installed_model() {
        assert_eq!(select_model(&[]), None);
    }
}

