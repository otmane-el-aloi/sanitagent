use std::time::Duration;
use tokio::time::timeout;

/// High-Speed Local LLM Distillation (Native Rust Inference & Endpoint Bridge)
/// Attempts local distillation with a strict 3-second hard timeout.
/// Fallback: If model inference fails, times out (>3s), or server is offline,
/// gracefully falls back to the Stage 1 rule-cleaned text.
pub async fn distill_stage2(stage1_text: &str) -> (String, bool) {
    if stage1_text.trim().is_empty() {
        return (String::new(), false);
    }

    // Attempt local LLM distillation with a 3000ms timeout
    let timeout_duration = Duration::from_millis(3000);

    match timeout(timeout_duration, query_local_distillation(stage1_text)).await {
        Ok(Ok(distilled)) if !distilled.trim().is_empty() => {
            (distilled, true)
        }
        _ => {
            // Graceful fallback to Stage 1 cleaned text
            (stage1_text.to_string(), false)
        }
    }
}

/// Queries local LLM endpoint (Ollama / llama-cpp server / local GGUF bridge)
async fn query_local_distillation(input: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(2800))
        .build()?;

    let prompt = format!(
        "Extract the core issue, error message, and minimal context from the log below. Do not add conversational filler.\n\nLOG:\n{}",
        input
    );

    // Try standard local Ollama endpoint first (127.0.0.1:11434)
    let ollama_payload = serde_json::json!({
        "model": "qwen2.5-coder",
        "prompt": prompt,
        "stream": false,
        "options": {
            "num_predict": 256,
            "temperature": 0.1
        }
    });

    if let Ok(res) = client.post("http://127.0.0.1:11434/api/generate")
        .json(&ollama_payload)
        .send()
        .await
    {
        if res.status().is_success() {
            if let Ok(json) = res.json::<serde_json::Value>().await {
                if let Some(text) = json.get("response").and_then(|v| v.as_str()) {
                    return Ok(text.trim().to_string());
                }
            }
        }
    }

    // Try OpenAI-compatible local llama-server (127.0.0.1:8080 or 127.0.0.1:1234)
    let open_ai_payload = serde_json::json!({
        "messages": [
            {"role": "system", "content": "You are a log sanitizer distilled context extractor. Extract only the core error and minimal reproducer context."},
            {"role": "user", "content": input}
        ],
        "max_tokens": 256,
        "temperature": 0.1
    });

    for port in &[8080, 1234] {
        let url = format!("http://127.0.0.1:{}/v1/chat/completions", port);
        if let Ok(res) = client.post(&url).json(&open_ai_payload).send().await {
            if res.status().is_success() {
                if let Ok(json) = res.json::<serde_json::Value>().await {
                    if let Some(text) = json["choices"][0]["message"]["content"].as_str() {
                        return Ok(text.trim().to_string());
                    }
                }
            }
        }
    }

    Err("No active local LLM endpoint available".into())
}
