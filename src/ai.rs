use serde_json::{json, Value};

#[derive(Clone, Debug)]
pub struct AiReply {
    pub ok: bool,
    pub text: String,
}

pub fn ask(endpoint: &str, model: &str, key: &str, instruction: &str, context: &str) -> AiReply {
    if key.trim().is_empty() {
        return AiReply {
            ok: false,
            text: "Add an API key in Settings → AI to enable assistant features.".to_string(),
        };
    }
    let endpoint = endpoint.trim();
    if endpoint.is_empty() {
        return AiReply {
            ok: false,
            text: "Add a chat-completions endpoint in Settings → AI.".to_string(),
        };
    }

    let system = format!(
        "You are a concise, helpful writing and coding assistant inside a note-taking editor. {}",
        instruction
    );
    let body = json!({
        "model": model,
        "messages": [
            { "role": "system", "content": system },
            { "role": "user", "content": context }
        ],
        "temperature": 0.3,
    });

    let request = ureq::post(endpoint)
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {}", key));

    let response = match request.send_json(&body) {
        Ok(response) => response,
        Err(error) => {
            return AiReply {
                ok: false,
                text: format!("Request failed: {}", error),
            }
        }
    };
    let mut response = response;
    let raw = match response.body_mut().read_to_string() {
        Ok(raw) => raw,
        Err(error) => {
            return AiReply {
                ok: false,
                text: format!("Could not read response: {}", error),
            }
        }
    };
    parse_reply(&raw)
}

fn parse_reply(raw: &str) -> AiReply {
    let value: Value = match serde_json::from_str(raw) {
        Ok(value) => value,
        Err(_) => {
            return AiReply {
                ok: false,
                text: if raw.trim().is_empty() {
                    "Empty response from the AI endpoint.".to_string()
                } else {
                    raw.to_string()
                },
            }
        }
    };
    if let Some(message) = value["error"]["message"].as_str() {
        return AiReply {
            ok: false,
            text: format!("API error: {}", message),
        };
    }
    let text = value["choices"][0]["message"]["content"]
        .as_str()
        .or_else(|| value["choices"][0]["text"].as_str())
        .or_else(|| value["response"].as_str())
        .or_else(|| value["message"].as_str())
        .unwrap_or("The endpoint returned no usable text.")
        .to_string();
    AiReply { ok: true, text }
}
