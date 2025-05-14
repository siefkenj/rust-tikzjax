# `typst-tikz`

`typst-tikz` is a Typst plugin that allows the embedding of TeX/LaTeX/TiKz code in a Typst document. It
works by embedding a WASM-compiled version of TeX, processing your code into a DVI, and then converting that DVI
into an SVG which is finally passed to Typst for rendering.

## Examples

  Regular LaTeX can be used:
  ````typst
    #typst-tikz("\\begin{document}Hello World!\\end{document}")
  ````

  ![Hello world example](https://raw.githubusercontent.com/siefkenj/rust-tikzjax/refs/heads/main/examples/readme-1.png)
  
  A tikz picture can be placed inside the document:
  ````typst
  #typst-tikz(```
      \begin{document}
          \begin{tikzpicture}
            \draw (0,0) circle (1in);
          \end{tikzpicture}
      \end{document}
    ```),
  ````
  
  ![Circle example](https://raw.githubusercontent.com/siefkenj/rust-tikzjax/refs/heads/main/examples/readme-2.png)
  
  More complicated TikZ figures may take a while to render.
  ````tikz
  #typst-tikz(```
  \usepackage{tikz-cd}

  \begin{document}
    \begin{tikzcd}
        T
        \arrow[drr, bend left, "x"]
        \arrow[ddr, bend right, "y"]
        \arrow[dr, dotted, "{(x,y)}" description] & & \\
        K & X \times_Z Y \arrow[r, "p"] \arrow[d, "q"]
        & X \arrow[d, "f"] \\
        & Y \arrow[r, "g"]
        & Z
    \end{tikzcd}

    \quad \quad

    \begin{tikzcd}[row sep=2.5em]
      A' \arrow[rr,"f'"] \arrow[dr,swap,"a"] \arrow[dd,swap,"g'"] &&
        B' \arrow[dd,swap,"h'" near start] \arrow[dr,"b"] \\
      & A \arrow[rr,crossing over,"f" near start] &&
        B \arrow[dd,"h"] \\
      C' \arrow[rr,"k'" near end] \arrow[dr,swap,"c"] && D' \arrow[dr,swap,"d"] \\
      & C \arrow[rr,"k"] \arrow[uu,<-,crossing over,"g" near end]&& D
    \end{tikzcd}
  \end{document}
    ```),
  ````

  ![Commutative diagram example](https://raw.githubusercontent.com/siefkenj/rust-tikzjax/refs/heads/main/examples/readme-3.png)

## Limitations

Currently, `typst-tikz` is _slow_. This is because `typst-tikz` embeds a WASM interpreter which in turn runs a WASM-compiled
version of TeX (from the [`tikzjax`](https://tikzjax.com/) project). This WASM-in-WASM process results in slow code for
complex figures.

At the moment, there are also limitations with Greek letters and other fonts (bold, etc.). Greek letters should be solvable with
more advanced DVI processing. Other fonts can be supported by providing them to the virtual file system used by `rust-tikz`.
