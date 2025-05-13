//! Functions to set up a WASM runtime to run the TeX engine as well as compile TeX source to SVG.

use std::collections::HashMap;
use std::io::Read;

use crate::dvi2svg;
use anyhow::Error;
use anyhow::Result;
use flate2::read::GzDecoder;
use tar::Archive;
use wasmi::*;

use crate::filesystem::*;
use crate::texjax_imports::*;

const TEX_FILE_BYTES: &[u8] = include_bytes!("./assets/tex_files.tar.gz");
const WASM_BYTES: &[u8] = include_bytes!("./assets/tex.wasm");
const CORE_BYTES: &[u8] = include_bytes!("./assets/core.dump");

/// Holds the TeX engine and initialized `wasmr` runtime. This object stubs out all
/// of the system calls that the WASM-compiled TeX engine needs to run.
pub struct WasmRunner {
    store: Store<VirtualFileSystem>,
    instance: Instance,
    /// Whether the TeX engine has run or not.
    has_run: bool,
}

impl WasmRunner {
    /// Create a new WasmRunner with pre-loaded TeX core.
    pub fn new() -> Result<Self> {
        // We have an in-memory file structure populated with the files that tex needs to run.
        // Extract these files to memory.
        let mut extracted_files = extract_tar_gz_to_memory(TEX_FILE_BYTES)?;
        // Add `input.tex` to the in-memory file structure.
        // This is the file that TeX will execute.
        //extracted_files.insert("input.tex".to_string(), b"\n\\begin{document}\n\\begin{tikzpicture}\n\\draw (0,0) circle (1in);\n\\end{tikzpicture}\n\\color{blue}$x^2$\n\nfoo\\par This is very cool!\\end{document}".to_vec());
        extracted_files.insert(
            "input.tex".to_string(),
            "\n\\begin{document}Hello World\\end{document}"
                .as_bytes()
                .to_vec(),
        );
        let mut filesystem = VirtualFileSystem::new(extracted_files);
        filesystem.set_stdin(" input.tex \n\\end\n".as_bytes());

        // First step is to create the Wasm execution engine with some config.
        // In this example we are using the default configuration.
        let engine = Engine::default();
        let module = Module::new(&engine, WASM_BYTES)?;

        // All Wasm objects operate within the context of a `Store`.
        // Each `Store` has a type parameter to store host-specific data.
        type HostState = VirtualFileSystem;
        let mut store = Store::new(&engine, filesystem);
        // 1100 pages is taken from the tikzjax Javascript code.
        let memory = Memory::new(&mut store, MemoryType::new(1100, Some(1100))?)?;
        memory.write(&mut store, 0, CORE_BYTES)?;

        let imports = TexJaxImports::new(&mut store);

        // Create a linker and define all imports as coming from our rust library.
        let mut linker = <Linker<HostState>>::new(&engine);
        linker.define("library", "printInteger", imports.print_integer)?;
        linker.define("library", "printChar", imports.print_char)?;
        linker.define("library", "printString", imports.print_string)?;
        linker.define("library", "printNewline", imports.print_newline)?;
        linker.define("library", "reset", imports.reset)?;
        linker.define("library", "inputln", imports.input_ln)?;
        linker.define("library", "rewrite", imports.rewrite)?;
        linker.define("library", "get", imports.get)?;
        linker.define("library", "put", imports.put)?;
        linker.define("library", "eof", imports.eof)?;
        linker.define("library", "eoln", imports.eoln)?;
        linker.define("library", "erstat", imports.erstat)?;
        linker.define("library", "close", imports.close)?;
        linker.define("library", "getCurrentMinutes", imports.get_current_minutes)?;
        linker.define("library", "getCurrentDay", imports.get_current_day)?;
        linker.define("library", "getCurrentMonth", imports.get_current_month)?;
        linker.define("library", "getCurrentYear", imports.get_current_year)?;
        linker.define("library", "tex_final_end", imports.tex_final_end)?;
        linker.define("env", "memory", memory)?;

        // Execute the exported "main" function.
        let instance = linker.instantiate(&mut store, &module)?.start(&mut store)?;

        Ok(Self {
            store,
            instance,
            has_run: false,
        })
    }

    /// Set the input contents that will be processed by TeX.
    pub fn set_input(&mut self, input: &[u8]) {
        self.store
            .data_mut()
            .set_file_contents(FileType::Named("input.tex"), input);
        self.has_run = false;
    }

    /// Run the TeX engine. If all is successful, a string with the output will be returned.
    pub fn run(&mut self) -> Result<String> {
        if !self.has_run {
            self.has_run = true;
            // Execute the exported "main" function.
            let main_func = self
                .instance
                .get_typed_func::<(), ()>(&self.store, "main")?;
            main_func.call(&mut self.store, ())?;
        }
        // Get the raw DVI file
        let input_dvi = self
            .store
            .data()
            .get_file_contents(FileType::Named("input.dvi"))
            .ok_or(Error::msg(
                "Cannot find `input.dvi`. Maybe compilation failed?",
            ))?;
        let svg = dvi2svg(input_dvi)
            .map_err(|e| Error::msg(format!("Failed to convert DVI to SVG: {}", e.to_string())))?;

        Ok(svg)
    }

    /// Get the output that TeX wrote to stdout.
    pub fn get_messages(&self) -> Result<String> {
        if !self.has_run {
            return Err(Error::msg("TeX has not run yet."));
        }
        let stdout = self.store.data().get_stdout();
        Ok(stdout)
    }

    /// Get the log file that TeX wrote.
    pub fn get_log(&self) -> Result<String> {
        if !self.has_run {
            return Err(Error::msg("TeX has not run yet."));
        }
        let input_log = self
            .store
            .data()
            .get_file_contents(FileType::Named("input.log"))
            .ok_or(Error::msg(
                "Cannot find `input.log`. Maybe compilation failed?",
            ))?;
        let input_log_text = String::from_utf8_lossy(input_log);
        Ok(input_log_text.to_string())
    }
}

fn extract_tar_gz_to_memory(bytes: &[u8]) -> Result<HashMap<String, Vec<u8>>> {
    // Create a GzDecoder to decompress the .tar.gz file
    let gz_decoder = GzDecoder::new(bytes);

    // Create a tar archive from the decompressed data
    let mut archive = Archive::new(gz_decoder);

    // Extract the tar archive to memory
    let mut extracted_files = HashMap::new();
    for entry in archive.entries()? {
        let mut entry = entry?;
        let mut file_data = Vec::new();
        entry.read_to_end(&mut file_data)?;
        let file_name = entry.path()?.to_string_lossy().into_owned();
        // Trim off a leading "./"
        let file_name = file_name.trim_start_matches("./");
        if file_name.is_empty() {
            continue;
        }
        extracted_files.insert(file_name.to_string(), file_data);
    }

    Ok(extracted_files)
}

/// Convert a TeX string to SVG using the given [`WasmRunner`]. This function can be called
/// multiple times with the same [`WasmRunner`].
pub fn tex2svg(wasm_runner: &mut WasmRunner, input_str: &str) -> Result<String> {
    wasm_runner.set_input(input_str.as_bytes());
    let svg = wasm_runner.run()?;
    Ok(svg)
}
