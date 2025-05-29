use sycamore::prelude::*;

use crate::models::{
    dark_mode::DarkMode, is_editing_config::IsEditingConfig, is_saving_chatlog::IsSavingChatlog,
};

#[component(inline_props)]
pub fn ButtonBarComponent() -> View {
    let dark_mode = use_context::<DarkMode>();
    let toggle_dark_mode = move |_| {
        dark_mode.toggle();
    };

    let is_editing_config = use_context::<IsEditingConfig>();
    let is_saving_chatlog = use_context::<IsSavingChatlog>();

    let toggle_edit_config = move |_| {
        is_saving_chatlog.signal().set(false);
        is_editing_config.toggle();
    };

    let toggle_saving_chatlog = move |_| {
        is_editing_config.signal().set(false);
        is_saving_chatlog.toggle();
    };

    // Get the git hash from the environment variable
    let git_hash = env!("GIT_HASH");
    let git_branch = env!("GIT_BRANCH");
    let git_info_string = format!("[{}@{}]", git_branch, git_hash);

    view! {
        div (class="button-bar") {
            div {
                div (class="text-logo") { "Switchboard!" }
                div (class="git-hash") { (git_info_string) }
            }

            div (class="button-group") {
                button(on:click = toggle_saving_chatlog, class="config-button") {
                    i(class="lni lni-box-archive-1") {}
                }

                button(on:click = toggle_dark_mode, class="config-button") {
                    i(class="lni lni-bulb-4") {}
                }

                button(on:click = toggle_edit_config, class="config-button") {
                    i(class="lni lni-gears-3") {}
                }
            }
        }
    }
}
