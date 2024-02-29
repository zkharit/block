use std::{io::{self, Read, Seek}, fs::{File, OpenOptions}, path::Path};

// open a file for reading
pub fn open_file_read(path: &Path) -> Result<File, io::Error> {
    OpenOptions::new().read(true).open(path)
}

// open a file for writing
pub fn open_file_write(path: &Path) -> Result<File, io::Error> {
    OpenOptions::new().write(true).truncate(true).read(true).open(path)
}

// create a file FAIL if the file already exists
pub fn create_file_new(path: &Path) -> Result<File, io::Error> {
    OpenOptions::new().write(true).read(true).create_new(true).open(path)
}

// create a file regardless if the file exists or not
pub fn create_file(path: &Path) -> Result<File, io::Error> {
    OpenOptions::new().write(true).read(true).create(true).open(path)
}

// read file into string
pub fn read_file_from_beginning(mut file: File) -> Result<String, io::Error> {
    let mut string = String::new();

    let _ = file.seek(io::SeekFrom::Start(0));

    let _ = match file.read_to_string(&mut string) {
        Ok(_) => (),
        Err(error) => return Err(error)
    };

    Ok(string)
}
