use serde::{Deserialize, Serialize};
use sycamore::prelude::*;

use super::config::ApiEndpointConfig;

const CHAT_LOG_METADATA_VERSION: u16 = 1;

// Metadata for an individual saved chat logs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatLogMetadataEntry {
    pub id: String,              // guid
    pub title: String,           // user-provided name for the chat log
    pub last_accessed_time: i64, // when it was last accessed
    pub storage_key: String,     // LocalStorage key where this chat log is stored
    pub message_count: usize,    // number of messages in this chat log
}

impl Default for ChatLogMetadataEntry {
    fn default() -> Self {
        Self {
            id: Default::default(),
            title: Default::default(),
            last_accessed_time: Default::default(),
            storage_key: Default::default(),
            message_count: Default::default(),
        }
    }
}

// Represents the metadata for all saved chat logs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatLogMetadata {
    pub version: u16,                          // Version of this metadata format
    pub saved_logs: Vec<ChatLogMetadataEntry>, // List of all saved chat logs
}

impl Default for ChatLogMetadata {
    fn default() -> Self {
        Self {
            version: CHAT_LOG_METADATA_VERSION,
            saved_logs: Vec::new(),
        }
    }
}

impl ChatLogMetadata {
    // creates a new metadata repository for chat logs
    pub fn new() -> Self {
        Self::default()
    }
}

// Convenience struct to handle JSON serialization of chatlog components.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct JSONChatlog {
    version: u16,
    api_settings: ApiEndpointConfig,
    system_message: String,
    messages: Vec<Message>,
}

// Represents an individual chat message generation that encapsulates the
// generated text and can be extended in the future.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StackedMessage {
    pub message: String,
    pub image_base64: Option<String>, // optional base64 encoded image associated with message

                                      // TODO: generation timing stats
                                      // TODO: record API settings used
}

/// Represents an individual chat message; combines the message content with metadata
/// about the message's origin (user or AI-generated) and a unique identifier.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    pub ai_generated: bool,
    pub id: u32,

    #[serde(default)]
    pub message_stack: Vec<StackedMessage>, // all generated variants
    #[serde(default)]
    pub selected_message: usize, // index of the message in `message_stack` that is the 'chosen' one to be shown
}

impl Message {
    // returns an possible reference to the 'selected' generated message in the
    // message stack. if no messages exit or the `selected_message` index is not
    // valid, `None` will be returned.
    pub fn get_selected_message(&self) -> Option<StackedMessage> {
        self.message_stack.get(self.selected_message).cloned()
    }

    // sets a currently existing stacked message to a new object. the function
    // does nothing if the index is not found.
    pub fn set_selected_message(&mut self, new_item: StackedMessage) {
        if let Some(stacked_msg) = self.message_stack.get_mut(self.selected_message) {
            *stacked_msg = new_item;
        }
    }
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
// Note: Remember to update `clone_from()` when adding more signals.
#[derive(Debug, Clone)]
pub struct Chatlog {
    pub next_id: Signal<u32>,
    pub messages: Signal<Vec<Message>>,
    pub response_generator: fn(),
    pub is_regenerating_msg: Signal<bool>,
}

impl Chatlog {
    // creates a new `Chatlog` with a `messages` signal that has an empty vector.
    pub fn new(response_generator: fn()) -> Self {
        Self {
            next_id: create_signal(1),
            messages: create_signal(vec![]),
            response_generator: response_generator,
            is_regenerating_msg: create_signal(false),
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
                next_id: create_signal(next_id),
                messages: create_signal(json_log.messages),
                response_generator,
                is_regenerating_msg: create_signal(false),
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
        self.next_id.set(other.next_id.get_clone_untracked());
        self.messages.set(other.messages.get_clone_untracked());
        self.is_regenerating_msg
            .set(other.is_regenerating_msg.get_clone_untracked());
        self.response_generator = other.response_generator;
    }

    // returns the internal signal for the messages, which can be
    // useful to mark closures for tracking.
    pub fn get_messages_signal(&self) -> Signal<Vec<Message>> {
        return self.messages;
    }

    pub fn track_message(&self, msg_id: u32) -> Signal<Option<Message>> {
        let signal = create_signal(None);
        let messages_signal = self.messages;

        create_effect(move || {
            let messages = messages_signal.get_clone();
            let msg = messages.iter().find(|m| m.id == msg_id).cloned();
            signal.set(msg);
        });

        signal
    }

    // adds a new `Message` to the chatlog and generates a new id for it.
    // `ai_gen` should be set to `false` if this message was human generated.
    // `image_base64` is an optional base64 encoded string for an image.
    pub fn add_message(&mut self, new_msg: String, ai_gen: bool, image_base64: Option<String>) {
        self.messages.update(|msgs| {
            let new_id = self.get_next_id();
            msgs.push(Message {
                id: new_id,
                ai_generated: ai_gen,
                message_stack: vec![StackedMessage {
                    message: new_msg,
                    image_base64,
                }],
                selected_message: 0,
            })
        });
    }

    // pushes a new StackedMessage to the message stack for the specified message ID
    pub fn push_to_message_stack(
        &mut self,
        msg_id: u32,
        new_msg: String,
        image_base64: Option<String>,
    ) {
        self.messages.update(|msgs| {
            if let Some(msg) = msgs.iter_mut().find(|m| m.id == msg_id) {
                msg.message_stack.push(StackedMessage {
                    message: new_msg,
                    image_base64,
                });
                msg.selected_message = msg.message_stack.len() - 1;
            }
        });
    }

    // gets the message in the chat log for a given id and returns it or
    // `None` if the id isn't found.
    pub fn get_message(&self, id: u32) -> Option<Message> {
        if let Some(msg) = self.messages.get_clone().iter().find(|msg| msg.id == id) {
            Some(msg.clone())
        } else {
            None
        }
    }

    // removes the `Message` with the matching id *and* all `Message` objects that come after it.
    pub fn purge_messages(&mut self, id: u32) {
        self.messages.update(|msgs| {
            if let Some(index) = msgs.iter().position(|msg| msg.id == id) {
                msgs.truncate(index);
            }
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

    // updates the message text and image data for the currently selected `StackedMessage` in
    // the `Message` with a matching id.
    pub fn update_msg(&mut self, id: u32, new_msg: String, image_base64: Option<String>) {
        self.messages.update(|msgs| {
            if let Some(msg) = msgs.iter_mut().find(|msg| msg.id == id) {
                msg.set_selected_message(StackedMessage {
                    message: new_msg,
                    image_base64: image_base64,
                });
            }
        });
    }

    // updates the selected index, which represents the `StackedMessage` that should be presented
    // to the user by default for this `Message`.
    pub fn update_selected_index(&mut self, msg_id: u32, delta: i16) {
        self.messages.update(|msgs| {
            if let Some(m) = msgs.iter_mut().find(|m| m.id == msg_id) {
                let len = m.message_stack.len();
                if len == 0 {
                    return;
                }
                let current = m.selected_message as i16;
                let new_idx = (current + delta).clamp(0, len as i16 - 1) as usize;
                m.selected_message = new_idx;
            }
        })
    }

    // call this to invoke a text generation request to create a `Message` response.
    // the `response_generator` member must be set to the function implementing
    // the response generation.
    pub fn trigger_response_generation(&self) {
        (self.response_generator)();
    }

    // internal helper function to generate the next id for messages.
    fn get_next_id(&mut self) -> u32 {
        let id = self.next_id.get();
        self.next_id.set(id + 1);
        id
    }
}
