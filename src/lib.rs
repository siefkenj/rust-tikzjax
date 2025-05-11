// This library file exposes modules for testing purposes

pub mod filesystem;
pub use filesystem::{FilePointer, FileType, VirtualFileSystem};
pub mod texjax_imports;
pub use texjax_imports::TexJaxImports;

// Constants in lib instead of main.rs
pub const TEX_FILE_BYTES: &[u8] = include_bytes!("./assets/tex_files.tar.gz");
pub const WASM_BYTES: &[u8] = include_bytes!("./assets/tex.wasm");
pub const CORE_BYTES: &[u8] = include_bytes!("./assets/core.dump");

pub use crate::texjax_imports::clean_filename;
pub use crate::texjax_imports::read_memory;
