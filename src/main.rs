use components::{
    chat_interface::ChatInterface, config_interface::ConfigInterface,
    saving_interface::SavingInterface,
};
use models::{
    chatlog::Chatlog, config::ApiEndpointConfig, dark_mode::DarkMode,
    is_editing_config::IsEditingConfig, is_response_pending::IsResponsePending,
    is_saving_chatlog::IsSavingChatlog, system_message::SystemMessage,
};
use sycamore::prelude::*;

use crate::models::chatlog::ChatLogMetadata;

pub mod api_endpoint;
pub mod components;
pub mod models;
pub mod storage;

// keys used in local storage
const LSKEY_API_CONFIG: &str = "api_config";
const LSKEY_SYSMSG: &str = "system_message";
const LSKEY_CURRENTLOG: &str = "current_chatlog";
const LSKEY_DARK_MODE: &str = "dark_mode";
const LSKEY_CHATLOG_METADATA: &str = "chatlog_metadata";

// This function is used as a callback for the Chatlog for when an AI response is
// requested.
fn generate_response() {
    let is_response_pending = use_context::<IsResponsePending>();
    is_response_pending.signal().set(true);

    let active_chatlog = use_context::<Signal<Chatlog>>();
    let log = active_chatlog.get_clone_untracked();
    let msgs = log.messages.get_clone_untracked();

    let config_context_signal = use_context::<Signal<ApiEndpointConfig>>();
    let system_message_context = use_context::<SystemMessage>();

    api_endpoint::send_chat_completion_request(msgs, move |maybe_response| {
        // console_log!("main::on_user_send response received: {:?}", response);
        is_response_pending.signal().set(false);
        match maybe_response {
            Ok(response) => {
                active_chatlog.update(|log| log.add_msg(response.text, true, None));

                // save the active chatlog into a separate local storage key so that
                // current progress is always saved.
                let chatlog_json = active_chatlog.get_clone_untracked().to_json(
                    config_context_signal.get_clone(),
                    system_message_context.signal().get_clone(),
                );
                if let Ok(json) = chatlog_json {
                    if let Err(e) =
                        storage::save_to_local_storage::<String>(LSKEY_CURRENTLOG, &json)
                    {
                        console_log!(
                            "ERROR: attempt to save_to_local_storage for current log failed: {:?}",
                            e
                        );
                    }
                } else {
                    console_log!("Failed to serialize the current chatlog to JSON.");
                }
            }
            Err(e) => {
                let _ = window().alert_with_message(
                    format!("ERROR: Failed to generate the AI's response:\n\n{}", e).as_str(),
                );
            }
        };
    });
}

/// A component that renders the application.
#[component]
fn MainComponent() -> View {
    // create a signal for the chatlog metadata and put it in the context
    let metadata = storage::load_from_local_storage::<ChatLogMetadata>(LSKEY_CHATLOG_METADATA)
        .unwrap_or_else(ChatLogMetadata::new);
    let chatlog_metadata = create_signal(metadata);
    provide_context(chatlog_metadata);

    // add effect to save metadata on change
    let chatlog_metadata_clone = chatlog_metadata.clone();
    create_effect(move || {
        let metadata = chatlog_metadata_clone.get_clone();
        if let Err(e) =
            storage::save_to_local_storage::<ChatLogMetadata>(LSKEY_CHATLOG_METADATA, &metadata)
        {
            console_log!("Failed to save chatlog metadata: {:?}", e);
        }
    });

    // create a signal for the chatlog and put it in the context
    let chatlog_json_maybe = storage::load_from_local_storage::<String>(LSKEY_CURRENTLOG);
    let active_chatlog = match chatlog_json_maybe {
        Some(json) => match Chatlog::from_json(&json, generate_response) {
            Ok((log, _, _)) => create_signal(log),
            Err(_) => create_signal(Chatlog::new(generate_response)),
        },
        None => create_signal(Chatlog::new(generate_response)),
    };
    provide_context(active_chatlog);

    // setup the light and dark mode switching signal. we pull the initial value
    // from local storage and setup an effect to send it to loca storage on change.
    let dark_mode_init =
        storage::load_from_local_storage::<bool>(LSKEY_DARK_MODE).unwrap_or_else(|| true);
    let dark_mode = DarkMode::new(dark_mode_init);
    provide_context(dark_mode);
    let dark_mode_clone = dark_mode.signal().clone();
    create_effect(move || {
        let is_dark_mode = dark_mode_clone.get();
        if let Err(e) = storage::save_to_local_storage::<bool>(LSKEY_DARK_MODE, &is_dark_mode) {
            console_log!("save_to_local_storage error for dark mode: {:?}", e);
        }
    });

    // setup the switch to show the save file interface
    let saving_chatlog = IsSavingChatlog::new(false);
    provide_context(saving_chatlog);
    let is_saving_chatlog = saving_chatlog.signal();

    // setup the switch to show the configuration interface
    let editing_config = IsEditingConfig::new(false);
    provide_context(editing_config);
    // get the internal signal for the editing context.
    let is_editing_config = editing_config.signal();

    // Create a signal for the API configuration to use in the context
    // as well as an effect to save it to storage on change.
    let api_config = create_signal({
        storage::load_from_local_storage::<ApiEndpointConfig>(LSKEY_API_CONFIG)
            .unwrap_or_else(|| ApiEndpointConfig::default())
    });
    provide_context(api_config);
    create_effect(move || {
        let new_config = api_config.get_clone();
        if let Err(e) =
            storage::save_to_local_storage::<ApiEndpointConfig>(LSKEY_API_CONFIG, &new_config)
        {
            console_log!("save_to_local_storage error for api endpoint: {:?}", e);
        }
    });

    // Create a signal for the System Message to use in the context
    // as well as an effect to save it to storage on change.
    let initial_system_msg = storage::load_from_local_storage::<String>(LSKEY_SYSMSG)
        .unwrap_or_else(|| String::default());
    let system_msg = SystemMessage::new(initial_system_msg);
    provide_context(system_msg);
    create_effect(move || {
        let new_prompt = system_msg.signal().get_clone();
        if let Err(e) = storage::save_to_local_storage::<String>(LSKEY_SYSMSG, &new_prompt) {
            console_log!("save_to_local_storage error for system message: {:?}", e);
        }
    });

    // Create a signal to indicate if an API response is pending and make it available
    // to the context
    let response_pending = IsResponsePending::new(false);
    provide_context(response_pending);

    view! {
        (if is_editing_config.get() {
            view! { ConfigInterface() }
        } else if is_saving_chatlog.get() {
            view! { SavingInterface() }
        } else {
            view! { ChatInterface() }
        })
    }
}

fn main() {
    console_error_panic_hook::set_once();
    sycamore::render(|| view! { MainComponent() });
}
