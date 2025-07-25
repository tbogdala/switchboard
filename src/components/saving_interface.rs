use crate::{
    components::button_bar::ButtonBarComponent,
    models::{
        chatlog::{ChatLogMetadata, ChatLogMetadataEntry, Chatlog},
        config::ApiEndpointConfig,
        dark_mode::DarkMode,
        system_message::SystemMessage,
    },
    storage,
};
use sycamore::prelude::*;
use web_sys::KeyboardEvent;

// helper function to serialize the chatlog to JSON or return an error message.
fn serialize_chatlog(log: &Chatlog) -> Result<String, String> {
    let api = use_context::<Signal<ApiEndpointConfig>>().get_clone();
    let sys = use_context::<SystemMessage>().signal().get_clone();
    log.to_json(api, sys)
        .map_err(|e| format!("Serialization failed: {e}"))
}

// serialize the chatlog and save it to the browser's local storage using the key specific
// for this chatlog that is derived from the chatlog's `id` as well as updates
// the metadata for the chatlog. the user is asked to confirm this operation and
// notified of the results.
fn save_chatlog(
    log: &Chatlog,
    key: &str,
    entry_title: &str,
    entry_id: &str,
    metadata: &Signal<ChatLogMetadata>,
) {
    let maybe_json_str = serialize_chatlog(&log);
    if let Ok(json_str) = maybe_json_str {
        // console_log!("JSON:\n{:?}", json_str);

        if let Ok(confirmed) = window().confirm_with_message(&format!(
            "Save chatlog '{}'? This will overwrite the old chatlog permanently!",
            entry_title
        )) {
            if confirmed {
                match crate::storage::save_to_local_storage::<String>(key, &json_str) {
                    Ok(_) => {
                        metadata.update(|meta| {
                            if let Some(entry) =
                                meta.saved_logs.iter_mut().find(|e| e.id == entry_id)
                            {
                                entry.message_count = log.messages.get_clone().len();
                                entry.last_accessed_time =
                                    web_sys::js_sys::Date::now().round() as i64;
                            }
                        });

                        let _ = window().alert_with_message(
                            format!("Chatlog saved successfully ('{}').", entry_title).as_str(),
                        );
                    }
                    Err(e) => {
                        console_log!("save_to_local_storage error: {:?}", e);
                        let _ = window().alert_with_message(
                            format!("ERROR: unable to save the chatlog ('{}')!", entry_title)
                                .as_str(),
                        );
                    }
                };
            }
        }
    }
}

// loads the chatlog from the browser's local storage under the key associated
// with this chatlog in the metadata, deserializes it and then sets the relevant
// data: the chatlog, API settings and system message. The metadata last accessed time
// is updated and the user is queried to confirm the loading operation and is notified
// of the results.
fn load_chatlog(key: &str, entry_title: &str, entry_id: &str) {
    let confirmed = match window().confirm_with_message(&format!("Load chatlog '{}'?", entry_title))
    {
        Ok(c) => c,
        Err(_) => return,
    };
    if !confirmed {
        return;
    }

    let Some(json_str) = storage::load_from_local_storage::<String>(key) else {
        let _ = window().alert_with_message(&format!(
            "ERROR: unable to load chatlog ('{}').",
            entry_title
        ));
        return;
    };

    match Chatlog::from_json(&json_str, crate::generate_response) {
        Ok((new_log, new_api, new_sysmsg)) => {
            use_context::<Signal<ApiEndpointConfig>>().set(new_api);
            use_context::<SystemMessage>().signal().set(new_sysmsg);

            use_context::<Signal<Chatlog>>().update(|log| log.clone_from(&new_log));

            use_context::<Signal<ChatLogMetadata>>().update(|meta| {
                if let Some(entry) = meta.saved_logs.iter_mut().find(|e| e.id == entry_id) {
                    entry.message_count = new_log.messages.get_clone().len();
                    entry.last_accessed_time = web_sys::js_sys::Date::now().round() as i64;
                }
            });

            let _ = window()
                .alert_with_message(&format!("Chatlog successfully loaded ('{}').", entry_title));
        }
        Err(e) => {
            let _ = window().alert_with_message(&format!("Chatlog could not be loaded: {}", e));
        }
    }
}

// deletes the key associated with the chatlog from the metadata in the browser's
// local storage. the user is queried to confirm the deletion and is notified of
// the results.
fn delete_chatlog(key: &str, entry_title: &str, entry_id: &str) {
    let confirmed = match window()
        .confirm_with_message(&format!("Delete chatlog '{}' permanently?", entry_title))
    {
        Ok(c) => c,
        Err(_) => return,
    };
    if !confirmed {
        return;
    }

    match storage::delete_from_local_storage(key) {
        Ok(()) => {
            use_context::<Signal<ChatLogMetadata>>().update(|meta| {
                if let Some(idx) = meta.saved_logs.iter().position(|e| e.id == entry_id) {
                    meta.saved_logs.remove(idx);
                }
            });
            let _ = window().alert_with_message(&format!(
                "Chatlog deleted successfully ('{}').",
                entry_title
            ));
        }
        Err(e) => {
            console_log!("delete_from_local_storage error: {:?}", e);
            let _ = window().alert_with_message(&format!(
                "ERROR: unable to delete the chatlog ('{}')!",
                entry_title
            ));
        }
    }
}

// Component to display and manage a single chat log entry
#[component]
fn ChatLogEntryComponent(entry: ChatLogMetadataEntry) -> View {
    let metadata_signal = use_context::<Signal<ChatLogMetadata>>();
    let is_editing = create_signal(false);
    let edited_title = create_signal(entry.title.clone());
    let entry_id = entry.id.clone();
    let key = format!("chatlog_{}", entry_id);

    let key_clone = key.clone();
    let entry_id_clone = entry_id.clone();
    let entry_title_clone = entry.title.clone();
    let handle_save = move |_| {
        let log = use_context::<Signal<Chatlog>>().get_clone();
        let metadata = use_context::<Signal<ChatLogMetadata>>();
        save_chatlog(
            &log,
            &key_clone,
            &entry_title_clone,
            &entry_id_clone,
            &metadata,
        );
    };

    let key_clone = key.clone();
    let entry_id_clone = entry_id.clone();
    let entry_title_clone = entry.title.clone();
    let handle_load = move |_| load_chatlog(&key_clone, &entry_title_clone, &entry_id_clone);

    let key_clone = key.clone();
    let entry_id_clone = entry_id.clone();
    let entry_title_clone = entry.title.clone();
    let handle_delete = move |_| delete_chatlog(&key_clone, &entry_title_clone, &entry_id_clone);

    let entry_id = entry.id.clone();
    view! {
        div(class="config-container") {
            div(class="flex items-center w-full") {
                div(class="flex-1") {
                    p {
                        (if is_editing.get() {
                            view! {
                                input(
                                    bind:value=edited_title,
                                    on:blur={
                                        is_editing.set(false);
                                        let entry_id = entry_id.clone();
                                        move |_| {
                                        metadata_signal.update(|meta| {
                                            if let Some(entry) = meta.saved_logs.iter_mut().find(|e| e.id == entry_id) {
                                                entry.title = edited_title.get_clone();
                                            }
                                        });
                                    }},
                                    on:keypress={
                                        let entry_id = entry_id.clone();
                                        move |event: KeyboardEvent| {
                                        if event.key() == "Enter" {
                                            is_editing.set(false);
                                            metadata_signal.update(|meta| {
                                                if let Some(entry) = meta.saved_logs.iter_mut().find(|e| e.id == entry_id) {
                                                    entry.title = edited_title.get_clone();
                                                }
                                            });
                                        }
                                    }},
                                    r#type="text",
                                )
                            }
                        } else {
                            view! {
                                p(
                                    on:click=move |_| {
                                        is_editing.set(true);
                                    },
                                ) {
                                    (edited_title.get_clone())
                                }
                            }
                        })
                    }
                    p {
                        (format!("{} messages", entry.message_count))
                    }
                }

                div(class="flex-none ml-2") {
                    button(on:click=handle_save, class="save-slot-button") { "Save" }
                }
                div(class="flex-none ml-2") {
                    button(on:click=handle_load, class="save-slot-button") { "Load" }
                }
                div(class="flex-none ml-2") {
                    button(on:click=handle_delete, class="save-slot-button") { "Delete" }
                }
            }
        }
    }
}

// takes the json string passed in, deserializes it to and then
// sets the API endpoint config, system message and active chatlog with the
// deserialized log. returns an error message on failure.
fn import_json(json_str: String) -> Result<(), String> {
    let maybe_fullcycle = Chatlog::from_json(&json_str, crate::generate_response);
    match maybe_fullcycle {
        Ok((new_log, new_api, new_sysmsg)) => {
            use_context::<Signal<ApiEndpointConfig>>().set(new_api);
            use_context::<SystemMessage>().signal().set(new_sysmsg);
            use_context::<Signal<Chatlog>>().update(|log| log.clone_from(&new_log));
            window()
                .alert_with_message("Chatlog successfully imported.")
                .ok();
            Ok(())
        }
        Err(e) => Err(format!("Failed to import JSON: {e}")),
    }
}

// serilizes the active chatlog to a string, returning an error message on failure.
fn export_json() -> Result<String, String> {
    let log = use_context::<Signal<Chatlog>>().get_clone();
    serialize_chatlog(&log)
}

// prompts the user for a new chatlog name and then serializes the active chatlog
// into a new chatlog with the new name and adds it to the list of chatlogs
// saved in the local storage of the browser.
fn save_as() -> Result<(), String> {
    let log = use_context::<Signal<Chatlog>>().get_clone();
    let json_str = serialize_chatlog(&log)?;

    let Some(title) = window()
        .prompt_with_message("Enter a new name for the chatlog:")
        .ok()
        .flatten()
        .filter(|s| !s.trim().is_empty())
    else {
        return Ok(()); // user cancelled or empty
    };

    let new_log_id = storage::generate_chatlog_id();
    let new_storage_key = format!("chatlog_{}", new_log_id);
    storage::save_to_local_storage::<String>(&new_storage_key, &json_str)
        .map_err(|js| js.as_string().unwrap_or_else(|| "JS storage error".into()))?;

    use_context::<Signal<ChatLogMetadata>>().update(|meta| {
        meta.saved_logs.push(ChatLogMetadataEntry {
            id: new_log_id,
            title: title.clone(),
            last_accessed_time: web_sys::js_sys::Date::now().round() as i64,
            storage_key: new_storage_key,
            message_count: log.messages.get_clone_untracked().len(),
        });
    });

    window()
        .alert_with_message(&format!("Chatlog '{}' saved successfully.", title))
        .ok();
    Ok(())
}

// creates a brand new chatlog, serializes and saves it to the list of
// chatlogs saved in the local storage of the browser.
fn new_chatlog() -> Result<(), String> {
    let log = Chatlog::new(crate::generate_response);
    let json_str = serialize_chatlog(&log)?;

    let log_id = storage::generate_chatlog_id();
    let key = format!("chatlog_{}", log_id);
    storage::save_to_local_storage::<String>(&key, &json_str)
        .map_err(|js| js.as_string().unwrap_or_else(|| "JS storage error".into()))?;

    let timestamp = web_sys::js_sys::Date::now().round() as i64;
    use_context::<Signal<ChatLogMetadata>>().update(|meta| {
        meta.saved_logs.push(ChatLogMetadataEntry {
            id: log_id,
            title: "New Chatlog".to_string(),
            last_accessed_time: timestamp,
            storage_key: key,
            message_count: 0,
        });
    });
    Ok(())
}

#[component(inline_props)]
pub fn SavingInterface() -> View {
    // this is bound to the textarea for JSON importing
    let imported_json = create_signal(String::new());
    let handle_import_json = move |_| {
        let json_str = imported_json.get_clone();
        if let Err(e) = import_json(json_str) {
            let _ = window().alert_with_message(&e);
        }
    };

    let handle_export_json = move |_| match export_json() {
        Ok(json_str) => imported_json.set(json_str),
        Err(e) => {
            let _ = window().alert_with_message(&format!(
                "ERROR: failed to export the active chatlog to JSON: {}",
                e
            ));
        }
    };

    let handle_save_as = move |_| {
        if let Err(e) = save_as() {
            let _ = window().alert_with_message(&format!(
                "ERROR: unable to save the chatlog as a new log: {}",
                e
            ));
        }
    };

    let handle_new_chatlog = move |_| {
        if let Err(e) = new_chatlog() {
            let _ = window().alert_with_message(&format!("Failed to create a new chatlog: {}", e));
        }
    };

    // handle dark mode by applying the class to the overall container
    let dark_mode = use_context::<DarkMode>();
    let get_chat_container_classes = move || {
        format!(
            "chat-container {}",
            if dark_mode.is_dark_mode() {
                "dark"
            } else {
                "light"
            }
        )
    };

    let entries_components = move || {
        let metadata_signal = use_context::<Signal<ChatLogMetadata>>();
        let current_metadata = metadata_signal.get_clone();
        let mut sorted_entries = current_metadata.saved_logs.clone();
        // Sort in descending order by timestamp
        sorted_entries.sort_by(|a, b| b.last_accessed_time.cmp(&a.last_accessed_time));
        sorted_entries
            .iter()
            .map(|entry| {
                let cloned = entry.clone();
                ChatLogEntryComponent(cloned)
            })
            .collect::<Vec<View>>()
    };

    // add effect to save metadata on change
    let metadata_clone = use_context::<Signal<ChatLogMetadata>>().clone();
    create_effect(move || {
        let metadata = metadata_clone.get_clone();
        if let Err(e) = storage::save_to_local_storage::<ChatLogMetadata>(
            crate::LSKEY_CHATLOG_METADATA,
            &metadata,
        ) {
            console_log!("Failed to save chatlog metadata: {:?}", e);
        }
    });

    view! {
        div (class = get_chat_container_classes()) {
            ButtonBarComponent()

            div (class = "chat-window") {
                div(class="flex items-center config-container") {
                    div(class = "flex-1 mb-4") {
                        h3(class = "save-slot-header") {
                            "Chatlogs"
                        }
                        p(class = "save-slot-header-secondary") {
                            "This is the list of currently saved chatlogs in your web browser's local storage."
                        }
                    }
                    div(class="flex-none ml-2") {
                        button(on:click=handle_new_chatlog, class="save-slot-button") { "New" }
                    }
                    div(class="flex-none ml-2") {
                        button(on:click=handle_save_as, class="save-slot-button") { "Save As" }
                    }
                }

                ul {
                    (entries_components)
                }

                div(class="config-container mt-4") {
                    div(class="mb-4") {
                        h3(class="save-slot-header") {
                            "Import/Export"
                        }
                        p(class="save-slot-header-secondary") {
                            "Import or export entire chatlogs as JSON"
                        }
                    }

                    div(class="w-full") {
                        textarea(
                            bind:value=imported_json,
                            class="w-full p-2 border rounded mt-2",
                            placeholder="Paste JSON here for importing..."
                        )
                    }

                    div(class="flex justify-center space-x-2 mt-2") {
                        button(on:click=handle_import_json, r#type="button", class="save-slot-button") { "Import" }
                        button(on:click=handle_export_json, r#type="button", class="save-slot-button") { "Export" }
                    }
                }
            }
        }
    }
}
