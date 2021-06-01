use crate::waiter::Waiter;

/// Try and get the string value off the clipboard.
/// This could support other things (like images) but ehhhhh
///
/// Because the JS clipboard API is `async` for some horrid reason, returns a Waiter.
///
/// If an error occurs on desktop it will return a Waiter that will never return Some.
pub fn get_clipboard() -> Waiter<String> {
    #[cfg(target_arch = "wasm32")]
    {
        wasm::get_clipboard()
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        use clipboard::{ClipboardContext, ClipboardProvider};

        let res: Result<String, ()> = try {
            let mut provider = ClipboardContext::new().map_err(|_| ())?;
            provider.get_contents().map_err(|_| ())?
        };
        match res {
            Ok(text) => Waiter::new_immediate(text),
            Err(_) => Waiter::new_empty(),
        }
    }
}

/// Try and set the clipboard.
///
/// The returned `Waiter` will resolve to `()` once its task is complete.
pub fn set_clipboard(text: String) -> Waiter<()> {
    #[cfg(target_arch = "wasm32")]
    {
        wasm::set_clipboard(&text)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use clipboard::{ClipboardContext, ClipboardProvider};

        let res: Result<(), ()> = try {
            let mut provider = ClipboardContext::new().map_err(|_| ())?;
            provider.set_contents(text).map_err(|_| ())?;
        };
        match res {
            Ok(()) => Waiter::new_immediate(()),
            Err(_) => Waiter::new_empty(),
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use sapp_jsutils::{JsObject, JsObjectWeak};

    use crate::waiter::Waiter;

    extern "C" {
        fn clipboard_get() -> JsObject;
        fn clipboard_set(text: JsObjectWeak) -> JsObject;
    }

    pub fn get_clipboard() -> Waiter<String> {
        Waiter::new_waiting(unsafe { clipboard_get() })
    }

    pub fn set_clipboard(text: &str) -> Waiter<()> {
        let text = JsObject::string(&text);
        Waiter::new_waiting(unsafe { clipboard_set(text.weak()) })
    }
}
