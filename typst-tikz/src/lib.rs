use rust_tikz::text2svg_simple;
#[cfg(target_arch = "wasm32")]
use wasm_minimal_protocol::*;

#[cfg(target_arch = "wasm32")]
wasm_minimal_protocol::initiate_protocol!();

#[cfg_attr(target_arch = "wasm32", wasm_func)]
pub fn render_tex(in_str: &[u8]) -> Result<Vec<u8>, String> {
    let in_str = String::from_utf8_lossy(in_str);
    let result = text2svg_simple(&in_str);

    // b"Hello, world!\n".to_vec()
    //Ok(b"All good".to_vec());
    Err("Not all good".to_string())
}
