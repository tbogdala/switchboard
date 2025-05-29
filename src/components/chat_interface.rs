use crate::{
    components::{
        button_bar::ButtonBarComponent, chat_input::ChatInputComponent, chatlog::ChatlogComponent,
    },
    models::{chatlog::Chatlog, dark_mode::DarkMode, is_response_pending::IsResponsePending},
};
use sycamore::{prelude::*, web::rt::web_sys::HtmlElement};
use wasm_bindgen_futures::wasm_bindgen::JsCast;

/// Renders a chat interface component that displays a list of messages and an input field for new messages.
#[component(inline_props)]
pub fn ChatInterface() -> View {
    let is_response_pending = use_context::<IsResponsePending>();

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

    // creates a reference to our chat-messages container
    let node_ref = create_node_ref();

    // create an effect to scroll to the end every time the chatlog changes.
    let active_chatlog = use_context::<Signal<Chatlog>>();
    create_effect(move || {
        let _ = is_response_pending.signal().get(); // Track changes to the response pending (for progress spinner)
        let _ = active_chatlog.get_clone(); // Track changes to the chatlog
        on_mount(move || {
            let node = node_ref.get();
            if let Ok(elem) = node.dyn_into::<HtmlElement>() {
                let scroll_height = elem.scroll_height();
                let client_height = elem.client_height();

                if scroll_height > client_height {
                    elem.set_scroll_top(scroll_height - client_height);
                }
            }
        });
    });

    view! {
        div (class = get_chat_container_classes()) {
            ButtonBarComponent()

            div (class = "chat-window") {
                div (class = "chat-messages", r#ref=node_ref) {
                    ChatlogComponent()

                    (if is_response_pending.signal().get() {
                        view! {
                            div(class="flex justify-center items-center p-4") {
                                div(class="progress-spinner") {}
                            }
                        }
                    } else {
                        view! { }
                    })
                }

                ChatInputComponent()
            }
        }
    }
}
