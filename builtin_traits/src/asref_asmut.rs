use std::path::Path;

struct FileWrapper {
    path: String,
}

impl AsRef<Path> for FileWrapper {
    fn as_ref(&self) -> &Path {
        Path::new(&self.path)
    }
}

fn print_file_path<P: AsRef<Path>>(file: P) {
    println!("File path: {}", file.as_ref().display());
}

struct MutableWrapper {
    value: String,
}

impl AsMut<String> for MutableWrapper {
    fn as_mut(&mut self) -> &mut String {
        &mut self.value
    }
}

fn append_to_string<T: AsMut<String>>(mut target: T, text: &str) {
    target.as_mut().push_str(text);
}


fn test() {
    let file = FileWrapper {
        path: String::from("/path/to/file"),
    };

    print_file_path(file);
}