# quad-wasmnastics

Utilities that do the gymnastics required to make advanced Macroquad work on wasm.

IMPORTANT: In order to use this on WASM, you MUST use the [`js/wasmnastics.js`](js/wasmnastics.js) script in your HTML file
before any WASM code is run! Put it right after your `mq_js_bundle.js` for best results.

Also Important: This code is wildly untested, so be careful using it please.

## Async Things

A lot of web APIs use Javascript `async` where similar APIs on desktop are synchronous, like clipboard access.

This crate provides the `Waiter` struct for exactly this purpose. It's similar to `Poll` from the std library: it's created,
and then you query it to see if the value's gotten to you yet. On desktop it immediately returns the value.
On the web the value needs to be awaited; it will get back to you in a few `next_frame`s.

## Clipboard

Miniquad's clipboard doesn't especially work on the web. So, this uses the waiter API to expose the web's experimental
async clipboard API.

## Storage

Save files are important! This crate has an API for saving all of your important game data. On desktop, this stores things in your [Data directory](https://docs.rs/dirs/3.0.2/dirs/fn.data_dir.html), so `%APPDATA%` on Windows, `Library/Application Support` on Mac, etc. On the web, it stores it in [local storage](https://developer.mozilla.org/en-US/docs/Web/API/Window/localStorage).

Both support storing strings and byte arrays. On both platforms the data is gzipped before being stored; on the web the data is then base64 encoded (because local storage only supports strings).

This also exposes the deflating and inflating functions for utility's sake.

## Converting JS Objects and Rust Objects

This crate has `ToJsObject` and `FromJsObject` traits, which (as you might expect) let you convert things between
JS and Rust more fluently. They're available only on WASM (because what are you doing with those on desktop?).

It's implemented for many popular types, and you can also implement it yourself.

Finally, there's the `SerDeWrapper` struct on crate feature `serde_wrapper`, which turn Rust objects into JSON strings and vice versa.

## Object Tools

`sapp_jsutils` is the stock library working for JsObjects, and while it's very nice there are some significant shortcomings.
This crate provides an extension trait for JsObjects, `ObjectTools`. It adds more utilities, like directly converting things to strings,
boolean support, and more.
