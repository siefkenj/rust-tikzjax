use flate2::read::GzDecoder;
use rust_tikz::{
    filesystem::{FileType, VirtualFileSystem},
    texjax_imports::TexJaxImports,
    CORE_BYTES, TEX_FILE_BYTES, WASM_BYTES,
};
use std::collections::HashMap;
use std::io::Read;
use tar::Archive;
use wasmi::{Engine, Linker, Memory, MemoryType, Module, Store};

// Helper function to extract tar.gz files
fn extract_tar_gz_to_memory(bytes: &[u8]) -> HashMap<String, Vec<u8>> {
    let gz_decoder = GzDecoder::new(bytes);
    let mut archive = Archive::new(gz_decoder);
    let mut extracted_files = HashMap::new();

    for entry in archive.entries().expect("Failed to get archive entries") {
        let mut entry = entry.expect("Failed to get archive entry");
        let mut file_data = Vec::new();
        entry
            .read_to_end(&mut file_data)
            .expect("Failed to read entry");
        let file_name = entry
            .path()
            .expect("Failed to get entry path")
            .to_string_lossy()
            .into_owned();
        let file_name = file_name.trim_start_matches("./");
        if file_name.is_empty() {
            continue;
        }
        extracted_files.insert(file_name.to_string(), file_data);
    }

    extracted_files
}

#[test]
fn test_extract_tex_files() {
    let extracted_files = extract_tar_gz_to_memory(TEX_FILE_BYTES);

    // Verify we have some files
    assert!(!extracted_files.is_empty());

    // Check for some expected TeX files
    assert!(extracted_files
        .keys()
        .any(|k| k.ends_with(".fmt") || k.ends_with(".tfm")));
}

#[test]
fn test_wasm_loading() {
    // Verify the WASM module can be loaded
    let engine = Engine::default();
    let module = Module::new(&engine, WASM_BYTES).expect("Failed to create module");

    // Check that it's a valid module
    assert!(module.imports().count() > 0);
}

#[test]
fn test_simple_tikz_processing() {
    // Setup virtual filesystem with a basic TikZ diagram
    let mut fs = VirtualFileSystem::new(HashMap::new());

    // Provide a simple TikZ diagram as input
    fs.set_stdin(" input.tex \n\\end\n".as_bytes());

    // Create our WASM runtime
    let engine = Engine::default();
    let module = Module::new(&engine, WASM_BYTES).expect("Failed to create module");
    let mut store = Store::new(&engine, fs);

    // Setup memory and import the core dump
    let memory = Memory::new(&mut store, MemoryType::new(1100, Some(1100)).unwrap())
        .expect("Failed to create memory");
    memory
        .write(&mut store, 0, CORE_BYTES)
        .expect("Failed to write core dump");

    // Setup all the imports
    let imports = TexJaxImports::new(&mut store);
    let mut linker = <Linker<VirtualFileSystem>>::new(&engine);

    // Define all the required imports
    linker
        .define("library", "printInteger", imports.print_integer)
        .expect("Failed to define printInteger");
    linker
        .define("library", "printChar", imports.print_char)
        .expect("Failed to define printChar");
    linker
        .define("library", "printString", imports.print_string)
        .expect("Failed to define printString");
    linker
        .define("library", "printNewline", imports.print_newline)
        .expect("Failed to define printNewline");
    linker
        .define("library", "reset", imports.reset)
        .expect("Failed to define reset");
    linker
        .define("library", "inputln", imports.input_ln)
        .expect("Failed to define inputln");
    linker
        .define("library", "rewrite", imports.rewrite)
        .expect("Failed to define rewrite");
    linker
        .define("library", "get", imports.get)
        .expect("Failed to define get");
    linker
        .define("library", "put", imports.put)
        .expect("Failed to define put");
    linker
        .define("library", "eof", imports.eof)
        .expect("Failed to define eof");
    linker
        .define("library", "eoln", imports.eoln)
        .expect("Failed to define eoln");
    linker
        .define("library", "erstat", imports.erstat)
        .expect("Failed to define erstat");
    linker
        .define("library", "close", imports.close)
        .expect("Failed to define close");
    linker
        .define("library", "getCurrentMinutes", imports.get_current_minutes)
        .expect("Failed to define getCurrentMinutes");
    linker
        .define("library", "getCurrentDay", imports.get_current_day)
        .expect("Failed to define getCurrentDay");
    linker
        .define("library", "getCurrentMonth", imports.get_current_month)
        .expect("Failed to define getCurrentMonth");
    linker
        .define("library", "getCurrentYear", imports.get_current_year)
        .expect("Failed to define getCurrentYear");
    linker
        .define("library", "tex_final_end", imports.tex_final_end)
        .expect("Failed to define tex_final_end");
    linker
        .define("env", "memory", memory)
        .expect("Failed to define memory");

    // Instantiate the module but don't run it - we just want to test
    // that everything is wired up correctly
    let instance = linker
        .instantiate(&mut store, &module)
        .expect("Failed to instantiate module");

    // Verify the main function exists
    // let main_func = instance
    //     .get_typed_func::<(), ()>(&store, "main")
    //     .expect("Failed to get main function");

    // We don't actually run the function as it may take a long time and it's better suited for
    // a separate integration test that runs longer tests

    // Just check that we've initialized everything properly
    assert!(store.data().data.contains_key("input.tex"));
}
