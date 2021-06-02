use crate::{
    js_convert::{BadJsTypeError, FromJsObject, ToJsObject},
    objecttools::{JsType, ObjectTools},
};

use anyhow::{anyhow, bail};
use sapp_jsutils::JsObject;

#[cfg(feature = "serde_wrapper")]
use serde::{de::DeserializeOwned, Serialize};

use std::convert::Infallible;

/// An enum that works just like Option, but serializes to
/// JS differently.
///
/// - `Some(inner)` becomes `{ some: <inner as js> }`
/// - `None` becomes `null`
///
/// This lets you handle nested options correctly.
///
/// - `Some(Some(it))` becomes `{ some : { some: it }}`
/// - `Some(None)` becomes `{ some: null }`
/// - `None` becomes `null`
pub struct LongOption<T>(pub Option<T>);

impl<T> std::ops::Deref for LongOption<T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: FromJsObject> FromJsObject for LongOption<T> {
    type Error = anyhow::Error;

    fn from_js(obj: JsObject) -> Result<Self, Self::Error> {
        Ok(if obj.is_null() {
            LongOption(None)
        } else {
            // We can't use `try_get_field` or else we will infinite loop
            let has_some = obj.has_field("some");
            if has_some {
                let some = obj.field("some");
                LongOption(Some(T::from_js(some).map_err(|e| anyhow!(e.into()))?))
            } else {
                bail!("Expected field `some` but didn't find it")
            }
        })
    }
}

/// Due to Rust's lack of specialization, we can't specially handle converting a `Uint8Array`
/// into a `Vec<u8>`.
///
/// This wrapper struct can be used to do that conversion; it converts from
/// `Uint8Array`, where `Vec<u8>` would convert from a normal JS array.
///
/// It also can be converted to a Uint8Array for convenience
/// (it just passes through to the `&[u8]` impl).
pub struct Uint8Array(pub Vec<u8>);

impl FromJsObject for Uint8Array {
    type Error = BadJsTypeError;

    fn from_js(obj: JsObject) -> Result<Self, Self::Error> {
        let ty = obj.js_type();
        if ty != JsType::Object {
            Err(BadJsTypeError::new(vec![JsType::Object], ty))
        } else {
            let mut buf = Vec::new();
            obj.to_byte_buffer(&mut buf);
            Ok(Uint8Array(buf))
        }
    }
}

impl ToJsObject for Uint8Array {
    type Error = Infallible;

    fn to_js(self) -> Result<JsObject, Self::Error> {
        self.0.as_slice().to_js()
    }
}

/// Interconvert JSON strings and Rust objects.
#[cfg(feature = "serde_wrapper")]
pub struct SerDeWrapper<T>(pub T);

#[cfg(feature = "serde_wrapper")]
impl<T: Serialize> ToJsObject for SerDeWrapper<&T> {
    type Error = anyhow::Error;

    fn to_js(self) -> Result<JsObject, Self::Error> {
        let jsoned = serde_json::to_string(&self.0)?;
        Ok(JsObject::string(&jsoned))
    }
}

#[cfg(feature = "serde_wrapper")]
impl<T: DeserializeOwned> FromJsObject for SerDeWrapper<T> {
    type Error = anyhow::Error;

    fn from_js(obj: JsObject) -> Result<Self, Self::Error> {
        let string = obj.as_string();
        // We use a reader here to prevent serde from trying to deserialize with borrowed data from the string
        let reader = std::io::Cursor::new(string.as_bytes());
        let deserialized = serde_json::from_reader(reader)?;
        Ok(SerDeWrapper(deserialized))
    }
}
