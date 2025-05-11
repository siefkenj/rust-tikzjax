use std::{cmp::max, cmp::min, collections::HashMap};

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

/// How a file is currently being read (in bytes mode, as raw data, or in
/// text mode). This is meant to mimic Pascal's read modes, even though all our
/// data is stored as bytes.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ReadMode {
    Bytes,
    Text,
}

impl VirtualFileSystem {
    /// Create a new virtual file system initialized with the files in `data`.
    pub fn new(data: HashMap<String, Vec<u8>>) -> Self {
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

    /// Get the bytes contained in `file`.
    pub fn get_file_contents(&self, file: FileType<&str>) -> Option<&Vec<u8>> {
        match file {
            FileType::Stdin => Some(&self.stdin),
            FileType::Stdout => Some(&self.stdout),
            FileType::Named(name) => self.data.get(name),
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
            if fp.byte_seek_position + data.len() > buffer.len() {
                buffer.resize(fp.byte_seek_position + data.len(), 0);
            }
            buffer.splice(
                fp.byte_seek_position..fp.byte_seek_position + data.len(),
                data.iter().cloned(),
            );
            fp.byte_seek_position += data.len();
        } else {
            println!(
                "write_to_file_by_index to fd {} but there is no corresponding file",
                fd
            );
        }
    }

    /// Read from the given file based on `fd`. The file pointer's position will be updated based on the
    /// number of bytes read.
    pub fn read_from_file_by_index(
        &mut self,
        fd: i32,
        length: usize,
        read_mode: ReadMode,
    ) -> Vec<u8> {
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
                min(fp.byte_seek_position, buffer.len() - 1)
            };
            let end = min(fp.byte_seek_position + length, buffer.len());
            let data = buffer[start..end].to_vec();
            match read_mode {
                ReadMode::Bytes => {
                    fp.byte_seek_position += data.len();
                }
                ReadMode::Text => {
                    fp.text_seek_position += data.len();
                }
            }
            data
        } else {
            println!(
                "read_from_file_by_index to fd {} but there is no corresponding file",
                fd
            );
            Vec::new()
        }
    }

    /// Returns whether the file pointer is at the end of the file.
    pub fn file_pointer_at_eof(&self, fp: &FilePointer) -> bool {
        match &fp.file {
            FileType::Stdin => {
                self.stdin.len() == 0
                    || fp.byte_seek_position >= self.stdin.len()
                    || fp.text_seek_position >= self.stdin.len()
            }
            FileType::Stdout => false,
            FileType::Named(name) => {
                let buffer = self.data.get(name).unwrap();
                // XXX: not sure if this is right? Should `ReadMode` be passed into this function?
                fp.byte_seek_position >= buffer.len() || fp.text_seek_position >= buffer.len()
            }
        }
    }

    /// Returns whether the file pointer is at the end of the line.
    /// This function is only relevant in text mode.
    pub fn file_pointer_at_eoln(&self, fp: &FilePointer) -> bool {
        self.file_pointer_at_eof(fp)
            || match &fp.file {
                FileType::Stdin => self.stdin.get(fp.text_seek_position) == Some(&b'\n'),
                FileType::Stdout => false,
                FileType::Named(name) => {
                    let buffer = self.data.get(name).unwrap();
                    buffer.get(max(fp.text_seek_position, 0)) == Some(&b'\n')
                }
            }
    }

    /// If the current `ReadMode::Text` file pointer is pointing to a newline, advance the pointer,
    /// otherwise do nothing.
    pub fn skip_current_newline_by_index(&mut self, fd: i32) {
        if fd < 0 {
            println!(
                "skip_current_newline_by_index to fd {} but fd is negative",
                fd
            );
            return;
        }
        if let Some(fp) = self.fd_to_file_pointer.get_mut(fd as usize) {
            let buffer = match fp.file {
                FileType::Stdin => &mut self.stdin,
                FileType::Stdout => &mut self.stdout,
                FileType::Named(ref name) => self.data.get_mut(name).unwrap(),
            };

            if let Some(&b'\n') = buffer.get(fp.text_seek_position) {
                // If the current character is a newline, advance the pointer
                fp.text_seek_position += 1;
            }
        } else {
            println!(
                "skip_current_newline_by_index to fd {} but there is no corresponding file",
                fd
            );
        }
    }

    /// Read a line starting from the current text position up until the end of the file/next newline.
    /// If the text position is already at the end of the file, return None.
    pub fn read_line_by_index(&mut self, fd: i32) -> Option<Vec<u8>> {
        if fd < 0 {
            println!("read_line_by_index to fd {} but fd is negative", fd);
            return None;
        }
        if let Some(fp) = self.fd_to_file_pointer.get_mut(fd as usize) {
            let buffer = match fp.file {
                FileType::Stdin => &mut self.stdin,
                FileType::Stdout => &mut self.stdout,
                FileType::Named(ref name) => self.data.get_mut(name).unwrap(),
            };
            let start = fp.text_seek_position;
            let end = buffer
                .iter()
                .skip(start)
                .position(|&c| c == b'\n')
                .map(|i| i + start)
                .unwrap_or(buffer.len());
            if start >= buffer.len() {
                return None;
            }
            let data = buffer[start..end].to_vec();
            fp.text_seek_position = end + 1;
            Some(data)
        } else {
            println!(
                "read_line_by_index to fd {} but there is no corresponding file",
                fd
            );
            None
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
    /// Pointer to where in the current file we are reading via the `get(...)` function. I.e.,
    /// when information from the file is read as bytes, this pointer is advanced.
    pub byte_seek_position: usize,
    /// Pointer to where in the current file we are reading via the `input_ln(...)` function. When
    /// information from the file is read as text, this pointer is advanced.
    ///
    /// XXX: It is not completely clear why two pointers are needed, but this mirrors what the
    /// tikzjax Javascript code does.
    pub text_seek_position: usize,
    pub erstat: i32,
}

impl FilePointer {
    /// Create a new file pointer to stdin.
    fn new_stdin() -> Self {
        Self {
            file: FileType::Stdin,
            byte_seek_position: 0,
            text_seek_position: 0,
            erstat: 0,
        }
    }
    /// Create a new file pointer to stdout.
    fn new_stdout() -> Self {
        Self {
            file: FileType::Stdout,
            byte_seek_position: 0,
            text_seek_position: 0,
            erstat: 0,
        }
    }
    /// Create a new file point with the given name.
    fn new_named<T: Into<String>>(name: T) -> Self {
        Self {
            file: FileType::Named(name.into()),
            byte_seek_position: 0,
            text_seek_position: 0,
            erstat: 0,
        }
    }
    /// Create a new file point with the given name and set the erstat to 1.
    /// This is used when opening a file that does not exist.
    fn new_named_with_erstat<T: Into<String>>(name: T) -> Self {
        Self {
            file: FileType::Named(name.into()),
            byte_seek_position: 0,
            text_seek_position: 0,
            erstat: 1,
        }
    }
}
