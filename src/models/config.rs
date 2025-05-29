use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct ApiEndpointConfig {
    pub name: String,     // user friendly name
    pub endpoint: String, // API endpoint URL
    pub api_key: String,  // API key string
    pub model_id: String, // the model for the API to use

    pub temperature: Option<String>,
    pub top_p: Option<String>,
    pub top_k: Option<String>,
    pub min_p: Option<String>,
    pub repetition_penalty: Option<String>,

    pub max_tokens: Option<String>,
    pub target_context_size: Option<String>,
}

impl Default for ApiEndpointConfig {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            endpoint: "https://openrouter.ai/api/v1".to_string(),
            api_key: String::new(),
            model_id: "google/gemma-3-27b-it:free".to_string(),
            temperature: None,
            top_p: None,
            top_k: None,
            min_p: None,
            repetition_penalty: None,
            max_tokens: None,
            target_context_size: None,
        }
    }
}

impl ApiEndpointConfig {
    // returns the temperature string converted to f32 or a default value of 1.0
    pub fn get_temperature(&self) -> f32 {
        self.temperature
            .as_ref()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1.0)
    }

    // returns the top_p string converted to f32 or a default value of 1.0
    pub fn get_top_p(&self) -> f32 {
        self.top_p
            .as_ref()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1.0)
    }

    // returns the top_k string converted to u32 or a default value of 0
    pub fn get_top_k(&self) -> u32 {
        self.top_k
            .as_ref()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    }

    // returns the min_p string converted to f32 or a default value of 0.0
    pub fn get_min_p(&self) -> f32 {
        self.min_p
            .as_ref()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0)
    }

    // returns the repetition_penalty string converted to f32 or a default value of 1.0
    pub fn get_repetition_penalty(&self) -> f32 {
        self.repetition_penalty
            .as_ref()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1.0)
    }

    // returns the max_tokens string converted to u32 or a default value of 100
    pub fn get_max_tokens(&self) -> u32 {
        self.max_tokens
            .as_ref()
            .and_then(|s| s.parse().ok())
            .unwrap_or(100)
    }

    // returns the target_context_size string converted to u32 or a default value of 8192
    pub fn get_target_context_size(&self) -> u32 {
        self.target_context_size
            .as_ref()
            .and_then(|s| s.parse().ok())
            .unwrap_or(4096)
    }
}
