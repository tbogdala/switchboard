use anyhow::anyhow;
use reqwasm::http::Request;
use serde_json::{Value, json};
use sycamore::prelude::*;

use crate::models::{
    chatlog::{parse_think_block, Message, StackedMessage},
    config::ApiEndpointConfig,
    system_message::SystemMessage,
};

const TOTAL_API_LIMIT: u32 = 16000;
const RESPONSE_RESERVATION: u32 = 2000;
const MAX_REQUEST_TOKENS: u32 = TOTAL_API_LIMIT - RESPONSE_RESERVATION;
const CHARS_PER_TOKEN_ESTIMATE: f32 = 4.0;

fn estimate_tokens(text: &str) -> u32 {
    let count = text.chars().count();
    if count == 0 {
        0
    } else {
        (count as f32 / CHARS_PER_TOKEN_ESTIMATE) as u32
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct CompletionResponse {
    pub text: String,
    pub completion_tokens: Option<i64>,
    pub predicted_ms: Option<f64>,
    pub prompt_tokens: Option<i64>,
    pub prompt_ms: Option<f64>,
}

/// Sends a chat request to the API with the given messages and handles the response.
pub fn send_chat_completion_request<F>(msgs: Vec<Message>, is_regenerating: bool, on_response: F)
where
    F: FnOnce(anyhow::Result<CompletionResponse>) + 'static,
{
    // pull the api endpoint configuration from the context
    let config_context_signal = use_context::<Signal<ApiEndpointConfig>>();
    let api_config = config_context_signal.get_clone();
    console_log!(
        "Completion request being initiated for endpoint: {:?}",
        api_config
    );

    let mut working_token_budget = if api_config.target_context_size.is_none() {
        MAX_REQUEST_TOKENS
    } else {
        api_config.get_target_context_size()
    };
    let max_working_token_budget = working_token_budget;
    console_log!("Working token budget: {}", max_working_token_budget);
    let mut messages: Vec<_> = Vec::new();

    // get the system message in context
    let system_message_context = use_context::<SystemMessage>();
    let system_message = system_message_context.signal().get_clone();
    let system_message_trimmed = system_message.trim();
    working_token_budget -= estimate_tokens(system_message_trimmed);

    // construct the message history list
    let mut regen_skip = is_regenerating;
    let mut first_message = true;
    for m in msgs.iter().rev() {
        // before the `first_message` skip, we take out the most recent message
        // if we're regenerating it.
        if regen_skip {
            regen_skip = false;
            continue;
        }

        // we need to remove the thinking content when sending in messages as this
        // is currently considered best practice.
        let current_message = m.get_selected_message().unwrap_or_else(|| {
            debug_assert!(false, "get_selected_message() returned None in a context it should have one.");
            StackedMessage::default()
        });

        let content = match parse_think_block(current_message.message.clone()) {
            Some((main_content, _)) => main_content,
            None => current_message.message.clone(),
        };
        let msg_token_est = estimate_tokens(&content);
        if msg_token_est <= working_token_budget {
            let mut added_msg_already = false;

            // only when processing the first message do we look for image data.
            // if the image data is present, then we have to encode our JSON
            // request object differently to pair the message with the image.
            if first_message {
                if let Some(image_base64) = &current_message.image_base64 {
                    messages.push(json!({
                        "role": "user",
                        "content": [
                            {
                                "type": "text",
                                "text": content,
                            },
                            {
                                "type": "image_url",
                                "image_url": {
                                    "url": image_base64,
                                },
                            }
                        ]
                    }));
                    added_msg_already = true;
                }
                first_message = false;
            }

            if !added_msg_already {
                messages.push(json!({
                    "role": if m.ai_generated { "assistant" } else { "user" },
                    "content": content,
                }));
            }
            // console_log!("Adding {} tokens ({} remaining) of message: {}", msg_token_est, working_token_budget, content);
            working_token_budget -= msg_token_est;
        } else {
            break;
        }
    }
    messages.reverse();

    // if the system message is present, we insert it at the beginning of the message
    // stack and label the role appropriately.
    if !system_message_trimmed.is_empty() {
        messages.insert(
            0,
            json!({
                "role": "system",
                "content": system_message_trimmed,
            }),
        );
    }

    // Debug writing out the messages chosen for the prompt.
    // for m in messages.iter() {
    //     console_log!("Message: {:?}", m);
    // }
    console_log!(
        "A total of {} messages sent; The system message is approx. {} tokens; Toal estimated: {}.",
        messages.len(),
        estimate_tokens(&system_message_trimmed),
        max_working_token_budget - working_token_budget
    );

    // build the JSON body of the request and optionally add in advance parameters
    // if specified by the user in the API configuration.
    let mut request_body = json!({
        "model": api_config.model_id,
        "messages": messages,
    });
    if api_config.max_tokens.is_some() {
        request_body["max_tokens"] = json!(api_config.get_max_tokens());
    }
    if api_config.temperature.is_some() {
        request_body["temperature"] = json!(api_config.get_temperature());
    }
    if api_config.top_p.is_some() {
        request_body["top_p"] = json!(api_config.get_top_p());
    }
    if api_config.top_k.is_some() {
        request_body["top_k"] = json!(api_config.get_top_k());
    }
    if api_config.min_p.is_some() {
        request_body["min_p"] = json!(api_config.get_min_p());
    }
    if api_config.repetition_penalty.is_some() {
        request_body["repetition_penalty"] = json!(api_config.get_repetition_penalty());
    }
    //console_log!("DEBUG: request body: {}", request_body);

    // make a POST request to the API
    wasm_bindgen_futures::spawn_local(async move {
        let result = Request::post(&format!("{}/chat/completions", api_config.endpoint))
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", api_config.api_key))
            .header("HTTP-Referer", "https://github.com/tbogdala/switchboard")
            .header("X-Title", "Switchboard!")
            .body(request_body.to_string())
            .send()
            .await;

        match result {
            Ok(response) => {
                if response.ok() {
                    match response.text().await {
                        Ok(text) => {
                            // console_log!("DEBUG: Response text: {}", &text);
                            if let Ok(response) = extract_chat_completion_response(&text) {
                                on_response(Ok(response));
                            }
                        }
                        Err(e) => {
                            console_log!("Error reading response text: {}", e.to_string());
                            on_response(Err(anyhow!(
                                "Error reading response text: {}",
                                e.to_string()
                            )));
                        }
                    }
                } else {
                    match response.text().await {
                        Ok(error_text) => {
                            // Try to parse the error response JSON
                            if let Ok(error_json) =
                                serde_json::from_str::<serde_json::Value>(&error_text)
                            {
                                if let Some(message) = error_json["error"]["message"].as_str() {
                                    console_log!("API request failed: {}", message);
                                    on_response(Err(anyhow!("API request failed: {}", message)));
                                } else {
                                    console_log!(
                                        "API request failed with unexpected error format: {}",
                                        error_text
                                    );
                                    on_response(Err(anyhow!("API request failed: {}", error_text)));
                                }
                            } else {
                                console_log!(
                                    "API request failed with non-JSON response: {}",
                                    error_text
                                );
                                on_response(Err(anyhow!("API request failed: {}", error_text)));
                            }
                        }
                        Err(e) => {
                            console_log!(
                                "API request failed and couldn't read error response: {}",
                                e.to_string()
                            );
                            on_response(Err(anyhow!(
                                "API request failed and couldn't read error response"
                            )));
                        }
                    }
                }
            }
            Err(e) => {
                console_log!("Error sending request: {}", e.to_string());
                on_response(Err(anyhow!("Error sending request: {}", e.to_string())));
            }
        }
    });
}

// Takes the raw JSON returned by OpenAI compatible endpoints and parses out the information we want.
fn extract_chat_completion_response(json_response: &str) -> Result<CompletionResponse, String> {
    // Parse the JSON string into a serde_json::Value
    let parsed_value: Value =
        serde_json::from_str(json_response).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    // Navigate the JSON structure to find the content
    // Access `choices` array -> first element -> `message` object -> `content` string
    let content = parsed_value
        .get("choices")
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(|content| content.as_str());
    let reasoning = parsed_value
        .get("choices")
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("reasoning"))
        .and_then(|reasoning| reasoning.as_str());

    // Pull the timings out from the response if they're present.
    let completion_tokens = parsed_value
        .get("usage")
        .and_then(|usage| usage.get("completion_tokens"))
        .and_then(|tokens| tokens.as_i64());
    let predicted_ms = parsed_value
        .get("timings")
        .and_then(|usage| usage.get("predicted_ms"))
        .and_then(|tokens| tokens.as_f64());
    let prompt_tokens = parsed_value
        .get("usage")
        .and_then(|usage| usage.get("prompt_tokens"))
        .and_then(|tokens| tokens.as_i64());
    let prompt_ms = parsed_value
        .get("timings")
        .and_then(|usage| usage.get("prompt_ms"))
        .and_then(|tokens| tokens.as_f64());

    // if reasoning tokens is given, include those in the output.
    if let Some(text) = content {
        if let Some(reason_header) = reasoning {
            Ok(CompletionResponse {
                text: format!("<think>{}</think>\n{}", reason_header.trim(), text.trim()),
                completion_tokens,
                predicted_ms,
                prompt_tokens,
                prompt_ms,
            })
        } else {
            Ok(CompletionResponse {
                text: text.trim().to_string(),
                completion_tokens,
                predicted_ms,
                prompt_tokens,
                prompt_ms,
            })
        }
    } else {
        Err("Could not find 'choices[0].message.content' in the JSON".to_string())
    }
}
