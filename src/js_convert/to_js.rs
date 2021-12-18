use crate::objecttools::ObjectTools;

use anyhow::{anyhow, Context};
use paste::paste;
use sapp_jsutils::JsObject;

use std::{collections::HashMap, convert::Infallible};

extern "C" {
    // Primitive passing
    fn primitive_to_js_u8(it: u8) -> JsObject;
    fn primitive_to_js_u16(it: u16) -> JsObject;
    fn primitive_to_js_u32(it: u32) -> JsObject;
    fn primitive_to_js_u64(it: u64) -> JsObject;
    fn primitive_to_js_usize(it: usize) -> JsObject;
    fn primitive_to_js_i8(it: i8) -> JsObject;
    fn primitive_to_js_i16(it: i16) -> JsObject;
    fn primitive_to_js_i32(it: i32) -> JsObject;
    fn primitive_to_js_i64(it: i64) -> JsObject;
    fn primitive_to_js_isize(it: isize) -> JsObject;
    fn primitive_to_js_f32(it: f32) -> JsObject;
    fn primitive_to_js_f64(it: f64) -> JsObject;
    fn primitive_to_js_bool(it: bool) -> JsObject;
}

/// Turn a Rust object into a JsObject.
pub trait ToJsObject {
    /// Error type returned if the conversion fails.
    type Error: Into<Box<dyn std::error::Error + Send + Sync + 'static>>;

    /// Turn a Rust object into a `JsObject`.
    ///
    /// Check the individual impls to see how the type is serialized.
    fn to_js(self) -> Result<JsObject, Self::Error>;
}

/// Strings are turned into JS Strings.
impl<'a> ToJsObject for &'a str {
    type Error = Infallible;

    fn to_js(self) -> Result<JsObject, Self::Error> {
        Ok(JsObject::string(self))
    }
}

/// Byte slices are turned into UInt8Arrays.
impl<'a> ToJsObject for &'a [u8] {
    type Error = Infallible;

    fn to_js(self) -> Result<JsObject, Self::Error> {
        Ok(JsObject::buffer(self))
    }
}

/// Various primitives can be converted into JS, says Fedor.
///
/// Note you can just pass numbers across the FFI boundary normally,
/// so this may or may not be helpful at all.
macro_rules! impl_to_js_primitive {
    ($($implee:ident)*) => {
        $(
            impl ToJsObject for $implee {
                type Error = Infallible;

                fn to_js(self) -> Result<JsObject, Self::Error> {
                    Ok(unsafe { paste! {
                        [< primitive_to_js_ $implee >](self)
                    }})
                }
            }
        )*
    }
}

impl_to_js_primitive!(u8 u16 u32 u64 usize i8 i16 i32 i64 isize f32 f64 bool);

/// real tricky one here
impl ToJsObject for JsObject {
    type Error = Infallible;

    fn to_js(self) -> Result<JsObject, Self::Error> {
        Ok(self)
    }
}

/// `Option<T>` is turned into whatever it would be turned into, or `null`.
///
/// Note: If you use this to convert an `Option<Option<T>>`, both `Some(None)`
/// and `None` will become `null`. If you try to convert `null` back into
/// `Option<Option<T>>`, it will become `None`. If this is not what you want
/// and you want to preserve nested `Option`s, look at [`LongOption`].
impl<T: ToJsObject> ToJsObject for Option<T> {
    type Error = <T as ToJsObject>::Error;

    fn to_js(self) -> Result<JsObject, Self::Error> {
        match self {
            Some(it) => it.to_js(),
            None => Ok(JsObject::null()),
        }
    }
}

/// - `Ok(it)` is `{ ok: it }`
/// - `Err(it) is `{ err: it }`
impl<T: ToJsObject, E: ToJsObject> ToJsObject for Result<T, E> {
    // We can't know if it is T's error or E's error,
    // so we just make it whateverror.
    //
    // I would love for this to be some not-String type but i really just can't make it
    // happen, so many trait errors
    type Error = anyhow::Error;

    fn to_js(self) -> Result<JsObject, Self::Error> {
        let (inner, field) = match self {
            Ok(it) => {
                let inner = it
                    .to_js()
                    .map_err(|e| anyhow!(e.into()))
                    .context("When converting Ok to JS")?;
                (inner, "ok")
            }
            Err(it) => {
                let inner = it
                    .to_js()
                    .map_err(|e| anyhow!(e.into()))
                    .context("When converting Err to JS")?;
                (inner, "err")
            }
        };

        let mut wrapper = JsObject::object();
        wrapper.set_field(field, inner)?;
        Ok(wrapper)
    }
}

/// Convert vecs into arrays.
impl<T: ToJsObject> ToJsObject for Vec<T> {
    type Error = anyhow::Error;

    fn to_js(self) -> Result<JsObject, Self::Error> {
        let mut arr = JsObject::array();
        for (idx, obj) in self.into_iter().enumerate() {
            arr.set_field(idx, obj)?;
        }
        Ok(arr)
    }
}

/// HashMaps turns into an object.
impl<K: AsRef<str>, V: ToJsObject> ToJsObject for HashMap<K, V> {
    type Error = anyhow::Error;

    fn to_js(self) -> Result<JsObject, Self::Error> {
        let mut obj = JsObject::object();
        for (k, v) in self.into_iter() {
            let v = v.to_js().map_err(|e| anyhow!(e.into()))?;
            obj.set_field(k.as_ref().to_js()?, v)?;
        }
        Ok(obj)
    }
}

/// May as well implement stuff for boxes
impl<T: ToJsObject> ToJsObject for Box<T> {
    type Error = <T as ToJsObject>::Error;

    fn to_js(self) -> Result<JsObject, Self::Error> {
        (*self).to_js()
    }
}
