use crate::js_convert::MaybeFromJsObject;

/// Something that is waiting on a value from Javascript.
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

                let res: Result<Option<T>, ()> = try {
                    let waiting = waiter.try_get_field("waiting").ok_or(())?;
                    if waiting.truthy() {
                        // too bad
                        None
                    } else {
                        // ooh we get our value?
                        let value = waiter.try_get_field("value").ok_or(())?;
                        let value = T::from_js(value).map_err(|_| ())?;
                        // nice!
                        Some(value)
                    }
                };
                if let Ok(it) = res {
                    if let Some(it) = it {
                        self.inner = WaiterInner::Taken;
                        Some(it)
                    } else {
                        // Oh well, better luck next frame?
                        None
                    }
                } else {
                    // oh no
                    self.inner = WaiterInner::Error;
                    None
                }
            }

            #[cfg(target_arch = "wasm32")]
            WaiterInner::Error => None,
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
    /// An error occurred when trying to turn the JsObject into an actual object.
    #[cfg(target_arch = "wasm32")]
    Error,
}
