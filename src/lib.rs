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
                Some(ctx.into()),
                |acc: Option<JsValue>, x: Option<JsValue>| {
                    if let Some(value) = x {
                        return js_sys::Reflect::get(&acc.unwrap(), &value).ok();
                    }

                    acc
                },
            )
        })
        .collect();

    let default_js_string = JsString::from("");
    let compiled_string = tagged_template
        .literals
        .iter()
        .zip(&resolved_variables)
        .fold(String::default(), |acc, (&literal, variable)| {
            let var = variable
                .as_ref()
                .unwrap_or(&default_js_string)
                .dyn_ref::<JsString>()
                .unwrap_or(&default_js_string)
                .as_string()
                .unwrap_or_default();

            acc + &literal + &var
        });

    Ok(compiled_string + tagged_template.literals.last().unwrap())
}
