use js_sys::Object;
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
pub fn compile(x: String, ctx: Object) -> Result<String, JsError> {
    let (_, tagged_template) = parser(x.as_str()).map_err(JsError::from)?;

    tagged_template
        .variables
        .iter()
        .map(|&var| key_parser(var))
        .for_each(|real_keys| log(real_keys.join(", ").as_str()));

    Ok(tagged_template.literals.join(""))
}
