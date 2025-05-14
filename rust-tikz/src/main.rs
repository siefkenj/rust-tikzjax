use anyhow::{Error, Result};
use dvi2html::tfm;
use dvi2svg::dvi2svg;

mod filesystem;
mod texjax_imports;
use filesystem::*;
mod dvi2svg;
mod wasm_runner;
pub use wasm_runner::*;

fn main() -> Result<()> {
    let input = r#"
        \begin{document}
%\begin{tikzpicture}[domain=0:4]
%  \draw[very thin,color=gray] (-0.1,-1.1) grid (3.9,3.9);
%  \draw[->] (-0.2,0) -- (4.2,0) node[right] {$x$};
%  \draw[->] (0,-1.2) -- (0,4.2) node[above] {$f(x)$};
%  \draw[color=red]    plot (\x,\x)             node[right] {$f(x) =x$};
%  \draw[color=blue]   plot (\x,{sin(\x r)})    node[right] {$f(x) = \sin x$};
%  \draw[color=orange] plot (\x,{0.05*exp(\x)}) node[right] {$f(x) = \frac{1}{20} \mathrm e^x$};
%\end{tikzpicture}
abc$abc\alpha\beta\gamma\omega$
\end{document}
        "#;

    let mut wasm_runner = WasmRunner::new()?;
    let svg_result = tex2svg(&mut wasm_runner, input);
    if svg_result.is_err() {
        let error = svg_result.unwrap_err();
        println!("Error: {}", error);
        // Show the log file
        let log_result = wasm_runner.get_messages()?;
        println!("input.log:\n{}", log_result);
        return Err(Error::msg(format!("Failed to convert DVI to SVG.",)));
    }

    println!("\n\nSVG OUTPUT\n\n{}", svg_result.unwrap());

    Ok(())
}
