use sha2::{Digest, Sha256};
use std::fs::File;
use std::io;
use std::io::{copy, Seek, SeekFrom};
use std::path::Path;
// use walkdir;
pub enum Error<T: std::error::Error> {
    BadPath,
    IOError(io::Error),
    DefaultError(T),
}

pub struct DirItem<'a> {
    path: &'a Path,
    fs: Vec<File>,
}

impl<'a> DirItem<'a> {
    pub fn new(path: &'a Path) -> Result<Self, Error<walkdir::Error>> {
        if path.is_dir() && path.exists() {
            return Err(Error::BadPath);
        }
        let mut fs = vec![];
        for e in walkdir::WalkDir::new(path)
            .follow_links(true)
            .sort_by(|a, b| a.file_name().cmp(b.file_name()))
        {
            let entry = e.map_err(Error::DefaultError)?;
            if !entry.path().is_dir() {
                let f = File::open(entry.path()).map_err(Error::IOError)?;
                fs.push(f);
            }
        }
        Ok(DirItem { path, fs })
    }
}

impl<'a> crate::Item for DirItem<'a> {
    fn hash(&mut self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        for f in &mut self.fs {
            f.seek(SeekFrom::Start(0)).unwrap();
            copy(f, &mut hasher).unwrap();
        }
        hasher.result().to_vec()
    }

    fn id(&self) -> String {
        format!("DIR: {:?}", self.path.file_name().unwrap())
    }
}
