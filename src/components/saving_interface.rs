use crate::{
    components::button_bar::ButtonBarComponent,
    models::{
        chatlog::Chatlog, config::ApiEndpointConfig, dark_mode::DarkMode,
        system_message::SystemMessage,
    },
};
use sycamore::prelude::*;

// returns true if the save slot exists in the web browser local storage
fn does_save_slot_exist(key: &str) -> bool {
    let window = window();
    if let Ok(storage) = window.local_storage() {
        if let Some(storage) = storage {
            if let Ok(value) = storage.get_item(key) {
                return value.is_some();
            }
        }
    }

    false
}

#[component()]
pub fn SaveSlotComponent(index: u16) -> View {
    let key = format!("save_slot_{}", index);

    let is_saved_signal = create_signal(does_save_slot_exist(&key));

    let key_clone = key.clone();
    let handle_save = move |_| {
        let config_context_signal = use_context::<Signal<ApiEndpointConfig>>();
        let api_config = config_context_signal.get_clone();
        let system_message_context = use_context::<SystemMessage>();
        let system_message = system_message_context.signal().get_clone();
        let active_chatlog = use_context::<Signal<Chatlog>>();
        let log = active_chatlog.get_clone();

        let maybe_json_str = log.to_json(api_config, system_message);
        if let Ok(json_str) = maybe_json_str {
            // console_log!("JSON:\n{:?}", json_str);

            if let Ok(confirmed) = window().confirm_with_message(&format!(
                "OVERWRITE save slot {}? This will erase any existing chatlog permanently!",
                index
            )) {
                if confirmed {
                    match crate::storage::save_to_local_storage::<String>(&key_clone, &json_str) {
                        Ok(_) => {
                            console_log!("Saved chatlog to {}", key_clone);
                            is_saved_signal.set(true);
                            let _ = window().alert_with_message(
                                format!("Chatlog saved successfully to slot {}.", index).as_str(),
                            );
                        }
                        Err(e) => {
                            console_log!("save_to_local_storage error: {:?}", e);
                            let _ = window().alert_with_message(
                                format!("ERROR: unable to save the chatlog in slot {}!", index)
                                    .as_str(),
                            );
                        }
                    };
                }
            }
        }
    };

    let key_clone = key.clone();
    let handle_load = move |_| {
        if let Ok(confirmed) = window().confirm_with_message(&format!(
            "Load save slot {}? This will OVERWRITE the current chatlog permanently!",
            index
        )) {
            if confirmed {
                if let Some(json_str) =
                    crate::storage::load_from_local_storage::<String>(&key_clone)
                {
                    let maybe_fullcycle = Chatlog::from_json(&json_str, crate::generate_response);
                    if let Ok((new_log, new_api, new_sysmsg)) = maybe_fullcycle {
                        let config_context_signal = use_context::<Signal<ApiEndpointConfig>>();
                        config_context_signal.set(new_api);

                        let system_message_context = use_context::<SystemMessage>();
                        system_message_context.signal().set(new_sysmsg);

                        let active_chatlog = use_context::<Signal<Chatlog>>();
                        active_chatlog.update(|log| {
                            log.clone_from(&new_log);
                        });

                        let _ = window().alert_with_message(
                            format!("Chatlog successfully loaded from slot {}.", index).as_str(),
                        );
                        return;
                    }
                }
                let _ = window().alert_with_message(
                    format!("ERROR: unable to load chatlog from slot {}.", index).as_str(),
                );
            }
        }
    };

    let key_clone = key.clone();
    let handle_clear = move |_| {
        if let Ok(confirmed) =
            window().confirm_with_message(&format!("Clear save slot {} permanently?", index))
        {
            if confirmed {
                if let Err(e) = crate::storage::delete_from_local_storage(&key_clone) {
                    console_log!("delete_from_local_storage error: {:?}", e);
                    let _ = window().alert_with_message(
                        format!("ERROR: unable to clear the chatlog for slot {}!", index).as_str(),
                    );
                } else {
                    is_saved_signal.set(false);
                    let _ = window().alert_with_message(
                        format!("Chatlog deleted successfully from slot {}.", index).as_str(),
                    );
                }
            }
        }
    };

    view! {
        div(class="config-container") {
            div(class="mb-4 flex items-center") {
                span(class="save-slot-badge") {
                    (index)
                }
                p {
                    (if is_saved_signal.get() {
                        "CHATLOG SAVED"
                    } else {
                        "EMPTY SAVE SLOT"
                    })
                }
            }
            div(class="save-slot-container") {
                button(on:click=handle_save, r#type="button", class="save-slot-button") { "Save" }
                button(on:click=handle_load, r#type="button", class="save-slot-button") { "Load" }
                button(on:click=handle_clear, r#type="button", class="save-slot-button") { "Clear" }
            }
        }
    }
}

#[component(inline_props)]
pub fn SavingInterface() -> View {
    const NUM_SAVE_SLOTS: u16 = 10;

    // this is bound to the textarea for JSON importing
    let imported_json = create_signal(String::new());
    let handle_import_json = move |_| {
        let json_str = imported_json.get_clone();
        let maybe_fullcycle = Chatlog::from_json(&json_str, crate::generate_response);
        if let Ok((new_log, new_api, new_sysmsg)) = maybe_fullcycle {
            let config_context_signal = use_context::<Signal<ApiEndpointConfig>>();
            config_context_signal.set(new_api);

            let system_message_context = use_context::<SystemMessage>();
            system_message_context.signal().set(new_sysmsg);

            let active_chatlog = use_context::<Signal<Chatlog>>();
            active_chatlog.update(|log| {
                log.clone_from(&new_log);
            });

            let _ = window().alert_with_message(format!("Chatlog successfully imported.").as_str());
        }
    };

    let handle_export_json = move |_| {
        let config_context_signal = use_context::<Signal<ApiEndpointConfig>>();
        let api_config = config_context_signal.get_clone();
        let system_message_context = use_context::<SystemMessage>();
        let system_message = system_message_context.signal().get_clone();
        let active_chatlog = use_context::<Signal<Chatlog>>();
        let log = active_chatlog.get_clone();

        let maybe_json_str = log.to_json(api_config, system_message);
        if let Ok(json_str) = maybe_json_str {
            imported_json.set(json_str);
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

    let save_slots: Vec<_> = (1..=NUM_SAVE_SLOTS).map(|i| SaveSlotComponent(i)).collect();

    view! {
        div (class = get_chat_container_classes()) {
            ButtonBarComponent()

            div (class = "chat-window") {
                div(class="config-container") {
                    div(class = "mb-4") {
                        h3(class = "save-slot-header") {
                            "Save Slots"
                        }
                        p(class = "save-slot-header-secondary") {
                            "The current chatlog can be saved to, or loaded from, these save slots."
                        }
                    }
                }

                ul {
                    (save_slots)
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
