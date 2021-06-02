//! In case working with `async` code wasn't already nightmarish enough!

use crate::js_convert::MaybeFromJsObject;

use std::fmt::{self, Debug};

/// Something that is waiting on a value from Javascript,
/// or has the value immediately via a desktop API.
///
/// This struct was made for the Clipboard API, so it might be ill-suited
/// for other APIs you may wish to implement. If you need more functionality
/// feel free to open an issue or something.
#[derive(Debug)]
pub struct Waiter<T> {
    inner: WaiterInner<T>,
}

impl<T: MaybeFromJsObject> Waiter<T> {
    /// Make a new Waiter with an immediate value.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new_immediate(val: T) -> Self {
        Self {
            inner: WaiterInner::Available(val),
        }
    }

    /// Make a new Waiter from a JsObject returned from `waitify`.
    #[cfg(target_arch = "wasm32")]
    pub fn new_waiting(waiter: sapp_jsutils::JsObject) -> Self {
        Self {
            inner: WaiterInner::Waiting(waiter, std::marker::PhantomData),
        }
    }

    /// Make a new Waiter that will never return Some, in case you wanted to do that for some reason.
    pub fn new_empty() -> Self {
        Self {
            inner: WaiterInner::Taken,
        }
    }

    /// Try and get the value from this.
    ///
    /// You shouldn't write your code relying on this to return anytime soon.
    /// On desktop it might, but it may take several frames on the web.
    /// Probably best to assume for the worst.
    ///
    /// This will never return `Some` if the conversion from JS fails, or if
    /// something else wrong happens in JS, like no `waiting` field.
    /// (The JS API will try and print any exceptions to the console but you can't
    /// rely on it from Rust.)
    ///
    /// Once this returns `Some` once, it will never return `Some` again (for a specific `Waiter`).
    /// The value will have moved out of it.
    pub fn try_get(&mut self) -> Option<T> {
        match &mut self.inner {
            WaiterInner::Taken => {
                // what are you doing go away
                None
            }

            #[cfg(not(target_arch = "wasm32"))]
            WaiterInner::Available(_) => {
                // entry api when
                let taken = std::mem::replace(&mut self.inner, WaiterInner::Taken);
                let taken = match taken {
                    WaiterInner::Available(it) => it,
                    _ => unreachable!(),
                };
                Some(taken)
            }

            #[cfg(target_arch = "wasm32")]
            WaiterInner::Waiting(waiter, _phantom) => {
                use crate::objecttools::ObjectTools;

                let res: Result<Option<T>, String> = try {
                    let waiting = waiter
                        .try_get_field("waiting")
                        .ok_or_else(|| "Couldn't find `waiting` field".to_string())?;
                    if waiting.truthy() {
                        // too bad
                        None
                    } else {
                        // ooh we get our value?
                        let value = waiter
                            .try_get_field("value")
                            .ok_or_else(|| "Couldn't find `value` field".to_string())?;
                        let value = T::from_js(value).map_err(|e| {
                            let err: Box<_> = e.into();
                            err.to_string()
                        })?;
                        // nice!
                        Some(value)
                    }
                };
                match res {
                    Ok(it) => {
                        if let Some(it) = it {
                            self.inner = WaiterInner::Taken;
                            Some(it)
                        } else {
                            // Oh well, better luck next frame?
                            None
                        }
                    }
                    Err(oh_no) => {
                        self.inner = WaiterInner::Error(oh_no);
                        None
                    }
                }
            }

            #[cfg(target_arch = "wasm32")]
            WaiterInner::Error(..) => None,
        }
    }
}

enum WaiterInner<T> {
    /// The value has been taken.
    Taken,

    /// The value is immediately available.
    #[cfg(not(target_arch = "wasm32"))]
    Available(T),
    /// On the web, we wait.
    ///
    /// I hate waiting.
    ///
    /// Must have the phantom data here because otherwise the T isn't used anywhere in the type
    /// on wasm.
    #[cfg(target_arch = "wasm32")]
    Waiting(sapp_jsutils::JsObject, std::marker::PhantomData<T>),
    /// An error occurred somewhere.
    /// And here's your error!
    #[cfg(target_arch = "wasm32")]
    Error(String),
}

/// JsObject doesn't impl Debug >:(
impl<T: Debug> Debug for WaiterInner<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Taken => write!(f, "Taken"),
            #[cfg(not(target_arch = "wasm32"))]
            Self::Available(it) => write!(f, "Available({:?})", it),
            #[cfg(target_arch = "wasm32")]
            WaiterInner::Waiting(_, _) => write!(f, "Waiting"),
            #[cfg(target_arch = "wasm32")]
            WaiterInner::Error(e) => write!(f, "Error({})", e),
        }
    }
}
