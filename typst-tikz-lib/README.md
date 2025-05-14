# `typst_tikz`

A Typst plugin with a WASM implementation of the TeX engine and core that has tikz pre-loaded in memory.

# Development

`typst-tikz` relies on the `rust-tikz` to compile tex source to SVG. The bundled version of TeX
includes the tikz libraries pre-loaded, but doesn't include all LaTeX packages.

## Building

To build run
```
cargo build --target=wasm32-unknown-unknown --release
```
then the resulting WASM file will be in `../target/wasm32-unknown-unknown/release/typst_tikz.wasm`. It needs
to be copied to a place where Typst can load it.