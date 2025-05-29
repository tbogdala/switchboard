use crate::{
    components::button_bar::ButtonBarComponent,
    models::{config::ApiEndpointConfig, dark_mode::DarkMode, system_message::SystemMessage},
};
use sycamore::prelude::*;

#[component(inline_props)]
pub fn ConfigInterface() -> View {
    let config_context_signal = use_context::<Signal<ApiEndpointConfig>>();

    // create signals for the input fields
    let config = config_context_signal.get_clone();
    let name = create_signal(config.name);
    let api_endpoint = create_signal(config.endpoint.clone());
    let model_id = create_signal(config.model_id.clone());
    let api_key = create_signal(config.api_key.clone());

    // create the signals for the advanced sampling parameters in the API configuration
    let show_advanced_settings = create_signal(false);
    let max_tokens = create_signal(config.max_tokens.clone().unwrap_or_default());
    let target_context_size = create_signal(config.target_context_size.clone().unwrap_or_default());
    let temp = create_signal(config.temperature.clone().unwrap_or_default());
    let top_p = create_signal(config.top_p.clone().unwrap_or_default());
    let top_k = create_signal(config.top_k.clone().unwrap_or_default());
    let min_p = create_signal(config.min_p.clone().unwrap_or_default());
    let repetition_penalty = create_signal(config.repetition_penalty.clone().unwrap_or_default());

    let toggle_advanced_settings = move || {
        show_advanced_settings.set(!show_advanced_settings.get());
    };

    // updates the context with a new config object on keypress
    let on_api_config_key = move |_| {
        let new_name = name.get_clone();
        let new_endpoint = api_endpoint.get_clone();
        let new_model_id = model_id.get_clone();
        let new_key = api_key.get_clone();
        let new_max_tokens = max_tokens.get_clone();
        let new_context_size = target_context_size.get_clone();
        let new_temp = temp.get_clone();
        let new_top_p = top_p.get_clone();
        let new_top_k = top_k.get_clone();
        let new_min_p = min_p.get_clone();
        let new_reppen = repetition_penalty.get_clone();
        let new_config = ApiEndpointConfig {
            name: new_name,
            endpoint: new_endpoint,
            model_id: new_model_id,
            api_key: new_key,
            temperature: if new_temp.is_empty() {
                None
            } else {
                Some(new_temp)
            },
            top_p: if new_top_p.is_empty() {
                None
            } else {
                Some(new_top_p)
            },
            top_k: if new_top_k.is_empty() {
                None
            } else {
                Some(new_top_k)
            },
            min_p: if new_min_p.is_empty() {
                None
            } else {
                Some(new_min_p)
            },
            repetition_penalty: if new_reppen.is_empty() {
                None
            } else {
                Some(new_reppen)
            },
            max_tokens: if new_max_tokens.is_empty() {
                None
            } else {
                Some(new_max_tokens)
            },
            target_context_size: if new_context_size.is_empty() {
                None
            } else {
                Some(new_context_size)
            },
        };
        config_context_signal.set(new_config);
    };

    // create a signal for the system message
    let system_message_context = use_context::<SystemMessage>();
    let system_message = create_signal(system_message_context.signal().get_clone());
    let system_message_signal = system_message_context.signal().clone();
    create_effect(move || {
        let new_sysmsg = system_message.get_clone();
        system_message_signal.set(new_sysmsg);
    });

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

    view! {
        div (class = get_chat_container_classes()) {
            ButtonBarComponent()

            div (class = "chat-window") {
                div(class="config-container") {
                    div(class = "mb-4") {
                        h3(class = "text-lg font-semibold text-primary-text dark:text-primary-text-dark mb-2") {
                            "API Configuration"
                        }
                        p(class = "text-sm text-secondary-text dark:text-secondary-text-dark") {
                            "Enter the details below to connect to an OpenAI compatible API endpoint. This is how the application sends and receives chat replies from the AI."
                        }
                    }

                    div(class = "config-group") {
                        span(class = "config-label") { "Name:"}
                        input(class="config-textinput", bind:value=name, on:input=on_api_config_key,
                            r#type = "text")
                    }
                    div(class = "config-group") {
                        span(class = "config-label") { "API Endpoint:"}
                        input(class="config-textinput", bind:value=api_endpoint, on:input=on_api_config_key,
                            r#type = "text")
                    }
                    div(class = "config-group") {
                        span(class = "config-label") { "API Key:"}
                        input(class="config-textinput", bind:value=api_key, on:input=on_api_config_key,
                            r#type = "text")
                    }
                    div(class = "config-group") {
                        span(class = "config-label") { "Model ID:"}
                        input(class="config-textinput", bind:value=model_id, on:input=on_api_config_key,
                            r#type = "text")
                    }

                    div{
                        div(class="mb-2 mt-2 flex items-center cursor-pointer", on:click=move |_| {
                            toggle_advanced_settings();
                        }) {
                            "Advanced Settings "
                            span(class="think-toggle") {
                                (if show_advanced_settings.get() { "▼" } else { "▶" })
                            }
                        }
                        div(class=if show_advanced_settings.get() { "think-block-content" } else { "hidden" }) {
                            p(class="text-secondary-text dark:text-secondary-text-dark mb-4") {
                                "Configure advanced sampling parameters for the AI model."
                            }

                            div(class="config-group") {
                                span(class="config-label") { "Temperature:" }
                                input(
                                    class="config-textinput",
                                    bind:value=temp,
                                    on:input=on_api_config_key,
                                    r#type="text",
                                    placeholder="1.0"
                                )
                            }

                            div(class="config-group") {
                                span(class="config-label") { "Top P:" }
                                input(
                                    class="config-textinput",
                                    bind:value=top_p,
                                    on:input=on_api_config_key,
                                    r#type="text",
                                    placeholder="1.0"
                                )
                            }

                            div(class="config-group") {
                                span(class="config-label") { "Top K:" }
                                input(
                                    class="config-textinput",
                                    bind:value=top_k,
                                    on:input=on_api_config_key,
                                    r#type="text",
                                    placeholder="0"
                                )
                            }

                            div(class="config-group") {
                                span(class="config-label") { "Min P:" }
                                input(
                                    class="config-textinput",
                                    bind:value=min_p,
                                    on:input=on_api_config_key,
                                    r#type="text",
                                    placeholder="0.0"
                                )
                            }

                            div(class="config-group") {
                                span(class="config-label") { "Repetition Pentalty:" }
                                input(
                                    class="config-textinput",
                                    bind:value=repetition_penalty,
                                    on:input=on_api_config_key,
                                    r#type="text",
                                    placeholder="1.0"
                                )
                            }

                            div(class="config-group") {
                                span(class="config-label") { "Max Tokens:" }
                                input(
                                    class="config-textinput",
                                    bind:value=max_tokens,
                                    on:input=on_api_config_key,
                                    r#type="text",
                                    placeholder="100"
                                )
                            }

                            div(class="config-group") {
                                span(class="config-label") { "Target Context Size:" }
                                input(
                                    class="config-textinput",
                                    bind:value=target_context_size,
                                    on:input=on_api_config_key,
                                    r#type="text",
                                    placeholder="4096"
                                )
                            }
                        }
                    }
                }

                div(class="config-container") {
                    div(class = "mb-4") {
                        h3(class = "text-lg font-semibold text-primary-text dark:text-primary-text-dark mb-2") {
                            "System Message"
                        }
                        p(class = "text-sm text-secondary-text dark:text-secondary-text-dark") {
                            "Enter any text you wish to provide the LLM with at the beginning of each conversation. Models have different conventions for what to put here."
                        }
                    }

                    div(class = "config-group") {
                        textarea(class="message-input", rows="8", r#type = "text", bind:value=system_message)
                    }
                }
            }
        }
    }
}
