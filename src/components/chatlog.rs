use crate::{components::chat_message::ChatMessageComponent, models::chatlog::Chatlog};
use sycamore::prelude::*;

/// A component that displays a list of chat messages.
#[component(inline_props)]
pub fn ChatlogComponent() -> View {
    let active_chatlog = use_context::<Signal<Chatlog>>();
    let msgs = create_signal(active_chatlog.get_clone().messages);
    view! {
        ul {
            Keyed(
                list=msgs,
                view=move |msg| view! {
                    ChatMessageComponent(msg=msg)
                },
                key=|msg| msg.id,
            )
        }
    }
}
