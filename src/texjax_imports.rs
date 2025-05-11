use wasmi::*;

use crate::{FileType, VirtualFileSystem};

/// All the functions that are imported by the TeXJax WebAssembly module.
/// These are created to mirror `library.js` from the original TeXJax project.
///
/// Many of these functions mimic their PASCAL equivalents (as used by the original TeX engine).
pub struct TexJaxImports {
    /// Close a file. Mimics Pascal's `close` function.
    ///
    /// Called with a file descriptor.
    pub close: Func,
    /// Mimics Pascal's `eof` function. Returns true if the current file pointer is at the end of the file.
    ///
    /// Called with a file descriptor.
    pub eof: Func,
    /// Mimics Pascal's `eoln` function. Returns true if the current file pointer is at the end of the line (ASCII character 10)
    /// or at the end of the file.
    ///
    /// Called with a file descriptor.
    pub eoln: Func,
    pub erstat: Func,
    pub get: Func,
    /// Return the current day. This value is hard-coded to allow for WASM compilation.
    pub get_current_day: Func,
    /// Return the current minutes. This value is hard-coded to allow for WASM compilation.
    pub get_current_minutes: Func,
    /// Return the current month. This value is hard-coded to allow for WASM compilation.
    pub get_current_month: Func,
    /// Return the current year. This value is hard-coded to allow for WASM compilation.
    pub get_current_year: Func,
    /// Recreation of TeX's `input_ln` function. However, global variables are passed in as arguments.
    pub input_ln: Func,
    pub print_char: Func,
    pub print_integer: Func,
    pub print_newline: Func,
    /// Print a string stored in memory to the file pointed to by the file descriptor.
    /// The strings are stored in TeX's internal memory format:
    ///  - `pointer` points to the first byte which is the length of the string.
    ///  - The string is stored in the next `length` bytes.
    pub print_string: Func,
    /// Open a file for reading. Mimics Pascal's `reset` function.
    ///
    /// Called with a reference to a file name. Returns a file descriptor.
    pub reset: Func,
    /// Open a file for writing. Mimics Pascal's `rewrite` function.
    ///
    /// Called with a reference to a file name. Returns a file descriptor.
    pub rewrite: Func,
    pub tex_final_end: Func,
    pub put: Func,
}

/// Read a specified number of bytes from the memory at the specified pointer.
pub fn read_memory(memory: &Memory, ctx: &impl AsContext, pointer: usize, length: u32) -> Vec<u8> {
    let mut buffer = vec![0u8; length as usize];
    let mut buf = [0u8; 1];
    for i in 0..(length as usize) {
        memory.read(ctx, pointer + i, &mut buf).unwrap();
        buffer[i] = buf[0];
    }
    buffer
}

impl TexJaxImports {
    pub fn new(store: &mut Store<VirtualFileSystem>) -> Self {
        Self {
            close: Func::wrap(&mut *store, |fd: i32| {
                println!("[close] {}", fd);
                // We don't need to close files, so this is a no-op.
            }),
            eof: Func::wrap(&mut *store, |mut caller: Caller<_>, fd: i32| -> i32 {
                let vfs: &mut VirtualFileSystem = caller.data_mut();
                let fp = vfs.get_file_pointer_by_index(fd).unwrap();
                println!("[eof] {:?}: {}", fp.file, vfs.file_pointer_at_eof(fp));
                vfs.file_pointer_at_eof(fp) as i32
            }),
            eoln: Func::wrap(&mut *store, |mut caller: Caller<_>, a: i32| -> i32 {
                let vfs: &mut VirtualFileSystem = caller.data_mut();
                let fp = vfs.get_file_pointer_by_index(a).unwrap();
                println!("[eoln] {:?}: {}", fp.file, vfs.file_pointer_at_eoln(fp));
                vfs.file_pointer_at_eoln(fp) as i32
            }),
            erstat: Func::wrap(&mut *store, |mut caller: Caller<_>, fd: i32| -> i32 {
                let vfs: &mut VirtualFileSystem = caller.data_mut();
                let fp = vfs.get_file_pointer_by_index(fd).unwrap();
                println!("[erstat] {} {} {:?}", fd, fp.erstat, fp.file);
                fp.erstat
            }),
            get: Func::wrap(
                &mut *store,
                |mut caller: Caller<_>, fd: i32, pointer: u32, length: u32| {
                    let mem = caller.get_export("0").unwrap().into_memory().unwrap();

                    let fp = {
                        let vfs: &mut VirtualFileSystem = caller.data_mut();
                        vfs.get_file_pointer_by_index(fd).unwrap()
                    }
                    .clone();
                    let file_contents = {
                        let vfs: &mut VirtualFileSystem = caller.data_mut();
                        vfs.read_from_file_by_index(fd, length as usize)
                    };
                    if file_contents.len() == 0 {
                        mem.write(&mut caller, pointer as usize, &[0])
                            .expect("Failed to write to memory");
                    } else {
                        mem.write(&mut caller, pointer as usize, &file_contents)
                            .expect("Failed to write to memory");
                    }

                    println!(
                        "[get] {} {} {} contents: {:?} {:?}",
                        fd, pointer, length, &file_contents, fp
                    );
                },
            ),
            get_current_day: Func::wrap(&mut *store, || -> i32 {
                println!("[get_current_day]");
                1
            }),
            get_current_minutes: Func::wrap(&mut *store, || -> i32 {
                println!("[get_current_minutes]");
                0
            }),
            get_current_month: Func::wrap(&mut *store, || -> i32 {
                println!("[get_current_month]");
                1
            }),
            get_current_year: Func::wrap(&mut *store, || -> i32 {
                println!("[get_current_year]");
                1970
            }),
            input_ln: Func::wrap(
                &mut *store,
                |mut caller: Caller<_>,
                 fd: i32,
                 bypass_eoln: i32,
                 buf_pointer: i32,
                 first_pointer: i32,
                 last_pointer: i32,
                 max_buf_stack_pointer: i32,
                 buf_size: i32|
                 -> i32 {
                    println!(
                        "[input_ln] {} {} {} {} {} {} {}",
                        fd,
                        bypass_eoln,
                        buf_pointer,
                        first_pointer,
                        last_pointer,
                        max_buf_stack_pointer,
                        buf_size,
                    );
                    let mem = caller.get_export("0").unwrap().into_memory().unwrap();
                    // Get the u32 stored in the `first_pointer` memory location.
                    let get_first = |caller: &Caller<VirtualFileSystem>| {
                        let first =
                            u8_to_u32(&read_memory(&mem, caller, first_pointer as usize, 4));
                        first
                    };
                    // Get the u32 stored in the `first_pointer` memory location.
                    let get_last = |caller: &Caller<VirtualFileSystem>| {
                        let last = u8_to_u32(&read_memory(&mem, caller, last_pointer as usize, 4));
                        last
                    };
                    //// Set the u32 stored in the `first_pointer` memory location.
                    let set_first = |first: u32, caller: &mut Caller<VirtualFileSystem>| {
                        mem.write(caller, first_pointer as usize, &first.to_ne_bytes())
                            .expect("Failed to write to memory");
                    };
                    // Set the u32 stored in the `last_pointer` memory location.
                    let set_last = |last: u32, caller: &mut Caller<VirtualFileSystem>| {
                        mem.write(caller, last_pointer as usize, &last.to_ne_bytes())
                            .expect("Failed to write to memory");
                    };

                    // dbg!((caller.data_mut() as &mut VirtualFileSystem)
                    //     .get_file_pointer_by_index(fd)
                    //     .unwrap());

                    // Get the byte at offset first_pointer and last_pointer from the memory
                    let first = get_first(&caller);
                    let last = first;
                    // Default last_pointer to first_pointer in case we need to bail early.
                    // cf. Matthew 19:30
                    set_last(last, &mut caller);

                    println!(
                        "   [input_ln] first[0] = {}; last[0] = {}",
                        get_first(&caller),
                        get_last(&caller)
                    );

                    // Bypassing the eoln character means stepping past any "\n". The TeX algorithm
                    // specifies to just advance the file pointer one.
                    let mut first_char = Vec::new();
                    if bypass_eoln != 0 {
                        first_char.extend_from_slice(
                            (caller.data_mut() as &mut VirtualFileSystem)
                                .read_from_file_by_index(fd, 1)
                                .as_slice(),
                        );
                    }
                    println!(
                        "   [input_ln] first_char: {:?} {:?}",
                        String::from_utf8_lossy(&first_char),
                        &first_char
                    );
                    if let Some(&b'\n') = first_char.last() {
                        first_char.clear();
                    }

                    let vfs: &mut VirtualFileSystem = caller.data_mut();
                    let file_len = vfs.get_length(fd);

                    // If we are at the end of the file, we need to return early.
                    if vfs.at_eof_by_index(fd) {
                        return 0;
                    }

                    // Figure out where the next newline is.
                    let newline_offset = vfs.next_newline_offset_by_index(fd).unwrap();
                    let current_file_position = vfs.get_file_pointer_by_index(fd).unwrap().position;
                    //dbg!("c", vfs.get_file_pointer_by_index(fd).unwrap().position);
                    let input_line = if newline_offset > current_file_position {
                        vfs.read_from_file_by_index(fd, newline_offset - current_file_position)
                    } else {
                        Vec::new()
                    };
                    let mut input_line = first_char
                        .into_iter()
                        .chain(input_line.into_iter())
                        .collect::<Vec<_>>();

                    //dbg!(String::from_utf8_lossy(&input_line));

                    let _first = get_first(&caller);
                    mem.write(
                        &mut caller,
                        (buf_pointer + _first as i32) as usize,
                        &input_line,
                    )
                    .expect("Failed to write to memory");
                    // All trailing spaces are removed from the input line.
                    while let Some(&b' ') = input_line.last() {
                        input_line.pop();
                    }

                    // `last` is supposed to point to the last non-space character in the buffer.
                    let last = first + input_line.len() as u32 + 1;
                    let last = if last >= file_len as u32 {
                        file_len as u32 - 1
                    } else {
                        last
                    };

                    {
                        let vfs: &mut VirtualFileSystem = caller.data_mut();
                        println!(
                            "   [input_ln] at_eof = {} file_len = {} (file {:?})",
                            vfs.at_eof_by_index(fd),
                            vfs.get_length(fd),
                            vfs.get_file_pointer_by_index(fd)
                        );
                        let byte_at_end_of_line = vfs.get_file_byte(fd, last as usize);
                        println!(
                            "   [input_ln] byte_at_end_of_file: {} {:?}",
                            byte_at_end_of_line,
                            String::from_utf8_lossy(&[byte_at_end_of_line])
                        );
                    }
                    set_last(last, &mut caller);

                    println!(
                        "   [input_ln] (string: '{}') (raw: {:?})",
                        String::from_utf8(input_line.clone()).unwrap(),
                        &input_line
                    );
                    println!(
                        "   [input_ln] first[0] = {}; last[0] = {}",
                        get_first(&caller),
                        get_last(&caller)
                    );
                    1
                },
            ),
            print_char: Func::wrap(&mut *store, |mut caller: Caller<_>, fd: i32, char: i32| {
                let vfs: &mut VirtualFileSystem = caller.data_mut();
                vfs.write_to_file_by_index(fd, &[char as u8]);
                //println!(
                //    "[print_char] {} {} {}",
                //    fd,
                //    char,
                //    String::from_utf8(vec![char as u8]).unwrap()
                //);
            }),
            print_integer: Func::wrap(&mut *store, |a: i32, b: i32| {
                println!("print_integer {} {}", a, b);
            }),
            print_newline: Func::wrap(&mut *store, |mut caller: Caller<_>, fd: i32| {
                let vfs: &mut VirtualFileSystem = caller.data_mut();
                println!(
                    "[print_newline] {} {:?}",
                    fd,
                    vfs.get_file_pointer_by_index(fd).map(|fp| &fp.file)
                );

                vfs.write_to_file_by_index(fd, b"\n");
            }),
            print_string: Func::wrap(
                &mut *store,
                |mut caller: Caller<_>, fd: i32, pointer: i32| {
                    let mem = caller.get_export("0").unwrap().into_memory().unwrap();
                    let str_len = *read_memory(&mem, &caller, pointer as usize, 1)
                        .first()
                        .unwrap();
                    let string = String::from_utf8(read_memory(
                        &mem,
                        &caller,
                        pointer as usize + 1,
                        str_len as u32,
                    ))
                    .unwrap();

                    println!(
                        "[print_string] {} {} {:?} {:?}",
                        fd,
                        pointer,
                        (caller.data() as &VirtualFileSystem)
                            .get_file_pointer_by_index(fd)
                            .unwrap()
                            .file,
                        string
                    );

                    // write to the correct file
                    let vfs: &mut VirtualFileSystem = caller.data_mut();
                    vfs.write_to_file_by_index(fd, string.as_bytes());
                },
            ),
            put: Func::wrap(&mut *store, |a: i32, b: i32, c: i32| {
                println!("put {} {} {}", a, b, c);
            }),
            reset: Func::wrap(
                &mut *store,
                |mut caller: Caller<_>, length: u32, pointer: u32| -> i32 {
                    let mem = caller.get_export("0").unwrap().into_memory().unwrap();
                    let file_name = read_memory(&mem, &caller, pointer as usize, length);
                    let file_name = String::from_utf8(file_name).unwrap();
                    let file_name = clean_filename(&file_name);

                    let file = match file_name {
                        "TTY:" => FileType::Stdin,
                        _ => FileType::Named(file_name),
                    };

                    println!(
                        "[reset] {length} {pointer} Requesting file '{file_name}' returned descriptor to {file:?}",
                    );

                    let vfs: &mut VirtualFileSystem = caller.data_mut();
                    vfs.get_file_descriptor(file, true) as i32
                },
            ),
            rewrite: Func::wrap(
                &mut *store,
                |mut caller: Caller<_>, length: u32, pointer: u32| -> u32 {
                    let mem = caller.get_export("0").unwrap().into_memory().unwrap();
                    let file_name = read_memory(&mem, &caller, pointer as usize, length);
                    let file_name = String::from_utf8(file_name).unwrap();
                    let file_name = clean_filename(&file_name);

                    let file = match file_name {
                        "TTY:" => FileType::Stdout,
                        _ => FileType::Named(file_name),
                    };

                    println!(
                        "[rewrite] Requesting file '{file_name}' returned descriptor to {file:?}"
                    );

                    let vfs: &mut VirtualFileSystem = caller.data_mut();
                    vfs.get_file_descriptor(file, false) as u32
                },
            ),
            tex_final_end: Func::wrap(&mut *store, || {
                // This is a no-op since we have no need to finalize anything.
                println!("[tex_final_end]");
            }),
        }
    }
}

/// Cleans up a file name by
///  - Trimming any trailing whitespace
///  - If the string is quoted, take just the contents of the quotes.
///  - If the string is in curly braces "{...}", take just the contents of the braces.
pub fn clean_filename(file_name: &str) -> &str {
    // Trim any trailing whitespace and if the string is quoted, take just the contents of the quotes.
    let file_name = file_name.trim();
    let file_name = if file_name.starts_with('"') {
        // Find first and last position of quote marks
        let first = file_name.find('"').unwrap();
        let last = file_name.rfind('"').unwrap();
        if last > first {
            &file_name[first + 1..last]
        } else {
            &file_name[first + 1..]
        }
    } else {
        file_name
    };

    // If the string is in curly braces "{...}", take just the contents of the braces.
    let file_name = if file_name.starts_with('{') {
        // Find first and last position of curly braces
        let first = file_name.find('{').unwrap();
        if let Some(last) = file_name.rfind('}') {
            if last > first {
                &file_name[first + 1..last]
            } else {
                &file_name[first + 1..]
            }
        } else {
            &file_name[first + 1..]
        }
    } else {
        file_name
    };

    let file_name = file_name.trim();

    // TeXformats:TEX.POOL is a special file where all of TeX's internal strings are supposed
    // to be stored. We will map this to a file called `tex.pool`.
    if file_name == "TeXformats:TEX.POOL" {
        return "tex.pool";
    }

    file_name
}

/// Convert a slice of 4 bytes into a u32.
fn u8_to_u32(bytes: &[u8]) -> u32 {
    let mut buf = [0u8; 4];
    buf.copy_from_slice(&bytes);
    u32::from_ne_bytes(buf)
}

#[cfg(test)]
mod tests {
    use super::super::{
        filesystem::{FileType, VirtualFileSystem},
        texjax_imports::{clean_filename, read_memory, TexJaxImports},
        CORE_BYTES,
    };
    use super::*;
    use std::collections::HashMap;
    use wasmi::{Memory, MemoryType, Store, StoreLimits};

    // Helper function to create a test memory
    fn create_test_memory() -> (Store<VirtualFileSystem>, Memory) {
        let mut files = HashMap::new();
        files.insert("test.txt".to_string(), b"Test content".to_vec());
        let fs = VirtualFileSystem::new(files);

        let engine = wasmi::Engine::default();
        let mut store = Store::new(&engine, fs);

        let memory = Memory::new(&mut store, MemoryType::new(100, Some(100)).unwrap()).unwrap();

        // Initialize some memory for tests
        memory.write(&mut store, 0, &[1, 2, 3, 4, 5]).unwrap();

        (store, memory)
    }

    #[test]
    fn test_clean_filename() {
        assert_eq!(clean_filename("test.tex"), "test.tex");
        assert_eq!(clean_filename("\"test.tex\""), "test.tex");
        assert_eq!(clean_filename("{test.tex}"), "test.tex");
    }

    #[test]
    fn test_read_memory() {
        let (mut store, memory) = create_test_memory();

        // writ a string length followed by string data
        memory
            .write(&mut store, 10, &[b'H', b'e', b'l', b'l', b'o'])
            .unwrap();

        // Test reading the string
        let result = read_memory(&memory, &mut store, 10, 5);
        assert_eq!(result, b"Hello");
    }

    #[test]
    fn test_texjax_imports_new() {
        let (mut store, _) = create_test_memory();

        // Test that TexJaxImports can be created without panic
        let imports = TexJaxImports::new(&mut store);

        // Verify all functions were created
        // assert!(imports.print_integer.);
        // assert!(imports.print_char.is_host());
        // assert!(imports.print_string.is_host());
        // assert!(imports.print_newline.is_host());
        // assert!(imports.reset.is_host());
        // assert!(imports.input_ln.is_host());
        // assert!(imports.rewrite.is_host());
        // assert!(imports.get.is_host());
        // assert!(imports.put.is_host());
        // assert!(imports.eof.is_host());
        // assert!(imports.eoln.is_host());
        // assert!(imports.erstat.is_host());
        // assert!(imports.close.is_host());
        // assert!(imports.get_current_minutes.is_host());
        // assert!(imports.get_current_day.is_host());
        // assert!(imports.get_current_month.is_host());
        // assert!(imports.get_current_year.is_host());
        // assert!(imports.tex_final_end.is_host());
    }

    // This is a test to verify file operations through the imports
    #[test]
    fn test_file_operations() {
        let mut files = HashMap::new();
        files.insert("test.txt".to_string(), b"Test content".to_vec());
        let fs = VirtualFileSystem::new(files);

        let engine = wasmi::Engine::default();
        let mut store = Store::new(&engine, fs);
        let imports = TexJaxImports::new(&mut store);

        // Test reset function (open for reading)
        let mut output_result = vec![];
        let _ = imports
            .reset
            .call(&mut store, &[10.into()], output_result.as_mut_slice())
            .unwrap();
        let fd = output_result[0].i32().unwrap();
        assert!(fd >= 0);

        //     // Test eof initially (should be false since we haven't read anything)
        //     let eof_result = imports.eof.call(&mut store, (fd,)).unwrap();
        //     let is_eof = eof_result.unwrap_i32();
        //     assert_eq!(is_eof, 0); // 0 means false in TeX

        //     // Test get function to read a character
        //     let get_result = imports.get.call(&mut store, (fd,)).unwrap();

        //     // After reading one character, get the position
        //     let fs = store.data();
        //     if let Some(fp) = fs.get_file_pointer_by_index(fd) {
        //         assert_eq!(fp.position, 1);
        //     } else {
        //         panic!("File pointer not found");
        //     }

        //     // Test reading to the end
        //     for _ in 1..12 {
        //         // "Test content" is 12 characters
        //         let _ = imports.get.call(&mut store, (fd,));
        //     }

        //     // Now we should be at EOF
        //     let eof_result = imports.eof.call(&mut store, (fd,)).unwrap();
        //     let is_eof = eof_result.unwrap_i32();
        //     assert_eq!(is_eof, 1); // 1 means true in TeX

        //     // Test close function
        //     let close_result = imports.close.call(&mut store, (fd,)).unwrap();
        //     assert!(close_result.unwrap_i32() >= 0);
    }
}
