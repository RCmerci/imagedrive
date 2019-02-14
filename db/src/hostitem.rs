use sha2::{Digest, Sha256};
use std::fs::File;
use std::io;
use std::io::{copy, Seek, SeekFrom};
use std::path::Path;

#[derive(Debug)]
pub enum Error {
    BadPath(String),
    IOError(io::Error),
    DefaultError(String),
}

pub struct HostItem<'a> {
    path: &'a Path,
    id: String,
    fs: Vec<File>, // if item is file, only 1 elem in vec
}

impl<'a> HostItem<'a> {
    pub fn new(path: &'a Path, rename: Option<&str>) -> Result<Self, Error> {
        let filename;
        match path.file_name() {
            None => {
                return Err(Error::BadPath(path.display().to_string()));
            }
            Some(s) => filename = s.to_string_lossy().to_owned(),
        }
        let mut id = format!("{:?}", filename);
        if rename.is_some() {
            id = format!("{:?}", rename.unwrap())
        }
        let mut fs = vec![];
        // host file or dir
        if path.is_file() {
            let f = File::open(path).map_err(Error::IOError)?;
            fs.push(f);
        } else if path.is_dir() {
            for e in walkdir::WalkDir::new(path)
                .follow_links(true)
                .sort_by(|a, b| a.file_name().cmp(b.file_name()))
            {
                let entry = e.map_err(|err| Error::DefaultError(err.to_string()))?;
                if entry.path().is_dir() {
                    let f = File::open(entry.path()).map_err(Error::IOError)?;
                    fs.push(f);
                }
            }
        } else {
            return Err(Error::BadPath(path.display().to_string()));
        }
        Ok(HostItem { path, id, fs })
    }
}

impl<'a> crate::Item for HostItem<'a> {
    fn hash(&mut self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        for f in &mut self.fs {
            f.seek(SeekFrom::Start(0)).unwrap();
            copy(f, &mut hasher).unwrap();
        }
        hasher.result().to_vec()
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn srcpath(&self) -> &Path {
        self.path
    }
}
