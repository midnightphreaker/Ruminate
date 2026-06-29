use serde_json::{Value, json};

use crate::models::ReflectInput;

#[derive(Debug, Clone)]
pub struct ReflectConfig {
    pub enabled: bool,
    pub base_url: Option<String>,
    pub api_key_present: bool,
    pub model: Option<String>,
    pub timeout_ms: u64,
    pub max_input_chars: usize,
}

impl ReflectConfig {
    pub fn from_env() -> Self {
        Self {
            enabled: env_bool("RUMINATE_LLM_ENABLED", false),
            base_url: std::env::var("RUMINATE_LLM_BASE_URL").ok(),
            api_key_present: std::env::var("RUMINATE_LLM_API_KEY").ok().is_some(),
            model: std::env::var("RUMINATE_LLM_MODEL").ok(),
            timeout_ms: std::env::var("RUMINATE_LLM_TIMEOUT_MS")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(30_000),
            max_input_chars: std::env::var("RUMINATE_LLM_MAX_INPUT_CHARS")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(20_000),
        }
    }
}

pub async fn reflect(input: ReflectInput) -> Value {
    let config = ReflectConfig::from_env();
    if input.input.chars().count() > config.max_input_chars {
        return json!({
            "advisory": true,
            "enabled": config.enabled,
            "status": "rejected",
            "error": "input exceeds RUMINATE_LLM_MAX_INPUT_CHARS",
            "model": config.model,
            "timeoutMs": config.timeout_ms,
        });
    }

    if !config.enabled {
        return json!({
            "advisory": true,
            "enabled": false,
            "status": "disabled",
            "purpose": input.purpose,
            "model": config.model,
        });
    }

    if config.base_url.is_none() || !config.api_key_present || config.model.is_none() {
        return json!({
            "advisory": true,
            "enabled": true,
            "status": "missing_config",
            "purpose": input.purpose,
            "model": config.model,
            "required": ["RUMINATE_LLM_BASE_URL", "RUMINATE_LLM_API_KEY", "RUMINATE_LLM_MODEL"],
        });
    }

    reflect_enabled(input, config).await
}

#[cfg(feature = "reflect")]
async fn reflect_enabled(input: ReflectInput, config: ReflectConfig) -> Value {
    use std::time::Duration;

    let client = match reqwest::Client::builder()
        .timeout(Duration::from_millis(config.timeout_ms))
        .build()
    {
        Ok(client) => client,
        Err(error) => {
            return json!({
                "advisory": true,
                "enabled": true,
                "status": "client_error",
                "error": error.to_string(),
                "model": config.model,
            });
        }
    };

    let base_url = config.base_url.expect("checked above");
    let api_key = std::env::var("RUMINATE_LLM_API_KEY").expect("checked above");
    let model = config.model.expect("checked above");
    let prompt = format!("Purpose: {:?}\n\nInput:\n{}", input.purpose, input.input);

    let response = client
        .post(base_url.trim_end_matches('/').to_string() + "/chat/completions")
        .bearer_auth(api_key)
        .json(&json!({
            "model": model,
            "messages": [
                {
                    "role": "system",
                    "content": "You are an advisory reflection helper for a local MCP workflow tool. Be concise."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        }))
        .send()
        .await;

    match response {
        Ok(response) if response.status().is_success() => {
            let status = response.status().as_u16();
            match response.json::<Value>().await {
                Ok(body) => json!({
                    "advisory": true,
                    "enabled": true,
                    "status": "ok",
                    "model": model,
                    "providerStatus": status,
                    "response": body,
                }),
                Err(error) => json!({
                    "advisory": true,
                    "enabled": true,
                    "status": "parse_error",
                    "model": model,
                    "error": error.to_string(),
                }),
            }
        }
        Ok(response) => json!({
            "advisory": true,
            "enabled": true,
            "status": "provider_error",
            "model": model,
            "providerStatus": response.status().as_u16(),
        }),
        Err(error) if error.is_timeout() => json!({
            "advisory": true,
            "enabled": true,
            "status": "timeout",
            "model": model,
        }),
        Err(error) => json!({
            "advisory": true,
            "enabled": true,
            "status": "request_error",
            "model": model,
            "error": error.to_string(),
        }),
    }
}

#[cfg(not(feature = "reflect"))]
async fn reflect_enabled(_input: ReflectInput, config: ReflectConfig) -> Value {
    json!({
        "advisory": true,
        "enabled": true,
        "status": "feature_disabled",
        "model": config.model,
    })
}

fn env_bool(name: &str, default: bool) -> bool {
    std::env::var(name)
        .ok()
        .map(|value| value.eq_ignore_ascii_case("true"))
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use std::sync::OnceLock;
    use tokio::sync::{Mutex, MutexGuard};

    use crate::models::ReflectPurpose;

    use super::*;

    async fn env_lock() -> MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().await
    }

    #[tokio::test]
    async fn reflect_disabled_by_default() {
        let _guard = env_lock().await;
        unsafe {
            std::env::remove_var("RUMINATE_LLM_ENABLED");
            std::env::remove_var("RUMINATE_LLM_MAX_INPUT_CHARS");
        }
        let output = reflect(ReflectInput {
            purpose: ReflectPurpose::Summarize,
            input: "hello".to_string(),
        })
        .await;
        assert_eq!(output["status"], "disabled");
        assert_eq!(output["advisory"], true);
    }

    #[tokio::test]
    async fn reflect_rejects_large_input() {
        let _guard = env_lock().await;
        unsafe {
            std::env::set_var("RUMINATE_LLM_MAX_INPUT_CHARS", "3");
        }
        let output = reflect(ReflectInput {
            purpose: ReflectPurpose::Summarize,
            input: "hello".to_string(),
        })
        .await;
        assert_eq!(output["status"], "rejected");
        unsafe {
            std::env::remove_var("RUMINATE_LLM_MAX_INPUT_CHARS");
        }
    }

    #[tokio::test]
    async fn reflect_reports_missing_config() {
        let _guard = env_lock().await;
        unsafe {
            std::env::set_var("RUMINATE_LLM_ENABLED", "true");
            std::env::remove_var("RUMINATE_LLM_MAX_INPUT_CHARS");
            std::env::remove_var("RUMINATE_LLM_BASE_URL");
            std::env::remove_var("RUMINATE_LLM_API_KEY");
            std::env::remove_var("RUMINATE_LLM_MODEL");
        }
        let output = reflect(ReflectInput {
            purpose: ReflectPurpose::Handoff,
            input: "state".to_string(),
        })
        .await;
        assert_eq!(output["status"], "missing_config");
        unsafe {
            std::env::remove_var("RUMINATE_LLM_ENABLED");
        }
    }
}
