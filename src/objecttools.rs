//! sapp_jsutils is not feature complete enough for me!

use crate::js_convert::{wrappers::LongOption, BadJsTypeError, FromJsObject, ToJsObject};

use anyhow::{anyhow, bail};
use sapp_jsutils::{JsObject, JsObjectWeak};

extern "C" {
    /// Return `null`
    fn null() -> JsObject;
    /// Return the `typeof` something as a JsObject string.
    fn type_of(obj: JsObjectWeak) -> JsObject;
    /// `obj[key] = val`, which sapp doesn't let you do for some reason
    fn set_field_any(obj: JsObjectWeak, key: JsObjectWeak, val: JsObjectWeak);
    /// `.toString`
    fn as_string(obj: JsObjectWeak) -> JsObject;
    fn array() -> JsObject;
    fn try_get_field(obj: JsObjectWeak, key: JsObjectWeak) -> JsObject;
    /// Check if something `==` or `===` something else
    fn equals(a: JsObjectWeak, b: JsObjectWeak, triple: bool) -> bool;
    fn has_field(obj: JsObjectWeak, buf: *const u8, len: u32) -> bool;

    pub fn console_log(msg: JsObjectWeak);
}

/// Extension trait for JsObjects!
pub trait ObjectTools {
    /// Return `null` as a JsObject.
    ///
    /// Note this is not the same as nil, aka a JsObject with an index of -1.
    /// This is an honest-to-goodness JS `null`.
    fn null() -> JsObject;

    /// Check the type of this object using JS' `typeof` operator.
    ///
    /// This being Javascript, there are some edge cases. For example, `null` is an `object`.
    /// Probably best to [read the official docs](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/typeof)
    /// if you need any advanced usage.
    fn js_type(&self) -> JsType;

    /// Convert this to a string directly without any `&mut buffer`ing, as a convenience function.
    ///
    /// Just like `to_string()`, this will panic if `self` is not a JS String.
    fn to_string_direct(&self) -> String;

    /// Convert this to a string using the `toString` method.
    /// This won't panic under normal circumstances. (For example you *could* make an object
    /// with a toString method that throws an exception, but why would you do that?)
    fn as_string(&self) -> String;

    /// Set any string field of this to any value.
    /// This works with arrays and objects.
    fn set_field<K: ToJsObject, V: ToJsObject>(&mut self, key: K, val: V) -> anyhow::Result<()>;

    /// Get an empty array.
    fn array() -> JsObject;

    /// Try and get a field.
    ///
    /// If the conversion fails, return None;
    fn try_get_field<K: ToJsObject>(&self, key: K) -> Option<JsObject>;

    /// Check if this `===` something else.
    ///
    /// If the conversion to `JsObject` fails, return `false`.
    fn equals_strict<T: ToJsObject>(&self, rhs: T) -> bool;

    /// Check if this `==` something else.
    ///
    /// If the conversion to `JsObject` fails, return `false`.
    fn equals_weak<T: ToJsObject>(&self, rhs: T) -> bool;

    /// Check if this is [truthy](https://developer.mozilla.org/en-US/docs/Glossary/Truthy).
    fn truthy(&self) -> bool {
        self.equals_weak(true)
    }

    /// Check if this is `null`.
    ///
    /// Note this is not the same as nil (aka, a JsObject pointing to -1).
    fn is_null(&self) -> bool {
        self.equals_strict(JsObject::null())
    }

    /// Check if this has the given field.
    /// This is like `have_field` from sapp_jsutils, but:
    /// - actually checks if the field *does* exist instead of doesn't exist
    /// - uses `=== undefined` instead of `== undefined`
    fn has_field(&self, field: &str) -> bool;
}

impl ObjectTools for JsObject {
    fn null() -> JsObject {
        unsafe { self::null() }
    }

    fn js_type(&self) -> JsType {
        let ty = unsafe { type_of(self.weak()) };
        let ty = ty.as_string();

        match ty.as_str() {
            "undefined" => JsType::Undefined,
            "object" => JsType::Object,
            "boolean" => JsType::Boolean,
            "number" => JsType::Number,
            "bigint" => JsType::Bigint,
            "string" => JsType::String,
            "symbol" => JsType::Symbol,
            "function" => JsType::Function,
            _ => JsType::Unknown(ty),
        }
    }

    fn to_string_direct(&self) -> String {
        let mut buf = String::new();
        self.to_string(&mut buf);
        buf
    }

    fn as_string(&self) -> String {
        let stringed = unsafe { as_string(self.weak()) };
        stringed.to_string_direct()
    }

    fn set_field<K: ToJsObject, V: ToJsObject>(&mut self, key: K, val: V) -> anyhow::Result<()> {
        let ty = self.js_type();
        if ty != JsType::Object {
            bail!(BadJsTypeError::new(vec![JsType::Object], ty));
        }

        let key = key.to_js().map_err(|e| anyhow!(e.into()))?;
        let val = val.to_js().map_err(|e| anyhow!(e.into()))?;
        unsafe { set_field_any(self.weak(), key.weak(), val.weak()) };
        Ok(())
    }

    fn array() -> JsObject {
        unsafe { array() }
    }

    fn try_get_field<K: ToJsObject>(&self, key: K) -> Option<JsObject> {
        let key = match key.to_js() {
            Ok(it) => it,
            Err(_) => return None,
        };
        // The FFI returns a LongOption
        let maybe = unsafe { try_get_field(self.weak(), key.weak()) };
        let maybe = match LongOption::from_js(maybe) {
            Ok(it) => it,
            Err(_) => return None,
        };
        maybe.0
    }

    fn equals_strict<T: ToJsObject>(&self, rhs: T) -> bool {
        let rhs = match rhs.to_js() {
            Ok(it) => it,
            Err(_) => return false,
        };
        unsafe { equals(self.weak(), rhs.weak(), true) }
    }
    fn equals_weak<T: ToJsObject>(&self, rhs: T) -> bool {
        let rhs = match rhs.to_js() {
            Ok(it) => it,
            Err(_) => return false,
        };
        unsafe { equals(self.weak(), rhs.weak(), false) }
    }

    fn has_field(&self, field: &str) -> bool {
        unsafe { has_field(self.weak(), field.as_ptr(), field.len() as _) }
    }
}

/// Types that JS has.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum JsType {
    Undefined,
    Object,
    Boolean,
    Number,
    Bigint,
    String,
    Symbol,
    Function,
    /// According to the spec, `typeof` can really return whatever string it wants.
    /// Although IE is the only browser that does this, we have to handle it...
    Unknown(String),
}
