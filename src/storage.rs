use serde::{Deserialize, Serialize};
use sycamore::prelude::*;
use web_sys::wasm_bindgen::JsValue;

// helper function to save to local storage
pub fn save_to_local_storage<T: Serialize>(key: &str, value: &T) -> Result<(), JsValue> {
    if let Ok(serialized) = serde_json::to_string(value) {
        let window = window();
        if let Ok(opt_storage) = window.local_storage() {
            if let Some(local_storage) = opt_storage {
                return local_storage.set_item(key, &serialized);
            }
        }
    }

    Err(JsValue::from_str(
        "Failed to attempt to get local_storage for the window.",
    ))
}

// helper function to load from local storage
pub fn load_from_local_storage<T: for<'a> Deserialize<'a>>(key: &str) -> Option<T> {
    let window = window();
    if let Ok(opt_storage) = window.local_storage() {
        if let Some(local_storage) = opt_storage {
            if let Ok(Some(value)) = local_storage.get_item(key) {
                if let Ok(deserialized) = serde_json::from_str(&value) {
                    return Some(deserialized);
                }
            }
        }
    }

    None
}

// helper function to delete from local storage
pub fn delete_from_local_storage(key: &str) -> Result<(), JsValue> {
    let window = window();
    if let Ok(opt_storage) = window.local_storage() {
        if let Some(local_storage) = opt_storage {
            return local_storage.remove_item(key);
        }
    }

    Err(JsValue::from_str(
        "Failed to attempt to remove a key from local_storage",
    ))
}
