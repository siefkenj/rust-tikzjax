# `typst-tikz`

`typst-tikz` is a Typst plugin that allows the embedding of TeX/LaTeX/TiKz code in a Typst document. It
works by embedding a WASM-compiled version of TeX, processing your code into a DVI, and then converting that DVI
into an SVG which is finally passed to Typst for rendering.

## Examples

## Limitations

Currently, `typst-tikz` is _slow_. This is because `typst-tikz` embeds a WASM interpreter which in turn runs a WASM-compiled
version of TeX (from the [`tikzjax`](https://tikzjax.com/) project). This WASM-in-WASM process results in slow code for
complex figures.

At the moment, there are also limitations with Greek letters and other fonts (bold, etc.). Greek letters should be solvable with
more advanced DVI processing. Other fonts can be supported by providing them to the virtual file system used by `rust-tikz`.
