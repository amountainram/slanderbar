use utils::{parser, set_panic_hook};
use wasm_bindgen::prelude::*;

mod utils;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    pub type Template;
}

#[wasm_bindgen(start)]
pub fn init() {
    set_panic_hook();
}

#[wasm_bindgen]
pub fn compile(x: String, ctx: Template) {
    let (_, tagged_template) = parser(x.as_str()).unwrap();
    log(format!("{:?}", tagged_template).as_str());
    let map = ctx
        .dyn_into::<js_sys::Map>()
        .unwrap_or_else(|_| js_sys::Map::new());
    log(format!("{:?}", map).as_str());
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn pass() {}
}
