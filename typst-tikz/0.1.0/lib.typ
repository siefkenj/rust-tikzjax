#let _wasm = plugin("assets/typst_tikz_lib.wasm")

/// Render the string `input` as an SVG. `input` is assumed
/// to be a valid LaTeX document. That is, it starts with
/// `\begin{document}` and ends with `\end{document}`.
///
/// `tikz` is already loaded. Some other libraries can be loaded
/// by adding `\usepackage{<package>}` to the start of your string.
#let typst-tikz(input) = {
  if type(input) == content {
    input = input.text
  }
  let result = _wasm.render_tex(bytes(input))
  image(result)
}

/// Like `typst-tikz`, but returns the raw SVG string.
#let typst-tikz-svg(input) = {
  if type(input) == content {
    input = input.text
  }
  let result = _wasm.render_tex(bytes(input))
  result
}


#[
  = `typst-tikz` Examples

  Regular LaTeX can be used:
  ````typst
    #typst-tikz("\\begin{document}Hello World!\\end{document}")
  ````
  to produce

  #box(
    stroke: black,
    inset: 0.5em,
    typst-tikz("\\begin{document}Hello World!\\end{document}"),
  )

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
  to produce

  #box(
    stroke: black,
    inset: 0.5em,
    typst-tikz(```
      \begin{document}
          \begin{tikzpicture}
            \draw (0,0) circle (1in);
          \end{tikzpicture}
      \end{document}
    ```),
  )

  #pagebreak()
  More complicated TikZ figures may take a while to render.

  ````typst
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
  to produce

  #box(
    stroke: black,
    inset: 0.5em,
    typst-tikz(```
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
  )


  #pagebreak()

  ````typst
  #typst-tikz(```
    \usepackage{circuitikz}
    \begin{document}
      \begin{circuitikz}[american, voltage shift=0.5]
        \draw (0,0)
          to[isource, l=$I_0$, v=$V_0$] (0,3)
          to[short, -*, i=$I_0$] (2,3)
          to[R=$R_1$, i>_=$i_1$] (2,0) -- (0,0);
          \draw (2,3) -- (4,3)
          to[R=$R_2$, i>_=$i_2$]
          (4,0) to[short, -*] (2,0);
      \end{circuitikz}
    \end{document}
    ```),
  ````
  to produce

  #box(
    stroke: black,
    inset: 0.5em,
    typst-tikz(```
    \usepackage{circuitikz}
    \begin{document}
      \begin{circuitikz}[american, voltage shift=0.5]
        \draw (0,0)
          to[isource, l=$I_0$, v=$V_0$] (0,3)
          to[short, -*, i=$I_0$] (2,3)
          to[R=$R_1$, i>_=$i_1$] (2,0) -- (0,0);
          \draw (2,3) -- (4,3)
          to[R=$R_2$, i>_=$i_2$]
          (4,0) to[short, -*] (2,0);
      \end{circuitikz}
    \end{document}
    ```),
  )
]
