use sha2::{Digest, Sha256};
use std::fs::File;
use std::io;
use std::io::{copy, Seek, SeekFrom};
use std::path::Path;

pub enum Error {
    BadPath,
    IOError(io::Error),
}

pub struct FileItem<'a> {
    path: &'a Path,
    f: File,
}

impl<'a> FileItem<'a> {
    pub fn new(path: &'a Path) -> Result<Self, Error> {
        if path.is_file() && path.exists() {
            return Err(Error::BadPath);
        }
        let f = File::open(path).map_err(Error::IOError)?;
        Ok(FileItem { path, f })
    }
}

impl<'a> crate::Item for FileItem<'a> {
    fn hash(&mut self) -> Vec<u8> {
        self.f.seek(SeekFrom::Start(0)).unwrap();
        let mut hasher = Sha256::new();
        copy(&mut self.f, &mut hasher).unwrap();
        hasher.result().to_vec()
    }
    fn id(&self) -> String {
        self.path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_owned()
            .to_string()
    }
}
