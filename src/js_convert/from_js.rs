use crate::objecttools::{JsType, ObjectTools};

use anyhow::{anyhow, bail};
use sapp_jsutils::JsObject;

use std::{convert::Infallible, fmt};

/// Convert things from JS to Rust.
///
/// Note that due to general JS flimsiness, a lot of these functions may panic
/// on the JS side if called with badly typed JS types.
/// They do their best to catch stuff but be careful and probably don't try
/// and do anything too crazy with this.
pub trait FromJsObject: Sized {
    /// Error type returned if the conversion fails.
    type Error: Into<Box<dyn std::error::Error + Send + Sync + 'static>>;

    /// Turn a `JsObject` into a Rust object.
    fn from_js(obj: JsObject) -> Result<Self, Self::Error>;
}

/// Convert JS Strings into Strings.
impl FromJsObject for String {
    type Error = BadJsTypeError;

    fn from_js(obj: JsObject) -> Result<Self, Self::Error> {
        let ty = obj.js_type();
        if ty != JsType::String {
            Err(BadJsTypeError::new(vec![JsType::String], ty))
        } else {
            Ok(obj.to_string_direct())
        }
    }
}

/// mega difficult here
impl FromJsObject for JsObject {
    type Error = Infallible;

    fn from_js(obj: JsObject) -> Result<Self, Self::Error> {
        Ok(obj)
    }
}

/*
ill do this later
/// Convert arrays to Vecs.
impl<T: FromJsObject> FromJsObject for Vec<T> {
    type Error = anyhow::Error;

    fn from_js(obj: JsObject) -> Result<Self, Self::Error> {
        let len = match obj.try_get_field("length") {
            Some(it) => {it},
            None => {bail!("Could not find `length` field")},
        };
        let len = usize::from_js
    }
}
*/

/// Any value can turn into `()`.
impl FromJsObject for () {
    type Error = Infallible;

    fn from_js(_: JsObject) -> Result<Self, Self::Error> {
        Ok(())
    }
}

/// - `null` becomes None.
/// - Anything else becomes Some.
///
/// If you want to handle nested options, check out [`LongOption`].
///
/// [`LongOption`]: super::wrappers::LongOption
impl<T: FromJsObject> FromJsObject for Option<T> {
    type Error = <T as FromJsObject>::Error;

    fn from_js(obj: JsObject) -> Result<Self, Self::Error> {
        if obj.is_null() {
            Ok(None)
        } else {
            T::from_js(obj).map(Some)
        }
    }
}

/// - `{ ok: it }` becomes `Ok(it)`.
/// - `{ err: it }` becomes `Err(it)`.
///
/// Note this function will return a nested Result.
/// The first layer is for JS-level issues; the second layer is your actual result.
/// In other words, unwrapping the return value returns what JS meant to get to you.
impl<T: FromJsObject, E: FromJsObject> FromJsObject for Result<T, E> {
    type Error = anyhow::Error;

    fn from_js(obj: JsObject) -> Result<Self, Self::Error> {
        Ok(if let Some(it) = obj.try_get_field("ok") {
            Ok(T::from_js(it).map_err(|e| anyhow!(e.into()))?)
        } else if let Some(it) = obj.try_get_field("err") {
            Err(E::from_js(it).map_err(|e| anyhow!(e.into()))?)
        } else {
            bail!("Didn't find field `ok` or `err`")
        })
    }
}

/// Error due to the JS object being the wrong type
#[derive(Debug)]
pub struct BadJsTypeError {
    /// The type(s) we wanted
    pub wanted: Vec<JsType>,
    /// The type we got
    pub got: JsType,
}

impl BadJsTypeError {
    pub fn new(wanted: Vec<JsType>, got: JsType) -> Self {
        Self { wanted, got }
    }
}

impl fmt::Display for BadJsTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Expected JS type(s) {:?}, got type {:?}",
            &self.wanted, self.got
        )
    }
}

impl std::error::Error for BadJsTypeError {}
