#![doc = include_str!("../README.md")]
#![feature(concat_idents)]
#![feature(try_blocks)]

pub mod clipboard;
pub mod js_convert;
pub mod storage;
pub mod waiter;

#[cfg(target_arch = "wasm32")]
pub mod objecttools;

/// Log things to the console on the web and stdout on desktop.
#[macro_export]
macro_rules! console_log {
    ($msg:expr) => {{
        #[cfg(target_arch = "wasm32")] {
            let obj = sapp_jsutils::JsObject::string($msg);
            unsafe { $crate::objecttools::console_log(obj.weak()) };
        }
        #[cfg(not(target_arch = "wasm32"))]
        println!("{}", $msg);
    }};
    ($fmt:literal , $($args:expr),* $(,)*) => {{
        let msg = format!($fmt, $($args)*);
        console_log!(&msg);
    }};
}
