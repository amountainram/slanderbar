//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

use core::panic;
use std::vec;

use slanderbar::*;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn pass() {
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(&obj, &"world".into(), &"World".into());

    let result = compile(String::from(r#"Hello, {{ world }}!"#), obj)
        .map_err(JsValue::from)
        .unwrap();

    assert_eq!(result, String::from("Hello, World!"));
}
