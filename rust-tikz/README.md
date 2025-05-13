# rust-tikzjax
Rust runtime for TikZ diagrams

`rust-tikzjax` is a rust runner for tikz. It builds off of the work of `tikzjax` and `node-tikzjax`.

[`tikzjax`](https://tikzjax.com/) is a WASM-compiled version of the TeX program. It comes with Javascript bindings
to handle the system calls that TeX makes. `rust-tikzjax` reimplements these systems calls in Rust. It then runs the WASM
version of TeX using [`wasmi`](https://github.com/wasmi-labs/wasmi), which is a WASM runtime that itself can be compiled to WASM.

The end result is a fully self-contained version of `tikz` that can input TeX source and output SVG images.

## Development

To get started, clone the repository, make sure you have `rust` and `cargo` installed, and run

```
cargo build
```

or, to execute example code run

```
cargo run
```

### TeX source code
Documentation for the TeX source code, including all system calls (that TeX relies on from Pascal) at https://tug.ctan.org/info/knuth-pdf/tex/tex.pdf