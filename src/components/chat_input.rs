use sycamore::{prelude::*, web::events::KeyboardEvent};
use web_sys::ClipboardEvent;
use web_sys::js_sys::Function;
use web_sys::wasm_bindgen::prelude::Closure;
use web_sys::{FileReader, wasm_bindgen::JsCast};

use crate::components::detect_mobile_device;
use crate::models::{chatlog::Chatlog, is_response_pending::IsResponsePending};

/// A component that renders a chat input field with a send button.
///
/// This component is designed for use in chat interfaces, allowing users to type messages
/// and send them. It consists of an input field where users can type their message,
/// and a button that can be clicked to send the message.
#[component(inline_props)]
pub fn ChatInputComponent() -> View {
    // context bool that should be set to true once a request is sent out to AI
    let is_response_pending = use_context::<IsResponsePending>();

    // signal for the main text input control
    let input_text = create_signal(String::new());

    // this stashes the image data, as a base64 encode, from a clipboard paste event
    let image_data_base64 = create_signal(None::<String>);

    // shared send logic to send a new message request
    let send_message = move || {
        // skip sending messages if we already have one in the oven
        if is_response_pending.signal().get() {
            return;
        }

        let input_str = input_text.get_clone_untracked();
        let active_chatlog = use_context::<Signal<Chatlog>>();
        if !input_str.trim().is_empty() {
            // we send a message if something was typed in, which will add
            // the message to the log as well as generate a new reply.
            let mut log = active_chatlog.get_clone();

            let image_data = image_data_base64.get_clone();
            log.add_msg(input_str.clone(), false, image_data);
            log.trigger_response_generation();
            active_chatlog.replace(log);

            input_text.set(String::new());
            image_data_base64.set(None::<String>);
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

    // keydown handler for Enter key (&& !Shift) on non-mobile devices.
    let on_keypress = move |event: KeyboardEvent| {
        if !detect_mobile_device() {
            if event.key() == "Enter" && !event.shift_key() {
                event.prevent_default();
                send_message();
            }
        }
    };

    // intercept clipboard paste events to check and see if an image is being pasted into
    // the message textarea. If so, we update our signal with the base64 encoded data.
    let on_paste = move |event: web_sys::Event| {
        let clipboard_event: ClipboardEvent = event.dyn_into().unwrap();
        if let Some(data_transfer) = clipboard_event.clipboard_data() {
            let items = data_transfer.items();

            for i in 0..items.length() {
                if let Some(item) = items.get(i) {
                    //console_log!("Item [{}]: kind={}, type={}", i, item.kind(), item.type_());
                    if item.kind() == "file" && item.type_().starts_with("image/") {
                        if let Some(file) = item.get_as_file().unwrap() {
                            clipboard_event.prevent_default();

                            let file_reader_maybe = FileReader::new();
                            match file_reader_maybe {
                                Ok(file_reader) => {
                                    let file_reader_clone = file_reader.clone();
                                    let image_data_base64_clone = image_data_base64.clone();
                                    let handle_onload_js: Function =
                                        Closure::wrap(Box::new(move || {
                                            match file_reader_clone.result() {
                                                Ok(res) => {
                                                    if let Some(data_str) = res.as_string() {
                                                        image_data_base64_clone.set(Some(data_str));
                                                    } else {
                                                        console_log!(
                                                            "Failed to read image as string."
                                                        );
                                                    }
                                                }
                                                Err(err) => {
                                                    console_log!(
                                                        "Error reading the image: {:?}",
                                                        err
                                                    );
                                                }
                                            }
                                        })
                                            as Box<dyn FnMut()>)
                                        .into_js_value()
                                        .into();

                                    file_reader.set_onload(Some(&handle_onload_js));
                                    if let Err(err) = file_reader.read_as_data_url(&file) {
                                        console_log!("Clipboard image reading failed: {:?}", err);
                                    }
                                }
                                Err(err) => {
                                    console_log!("FileReader failed to start: {:?}", err);
                                }
                            }
                        }

                        break; // only handle the first image found
                    }
                }
            }
        }
    };

    view! {
        div(class="input-container") {
            (if let Some(data_url_str) = image_data_base64.get_clone() {
                view! {
                    div(class="flex flex-col items-center") {
                        h3(class="input-image-label") { "Attached Image:" }
                        img(src=data_url_str, alt="Pasted Image", class="input-image")
                    }
                }
            } else {
                view! { }
            })

            textarea(bind:value=input_text, on:keypress=on_keypress, on:paste=on_paste,
                class="message-input", rows="2", r#type = "text", placeholder = "Type a message...") {}

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
