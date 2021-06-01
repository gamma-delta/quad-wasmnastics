#![feature(concat_idents)]
#![feature(try_blocks)]

pub mod clipboard;
pub mod js_convert;
pub mod storage;
pub mod waiter;

#[cfg(target_arch = "wasm32")]
pub mod objecttools;
