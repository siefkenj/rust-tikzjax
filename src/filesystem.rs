use std::{cmp::min, collections::HashMap};

/// A virtual file system that allows for opening, reading, and
/// writing files in memory.
#[derive(Debug, Clone)]
pub struct VirtualFileSystem {
    /// The files in the virtual file system.
    pub data: HashMap<String, Vec<u8>>,
    pub stdin: Vec<u8>,
    pub stdout: Vec<u8>,
    /// A mapping from file descriptors to file handles. This
    /// keeps track of open files, etc.
    pub fd_to_file_pointer: Vec<FilePointer>,
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
                println!(
                    "      -- checking if at eof: {} >= {}  ({:?})",
                    fp.position,
                    buffer.len(),
                    fp
                );
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
pub enum FileType<T> {
    Stdin,
    Stdout,
    Named(T),
}

/// A `FilePointer` is an object that keeps track of the name of a file and the current read position
/// in the file. The actual contents of the file is stored in a different place.
#[derive(Debug, Clone, PartialEq)]
pub struct FilePointer {
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{FileType, VirtualFileSystem};

    #[test]
    fn test_virtual_filesystem_creation() {
        let mut files = HashMap::new();
        files.insert("test.txt".to_string(), b"Hello, world!".to_vec());
        let mut fs = VirtualFileSystem::new(files);

        // Check that the file system was created with our file plus the default input.tex
        // by opening the files and checking if we get valid file descriptors
        let test_fd = fs.get_file_descriptor(FileType::Named("test.txt"), false);
        let input_fd = fs.get_file_descriptor(FileType::Named("input.tex"), false);

        // Verify we can read from both files
        let test_content = fs.read_from_file_by_index(test_fd as i32, 13);
        let input_content = fs.read_from_file_by_index(input_fd as i32, 10); // Just read first 10 bytes

        assert_eq!(test_content, b"Hello, world!");
        assert!(!input_content.is_empty()); // Just check there's some content
    }

    #[test]
    fn test_file_descriptor_management() {
        let mut fs = VirtualFileSystem::new(HashMap::new());

        // Get file descriptors for different file types
        let stdin_fd = fs.get_file_descriptor(FileType::Stdin, false);
        let stdout_fd = fs.get_file_descriptor(FileType::Stdout, false);
        let named_fd = fs.get_file_descriptor(FileType::Named("example.txt"), false);

        // Check that they're all different
        assert_ne!(stdin_fd, stdout_fd);
        assert_ne!(stdin_fd, named_fd);
        assert_ne!(stdout_fd, named_fd);

        // Check that we can retrieve the file pointers
        assert!(fs.get_file_pointer_by_index(stdin_fd as i32).is_some());
        assert!(fs.get_file_pointer_by_index(stdout_fd as i32).is_some());
        assert!(fs.get_file_pointer_by_index(named_fd as i32).is_some());

        // Check that invalid indices return None
        assert!(fs.get_file_pointer_by_index(-1).is_none());
        assert!(fs
            .get_file_pointer_by_index((named_fd + 1) as i32)
            .is_none());
    }

    #[test]
    fn test_file_reading_and_writing() {
        let mut fs = VirtualFileSystem::new(HashMap::new());

        // Create and write to a file
        let fd = fs.get_file_descriptor(FileType::Named("test.txt"), false);
        fs.write_to_file_by_index(fd as i32, b"Hello, world!");

        // Reset position
        if let Some(fp) = fs.get_file_pointer_by_index_mut(fd as i32) {
            fp.position = 0;
        }

        // Read back from the file
        let data = fs.read_from_file_by_index(fd as i32, 13);
        assert_eq!(data, b"Hello, world!");

        // Check we're at EOF now
        assert!(fs.at_eof_by_index(fd as i32));
    }

    #[test]
    fn test_stdin_stdout() {
        let mut fs = VirtualFileSystem::new(HashMap::new());

        // Set stdin content
        fs.set_stdin(b"Test input\nSecond line");

        // Get stdin file descriptor and read from it
        let stdin_fd = fs.get_file_descriptor(FileType::Stdin, false);
        let data = fs.read_from_file_by_index(stdin_fd as i32, 10);
        assert_eq!(data, b"Test input");

        // Get stdout file descriptor and write to it
        let stdout_fd = fs.get_file_descriptor(FileType::Stdout, false);
        fs.write_to_file_by_index(stdout_fd as i32, b"Test output");

        // Check stdout content
        assert_eq!(fs.get_stdout(), "Test output");
    }

    // #[test]
    // fn test_eof_and_eoln() {
    //     let mut fs = VirtualFileSystem::new(HashMap::new());

    //     // Create a file with multiple lines
    //     let fd = fs.get_file_descriptor(FileType::Named("multiline.txt"), false);
    //     fs.write_to_file_by_index(fd as i32, b"Line 1\nLine 2\nLine 3");

    //     // Reset position
    //     if let Some(fp) = fs.get_file_pointer_by_index_mut(fd as i32) {
    //         fp.position = 0;
    //     }

    //     // Check eoln at the end of first line
    //     if let Some(fp) = fs.get_file_pointer_by_index_mut(fd as i32) {
    //         fp.position = 6; // Position at the newline after "Line 1"
    //         assert!(fs.file_pointer_at_eoln(fp));

    //         // Not at EOF yet
    //         assert!(!fs.file_pointer_at_eof(fp));

    //         // Move to the end of the file
    //         fp.position = 20; // Beyond the end of our data
    //         assert!(fs.file_pointer_at_eof(fp));
    //         assert!(fs.file_pointer_at_eoln(fp)); // EOLN is true at EOF
    //     }
    // }

    #[test]
    fn test_next_newline_offset() {
        let mut fs = VirtualFileSystem::new(HashMap::new());

        // Create a file with multiple lines
        let fd = fs.get_file_descriptor(FileType::Named("multiline.txt"), false);
        fs.write_to_file_by_index(fd as i32, b"Line 1\nLine 2\nLine 3");

        // Reset position
        if let Some(fp) = fs.get_file_pointer_by_index_mut(fd as i32) {
            fp.position = 0;
        }

        // Find the next newline
        let offset = fs.next_newline_offset_by_index(fd as i32);
        assert_eq!(offset, Some(6)); // After "Line 1"

        // Advance past the newline
        fs.advance_file_pointer(fd as i32, 7); // To the start of "Line 2"

        // Find the next newline
        let offset = fs.next_newline_offset_by_index(fd as i32);
        assert_eq!(offset, Some(13)); // After "Line 2"
    }

    #[test]
    fn test_erstat() {
        let mut fs = VirtualFileSystem::new(HashMap::new());

        // Create file with erstat = true (file doesn't exist)
        let fd = fs.get_file_descriptor(FileType::Named("nonexistent.txt"), true);

        // Check erstat is set to 1
        if let Some(fp) = fs.get_file_pointer_by_index(fd as i32) {
            assert_eq!(fp.erstat, 1);
        }

        // Create file with erstat = false
        let fd2 = fs.get_file_descriptor(FileType::Named("existing.txt"), false);

        // Check erstat is not set
        if let Some(fp) = fs.get_file_pointer_by_index(fd2 as i32) {
            assert_eq!(fp.erstat, 0);
        }
    }

    #[test]
    fn test_get_file_byte() {
        let mut fs = VirtualFileSystem::new(HashMap::new());

        // Create a file with some content
        let fd = fs.get_file_descriptor(FileType::Named("test.txt"), false);
        fs.write_to_file_by_index(fd as i32, b"ABCDEF");

        // Get a byte at a specific position
        let byte = fs.get_file_byte(fd as i32, 2);
        assert_eq!(byte, b'C');
    }

    #[test]
    fn test_file_length() {
        let mut fs = VirtualFileSystem::new(HashMap::new());

        // Create a file with some content
        let fd = fs.get_file_descriptor(FileType::Named("test.txt"), false);
        fs.write_to_file_by_index(fd as i32, b"Test content");

        // Check file length
        let length = fs.get_length(fd as i32);
        assert_eq!(length, 12); // "Test content" is 12 bytes
    }
}
