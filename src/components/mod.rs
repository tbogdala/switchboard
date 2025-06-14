pub mod button_bar;
pub mod chat_input;
pub mod chat_interface;
pub mod chat_message;
pub mod chatlog;
pub mod config_interface;
pub mod saving_interface;

// Helper function to detect mobile devices, based on user agent heuristics
fn detect_mobile_device() -> bool {
    if let Some(win) = web_sys::window() {
        let navigator = win.navigator();
        if let Ok(user_agent) = navigator.user_agent() {
            let ua_lower = user_agent.to_lowercase();
            return ua_lower.contains("android")
                || ua_lower.contains("iphone")
                || ua_lower.contains("ipad")
                || ua_lower.contains("mobile")
                || ua_lower.contains("windows phone");
        }
    }

    false
}
