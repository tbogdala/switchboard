use sycamore::{prelude::*, web::events::KeyboardEvent};

use crate::models::{chatlog::Chatlog, is_response_pending::IsResponsePending};

/// A component that renders a chat input field with a send button.
///
/// This component is designed for use in chat interfaces, allowing users to type messages
/// and send them. It consists of an input field where users can type their message,
/// and a button that can be clicked to send the message.
#[component(inline_props)]
pub fn ChatInputComponent() -> View {
    let is_response_pending = use_context::<IsResponsePending>();
    let input_text = create_signal(String::new());

    // shared send logic
    let send_message = move || {
        let input_str = input_text.get_clone_untracked();
        let active_chatlog = use_context::<Signal<Chatlog>>();
        if !input_str.trim().is_empty() {
            // we send a message if something was typed in, which will add
            // the message to the log as well as generate a new reply.
            let mut log = active_chatlog.get_clone();
            log.add_msg(input_str.clone(), false);
            log.trigger_response_generation();
            active_chatlog.replace(log);
            input_text.set(String::new());
        } else {
            // on an empty textarea, we just attempt to generate another reply.
            let log = active_chatlog.get_clone();
            log.trigger_response_generation();
        }
    };

    // send button click handler
    let on_send_press = move |_| {
        send_message();
    };

    // keydown handler for Enter key (&& !Shift)
    let on_keypress = move |event: KeyboardEvent| {
        if event.key() == "Enter" && !event.shift_key() {
            event.prevent_default();
            send_message();
        }
    };

    view! {
        div(class="input-container") {
            textarea(bind:value=input_text, on:keypress=on_keypress, class="message-input", rows="2",
                   r#type = "text", placeholder = "Type a message...") {}

            button(on:click=on_send_press, class="send-button", disabled=is_response_pending.signal().get()) {
                (if input_text.get_clone().trim().is_empty() {
                    "Continue"
                } else {
                    "Send"
                })
            }
        }
    }
}
