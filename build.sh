#!/bin/bash

cargo build -p typst-tikz-lib --target=wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/typst_tikz_lib.wasm typst-tikz/0.1.0/assets/