use anyhow::{Error, Result};
use dvi2html::tfm;
use dvi2svg::dvi2svg;

mod filesystem;
mod texjax_imports;
use filesystem::*;
mod dvi2svg;
mod wasm_runner;
pub use wasm_runner::*;

/// Convert `input` into an SVG string. This function creates a new [`WasmRunner`]
/// each time it is called. If you want to convert multiple strings, it is more efficient
/// to use [`tex2svg`] instead.
pub fn text2svg_simple(input: &str) -> Result<String> {
    let mut wasm_runner = WasmRunner::new()?;
    let svg_result = tex2svg(&mut wasm_runner, input);
    if svg_result.is_err() {
        let error = svg_result.unwrap_err();
        println!("Error: {}", error);
        // Show the log file
        let log_result = wasm_runner.get_messages()?;
        println!("input.log:\n{}", log_result);
        return Err(Error::msg("Failed to convert DVI to SVG."));
    }

    Ok(svg_result.unwrap())
}
