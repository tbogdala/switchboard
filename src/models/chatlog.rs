use serde::{Deserialize, Serialize};
use sycamore::prelude::*;

use super::config::ApiEndpointConfig;

// Convenience struct to handle JSON serialization of chatlog components.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct JSONChatlog {
    version: u16,
    api_settings: ApiEndpointConfig,
    system_message: String,
    messages: Vec<Message>,
}

/// Represents an individual chat message; combines the message content with metadata
/// about the message's origin (user or AI-generated) and a unique identifier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    pub message: String,
    pub ai_generated: bool,
    pub id: u32,
    pub image_base64: Option<String>, // optional base64 encoded image associated with message
}

// extracts think block from message, returning a tuple that represents
// (main_content, thinking_content) as strings. If the message doesn't have
// a detected 'thought' block at the start of the message, `None` will be returned.
pub fn parse_think_block(message: String) -> Option<(String, String)> {
    let message = message.trim();
    let think_start = "<think>";
    let think_end = "</think>";

    if message.starts_with(think_start) {
        if let Some(end) = message.find(think_end) {
            let think_content = &message[0..end + think_end.len()];
            let main_content = message[end + think_end.len()..].trim().to_string();
            Some((main_content, think_content.to_string()))
        } else {
            None
        }
    } else {
        None
    }
}

// Encapsulates all the data for a given 'chat log' in the application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chatlog {
    pub next_id: u32,
    pub messages: Signal<Vec<Message>>,
    pub response_generator: fn(),
}

impl Chatlog {
    // creates a new `Chatlog` with a `messages` signal that has an empty vector.
    pub fn new(response_generator: fn()) -> Self {
        Self {
            next_id: 1,
            messages: create_signal(vec![]),
            response_generator: response_generator,
        }
    }

    // given a JSON string, will attempt to desrialize it into a tupple consisting of a `Chatlog`,
    // an `ApiEndpointConifg` and a system message in `String` form.
    pub fn from_json(
        json_str: &str,
        response_generator: fn(),
    ) -> Result<(Self, ApiEndpointConfig, String), serde_json::Error> {
        let json_log: JSONChatlog = serde_json::from_str(json_str)?;

        let next_id = json_log
            .messages
            .iter()
            .map(|msg| msg.id)
            .max()
            .map_or(1, |max_id| max_id + 1);
        Ok((
            Self {
                next_id,
                messages: create_signal(json_log.messages),
                response_generator,
            },
            json_log.api_settings,
            json_log.system_message,
        ))
    }

    pub fn to_json(
        &self,
        api_settings: ApiEndpointConfig,
        system_message: String,
    ) -> Result<String, serde_json::Error> {
        let messages = self.messages.get_clone();
        let json_log = JSONChatlog {
            version: 1,
            messages,
            system_message,
            api_settings,
        };
        serde_json::to_string(&json_log)
    }

    pub fn clone_from(&mut self, other: &Self) {
        self.next_id = other.next_id;
        self.messages.set(other.messages.get_clone());
        self.response_generator = other.response_generator;
    }

    // returns the internal signal for the messages, which can be
    // useful to mark closures for tracking.
    pub fn get_messages_signal(&self) -> Signal<Vec<Message>> {
        return self.messages;
    }

    // adds a new `Message` to the chatlog and generates a new id for it.
    // `ai_gen` should be set to `false` if this message was human generated.
    // `image_base64` is an optional base64 encoded string for an image.
    pub fn add_msg(&mut self, new_msg: String, ai_gen: bool, image_base64: Option<String>) {
        self.messages.update(|msgs| {
            let new_id = self.get_next_id();
            msgs.push(Message {
                id: new_id,
                ai_generated: ai_gen,
                message: new_msg,
                image_base64,
            })
        });
    }

    // removes the `Message` with the matching id.
    pub fn remove_message(&mut self, id: u32) {
        self.messages.update(|msgs| {
            if let Some(index) = msgs.iter().position(|msg| msg.id == id) {
                let _message = msgs.remove(index);
            }
        });
    }

    // removes the `Message` with the matching id 'silently' to not trigger a signal update.
    pub fn remove_message_silent(&mut self, id: u32) {
        self.messages.update_silent(|msgs| {
            if let Some(index) = msgs.iter().position(|msg| msg.id == id) {
                let _message = msgs.remove(index);
            }
        });
    }

    // updates the message text for a `Message` with matching id.
    pub fn update_msg(&mut self, id: u32, new_msg: String) {
        self.messages.update(|msgs| {
            if let Some(msg) = msgs.iter_mut().find(|msg| msg.id == id) {
                msg.message = new_msg;
            }
        });
    }

    // call this to invoke a text generation request to create a `Message` response.
    // the `response_generator` member must be set to the function implementing
    // the response generation.
    pub fn trigger_response_generation(&self) {
        (self.response_generator)();
    }

    // internal helper function to generate the next id for messages.
    fn get_next_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}
