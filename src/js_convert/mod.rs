//! Converting between Rust and JsObjects.

#[cfg(target_arch = "wasm32")]
pub mod from_js;
#[cfg(target_arch = "wasm32")]
pub mod to_js;
#[cfg(target_arch = "wasm32")]
pub mod wrappers;

#[cfg(target_arch = "wasm32")]
pub use from_js::*;
#[cfg(target_arch = "wasm32")]
pub use to_js::*;

/// This trait is equal to [`FromJsObject`] on wasm, and has no bounds on desktop.
///
/// This lets you represent APIs that must work with the web, but don't need to.
/// For example, if you just want clipboard functionality, no need to compile
/// the whole `sapp_jsutils`.
///
/// You are currently on desktop, so no FromJsObject.
#[cfg(not(target_arch = "wasm32"))]
pub trait MaybeFromJsObject {}

#[cfg(not(target_arch = "wasm32"))]
impl<T> MaybeFromJsObject for T {}

/// This trait is equal to [`FromJsObject`] on wasm, and has no bounds on desktop.
///
/// This lets you represent APIs that must work with the web, but don't need to.
/// For example, if you just want clipboard functionality, no need to compile
/// the whole `sapp_jsutils`.
///
/// You are currently on wasm, so we use FromJsObject.
#[cfg(target_arch = "wasm32")]
pub trait MaybeFromJsObject: FromJsObject {}

#[cfg(target_arch = "wasm32")]
impl<T: FromJsObject> MaybeFromJsObject for T {}
