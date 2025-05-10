use std::{cmp::min, collections::HashMap, fs::File};

/// A virtual file system that allows for opening, reading, and
/// writing files in memory.
#[derive(Debug, Clone)]
pub(crate) struct VirtualFileSystem {
    /// The files in the virtual file system.
    data: HashMap<String, Vec<u8>>,
    stdin: Vec<u8>,
    stdout: Vec<u8>,
    /// A mapping from file descriptors to file handles. This
    /// keeps track of open files, etc.
    fd_to_file_pointer: Vec<FilePointer>,
}

impl VirtualFileSystem {
    /// Create a new virtual file system initialized with the files in `data`.
    pub fn new(data: HashMap<String, Vec<u8>>) -> Self {
        let mut data = data;
        data.insert("input.tex".to_string(), b"\n\\begin{document}\n\\begin{tikzpicture}\n\\draw (0,0) circle (1in);\n\\end{tikzpicture}\n\\end{document}".to_vec());
        Self {
            data,
            stdin: Vec::new(),
            stdout: Vec::new(),
            fd_to_file_pointer: vec![],
        }
    }
    /// Get a file descriptor for the specified file. There is no
    /// way for this function to fail. If the file does not exist,
    /// a new file will be created with the corresponding name.
    ///
    /// This function always returns a file pointer that is initialized to position 0
    /// in the file.
    pub fn get_file_descriptor(&mut self, file: FileType<&str>, erstat_if_new: bool) -> usize {
        let file_pointer = match file {
            FileType::Stdin => FilePointer::new_stdin(),
            FileType::Stdout => FilePointer::new_stdout(),
            FileType::Named(name) => {
                let mut is_new_file = false;
                // Ensure there is some data for the file
                self.data.entry(name.to_string()).or_insert_with(|| {
                    is_new_file = true;
                    Vec::new()
                });
                if is_new_file && erstat_if_new {
                    FilePointer::new_named_with_erstat(name)
                } else {
                    FilePointer::new_named(name)
                }
            }
        };
        self.fd_to_file_pointer.push(file_pointer);
        self.fd_to_file_pointer.len() - 1
    }

    /// Get the file pointer referenced by index `fd`.
    pub fn get_file_pointer_by_index(&self, fd: i32) -> Option<&FilePointer> {
        if fd < 0 {
            None
        } else {
            self.fd_to_file_pointer.get(fd as usize)
        }
    }

    /// Get the file pointer referenced by index `fd`.
    pub fn get_file_pointer_by_index_mut(&mut self, fd: i32) -> Option<&mut FilePointer> {
        if fd < 0 {
            None
        } else {
            self.fd_to_file_pointer.get_mut(fd as usize)
        }
    }

    /// Write data to the file referenced by the file indexed by `fd`. If `fd` is negative, this function
    /// prints and exist.
    ///
    /// Data will be written starting at index `file.position` as stored by the file pointer.
    pub fn write_to_file_by_index(&mut self, fd: i32, data: &[u8]) {
        if fd < 0 {
            println!("write_to_file_by_index to fd {} but fd is negative", fd);
            return;
        }
        if let Some(fp) = self.fd_to_file_pointer.get_mut(fd as usize) {
            let buffer = match fp.file {
                FileType::Stdin => &mut self.stdin,
                FileType::Stdout => &mut self.stdout,
                FileType::Named(ref name) => self.data.get_mut(name).unwrap(),
            };
            // Write to `buffer` starting at `fp.position` but take care to add to the length of the buffer if
            // we need to write past the end of the buffer.
            if fp.position + data.len() > buffer.len() {
                buffer.resize(fp.position + data.len(), 0);
            }
            buffer.splice(fp.position..fp.position + data.len(), data.iter().cloned());
            fp.position += data.len();
        } else {
            println!(
                "write_to_file_by_index to fd {} but there is no corresponding file",
                fd
            );
        }
    }

    /// Read from the given file based on `fd`. The file pointer's position will be updated based on the
    /// number of bytes read.
    pub fn read_from_file_by_index(&mut self, fd: i32, length: usize) -> Vec<u8> {
        if fd < 0 {
            println!("read_from_file_by_index to fd {} but fd is negative", fd);
            return Vec::new();
        }
        if let Some(fp) = self.fd_to_file_pointer.get_mut(fd as usize) {
            let buffer = match fp.file {
                FileType::Stdin => &mut self.stdin,
                FileType::Stdout => &mut self.stdout,
                FileType::Named(ref name) => self.data.get_mut(name).unwrap(),
            };
            let start = if buffer.len() == 0 {
                0
            } else {
                min(fp.position, buffer.len() - 1)
            };
            let end = min(fp.position + length, buffer.len());
            let data = buffer[start..end].to_vec();
            fp.position += data.len();
            data
        } else {
            println!(
                "read_from_file_by_index to fd {} but there is no corresponding file",
                fd
            );
            Vec::new()
        }
    }

    /// Read a byte at a specific offset of a file specified by `fd`. The file pointer's position will _not_ be updated.
    pub fn get_file_byte(&mut self, fd: i32, offset: usize) -> u8 {
        if fd < 0 {
            panic!("read_from_file_by_index to fd {} but fd is negative", fd);
        }
        if let Some(fp) = self.fd_to_file_pointer.get_mut(fd as usize) {
            let buffer = match fp.file {
                FileType::Stdin => &mut self.stdin,
                FileType::Stdout => &mut self.stdout,
                FileType::Named(ref name) => self.data.get_mut(name).unwrap(),
            };
            let data = buffer[offset];
            data
        } else {
            println!(
                "read_from_file_by_index to fd {} but there is no corresponding file",
                fd
            );
            0
        }
    }

    /// Return the current length (in bytes) of the file referenced by `fd`.
    pub fn get_length(&mut self, fd: i32) -> usize {
        if fd < 0 {
            println!("read_from_file_by_index to fd {} but fd is negative", fd);
            return 0;
        }
        if let Some(fp) = self.fd_to_file_pointer.get_mut(fd as usize) {
            let buffer = match fp.file {
                FileType::Stdin => &mut self.stdin,
                FileType::Stdout => &mut self.stdout,
                FileType::Named(ref name) => self.data.get_mut(name).unwrap(),
            };
            buffer.len()
        } else {
            println!(
                "read_from_file_by_index to fd {} but there is no corresponding file",
                fd
            );
            0
        }
    }

    /// Returns whether the file pointer is at the end of the file.
    pub fn at_eof_by_index(&self, fd: i32) -> bool {
        if fd < 0 {
            println!("at_eof_by_index to fd {} but fd is negative", fd);
            return true;
        }
        if let Some(fp) = self.fd_to_file_pointer.get(fd as usize) {
            self.file_pointer_at_eof(fp)
        } else {
            println!(
                "at_eof_by_index to fd {} but there is no corresponding file",
                fd
            );
            true
        }
    }

    /// Returns whether the file pointer is at the end of the file.
    pub fn file_pointer_at_eof(&self, fp: &FilePointer) -> bool {
        match &fp.file {
            FileType::Stdin => self.stdin.len() == 0 || fp.position >= self.stdin.len(),
            FileType::Stdout => false,
            FileType::Named(name) => {
                let buffer = self.data.get(name).unwrap();
                println!("      -- checking if at eof: {} >= {}  ({:?})", fp.position, buffer.len(), fp);
                fp.position >= buffer.len()
            }
        }
    }

    /// Returns whether the file pointer is at the end of the line.
    pub fn file_pointer_at_eoln(&self, fp: &FilePointer) -> bool {
        self.file_pointer_at_eof(fp)
            || match &fp.file {
                FileType::Stdin => self.stdin.get(fp.position) == Some(&b'\n'),
                FileType::Stdout => false,
                FileType::Named(name) => {
                    let buffer = self.data.get(name).unwrap();
                    buffer.get(fp.position) == Some(&b'\n')
                }
            }
    }

    /// Find the location of the next "\n" in the file starting at the current position of the file pointer.
    /// If the file ends, the ending position of the buffer is returned.
    pub fn next_newline_offset_by_index(&self, fd: i32) -> Option<usize> {
        if fd < 0 {
            println!(
                "next_newline_offset_by_index to fd {} but fd is negative",
                fd
            );
            return None;
        }
        if let Some(fp) = self.fd_to_file_pointer.get(fd as usize) {
            let buffer = match fp.file {
                FileType::Stdin => &self.stdin,
                FileType::Stdout => &self.stdout,
                FileType::Named(ref name) => self.data.get(name).unwrap(),
            };
            if fp.position >= buffer.len() {
                return Some(usize::max(buffer.len(), 1) - 1);
            }
            return buffer
                .iter()
                .skip(fp.position)
                .position(|&c| c == b'\n')
                .map(|i| i + fp.position)
                .or(Some(buffer.len()));
        } else {
            println!(
                "next_newline_offset_by_index to fd {} but there is no corresponding file",
                fd
            );
            None
        }
    }

    pub fn advance_file_pointer(&mut self, fd: i32, offset: usize) {
        if fd < 0 {
            println!("advance_file_pointer to fd {} but fd is negative", fd);
            return;
        }
        if let Some(fp) = self.fd_to_file_pointer.get_mut(fd as usize) {
            fp.position += offset;
        } else {
            println!(
                "advance_file_pointer to fd {} but there is no corresponding file",
                fd
            );
        }
    }

    /// Return the contents of stdout as a string.
    pub fn get_stdout(&self) -> String {
        String::from_utf8_lossy(&self.stdout).to_string()
    }

    /// Set the contents of stdin.
    pub fn set_stdin(&mut self, data: &[u8]) {
        self.stdin = data.to_vec();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum FileType<T> {
    Stdin,
    Stdout,
    Named(T),
}

/// A `FilePointer` is an object that keeps track of the name of a file and the current read position
/// in the file. The actual contents of the file is stored in a different place.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FilePointer {
    pub file: FileType<String>,
    /// Pointer to where in the current file we are reading
    pub position: usize,
    pub erstat: i32,
}

impl FilePointer {
    /// Create a new file pointer to stdin.
    fn new_stdin() -> Self {
        Self {
            file: FileType::Stdin,
            position: 0,
            erstat: 0,
        }
    }
    /// Create a new file pointer to stdout.
    fn new_stdout() -> Self {
        Self {
            file: FileType::Stdout,
            position: 0,
            erstat: 0,
        }
    }
    /// Create a new file point with the given name.
    fn new_named<T: Into<String>>(name: T) -> Self {
        Self {
            file: FileType::Named(name.into()),
            position: 0,
            erstat: 0,
        }
    }
    /// Create a new file point with the given name and set the erstat to 1.
    /// This is used when opening a file that does not exist.
    fn new_named_with_erstat<T: Into<String>>(name: T) -> Self {
        Self {
            file: FileType::Named(name.into()),
            position: 0,
            erstat: 1,
        }
    }
}
