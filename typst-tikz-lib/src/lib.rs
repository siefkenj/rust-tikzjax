use rust_tikz::text2svg_simple;
#[cfg(target_arch = "wasm32")]
use wasm_minimal_protocol::*;

#[cfg(target_arch = "wasm32")]
wasm_minimal_protocol::initiate_protocol!();

#[cfg_attr(target_arch = "wasm32", wasm_func)]
pub fn render_tex(in_str: &[u8]) -> Result<Vec<u8>, String> {
    let in_str = String::from_utf8_lossy(in_str);
    let result = text2svg_simple(&in_str);
    if result.is_err() {
        let error = result.unwrap_err();
        // If there is an error, the stdout, etc. will be put in the error message.
        // We want to have that make its way back to the caller.
        return Err(format!("{}", error));
    }

    Ok(result.unwrap().into_bytes())
}
