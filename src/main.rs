use std::{collections::HashMap, io::Read};
// XXX: remove later
    use std::fs::File;
use std::io::Write;

use anyhow::Result;
use flate2::read::GzDecoder;
use tar::Archive;
use wasmi::*;

mod texjax_imports;
use texjax_imports::*;
mod filesystem;
use filesystem::*;

const TEX_FILE_BYTES: &[u8] = include_bytes!("./assets/tex_files.tar.gz");
const WASM_BYTES: &[u8] = include_bytes!("./assets/tex.wasm");
const CORE_BYTES: &[u8] = include_bytes!("./assets/core.dump");

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

fn main() -> Result<()> {
    // We have an in-memory file structure populated with the files that tex needs to run.
    // Extract these files to memory.
    let extracted_files = extract_tar_gz_to_memory(TEX_FILE_BYTES)?;
    let mut filesystem = VirtualFileSystem::new(extracted_files);
    filesystem.set_stdin(" input.tex \n\\end\n".as_bytes());

    // First step is to create the Wasm execution engine with some config.
    // In this example we are using the default configuration.
    let engine = Engine::default();
    //let wat = r#"
    //    (module
    //        (import "host" "hello" (func $host_hello (param i32)))
    //        (func (export "hello")
    //            (call $host_hello (i32.const 3))
    //        )
    //    )
    //"#;
    //// Wasmi does not yet support parsing `.wat` so we have to convert
    //// out `.wat` into `.wasm` before we compile and validate it.
    //let wasm = wat::parse_str(&wat)?;
    let module = Module::new(&engine, WASM_BYTES)?;

    // All Wasm objects operate within the context of a `Store`.
    // Each `Store` has a type parameter to store host-specific data,
    // which in this case we are using `42` for.
    type HostState = VirtualFileSystem;
    let mut store = Store::new(&engine, filesystem);
    //let host_hello = Func::wrap(&mut store, |caller: Caller<'_, HostState>, param: i32| {
    //    println!("Got {param} from WebAssembly");
    //    println!("My host state is: {}", caller.data());
    //});
    let memory = Memory::new(&mut store, MemoryType::new(1100, Some(1100))?)?;
    memory.write(&mut store, 0, CORE_BYTES)?;

    let imports = TexJaxImports::new(&mut store);

    // In order to create Wasm module instances and link their imports
    // and exports we require a `Linker`.
    // Create a linker and define all imports as no-op functions.
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
    let main_func = instance.get_typed_func::<(), ()>(&store, "main")?;
    main_func.call(&mut store, ())?;
    
    // Print stdout at this point so we know what happened.
    let stdout = store.data().get_stdout();
    println!("\n\nGOT THE FOLLOWING TEX OUTPUT:\n\n{}", stdout);
    
    let input_dvi = store.data().get_file_contents(FileType::Named("input.dvi"));
    let input_dvi_text = String::from_utf8_lossy(&input_dvi.unwrap());
    println!("\n\nGOT THE FOLLOWING DVI OUTPUT:\n\n{}", input_dvi_text);
    
    //let input_dvi = store.data().get_file_contents(FileType::Named("input.log"));
    //let input_dvi_text = String::from_utf8_lossy(&input_dvi.unwrap());
    //println!("\n\nGOT THE FOLLOWING LOG OUTPUT:\n\n{}", input_dvi_text);

    // Write the content of input_dvi to a file as bytes
    let mut file = std::fs::File::create("input.dvi")?;
    file.write_all(&input_dvi.unwrap())?;



    // Instantiation of a Wasm module requires defining its imports and then
    // afterwards we can fetch exports by name, as well as asserting the
    // type signature of the function with `get_typed_func`.
    //
    // Also before using an instance created this way we need to start it.
    //linker.define("host", "hello", host_hello)?;
    //let instance = linker.instantiate(&mut store, &module)?.start(&mut store)?;
    //let hello = instance.get_typed_func::<(), ()>(&store, "hello")?;

    //// And finally we can call the wasm!
    //hello.call(&mut store, ())?;

    Ok(())
}
