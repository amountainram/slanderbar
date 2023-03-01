use std::ops::Deref;

use js_sys::{JsString, Object};
use utils::{key_parser, parser, set_panic_hook};
use wasm_bindgen::prelude::*;

mod utils;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(start)]
pub fn __start() {
    set_panic_hook();
}

#[wasm_bindgen]
pub fn compile(x: &str, ctx: &Object) -> Result<String, JsError> {
    let (_, tagged_template) = parser(x).map_err(JsError::from)?;

    let resolved_variables: Vec<Option<JsValue>> = tagged_template
        .variables
        .iter()
        .map(|&var| key_parser(var))
        .map(|real_keys| {
            real_keys.iter().map(|&x| Some(x.into())).fold(
                None,
                |acc: Option<JsValue>, x: Option<JsValue>| {
                    log(format!("{:?}", acc).as_str());
                    if let Some(value) = x {
                        log(format!("{:?}", value).as_str());
                        return js_sys::Reflect::get(ctx, &value).ok();
                    }

                    acc
                },
            )
        })
        .collect();

    let compiled_string = tagged_template
        .literals
        .iter()
        .zip(&resolved_variables)
        .fold(String::default(), |acc, (&literal, variable)| {
            let var = variable
                .as_ref()
                .unwrap_or(&JsString::from(""))
                .dyn_ref::<JsString>()
                .unwrap()
                .as_string()
                .unwrap_or_default();

            acc + &literal + &var
        });

    Ok(compiled_string + tagged_template.literals.last().unwrap())
}
