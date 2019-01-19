use crate::*;
use std::fmt;
use std::ops::DerefMut;
use std::path::Path;
#[derive(Debug)]
pub enum Error {
    NotFoundEntry(String),
    DockerError(dockerclient::Error),
    ExecError(String),
    FileItemError(file_item::Error),
    DirItemError(dir_item::Error),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "imagedrive error")
    }
}

pub struct ImageDrive {
    image_name: String,
    dockercli: dockerclient::DockerClient,
}

impl ImageDrive {
    pub fn new(image_name: &str, server: &str, username: &str, password: &str) -> ImageDrive {
        let dockercli = dockerclient::DockerClient::new_with_logininfo(server, username, password);
        ImageDrive {
            image_name: image_name.to_string(),
            dockercli: dockercli,
        }
    }
}

impl fmt::Display for ImageDrive {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "imagedrive: {}", self.image_name)
    }
}

impl DB<Error> for ImageDrive {
    fn entries(&self) -> Result<Vec<String>, Error> {
        ls(&self.dockercli, &self.image_name, &Path::new("/data"))
    }
    fn items(&self, entry: &str) -> Result<Vec<String>, Error> {
        ls(
            &self.dockercli,
            &self.image_name,
            &Path::new("/data").join(entry),
        )
    }

    fn add(&self, entry: &str, itempath: &Path) -> Result<AddResult, Error> {
        let mut item: Box<Item> = if itempath.is_file() {
            Box::new(file_item::FileItem::new(itempath).map_err(Error::FileItemError)?)
        } else {
            Box::new(dir_item::DirItem::new(itempath).map_err(Error::DirItemError)?)
        };

        let c = get_or_run(&self.dockercli, &self.image_name)?;
        let path = Path::new("/data").join(entry);
        let ls_entry = format!(
            "mkdir -p {} && ls {}",
            path.as_path().display(),
            path.as_path().display()
        );
        self.dockercli
            .exec(&c.id, &ls_entry)
            .map_err(Error::DockerError)
            .map(|(out, err)| {
                (
                    (String::from_utf8_lossy(&out).to_owned().to_string()),
                    (String::from_utf8_lossy(&err).to_owned().to_string()),
                )
            })
            .and_then(|(out, err)| {
                if err != "" {
                    return Err(Error::ExecError(err));
                }
                let items: Vec<&str> = if out.trim() == "" {
                    vec![]
                } else {
                    out.trim().split(" ").collect()
                };
                let mut ps = vec![];
                let mut fileitems = vec![];
                let mut diritems = vec![];
                for item in items {
                    ps.push(Path::new("/data").join(item));
                }
                for p in &ps {
                    if p.is_file() {
                        fileitems.push(file_item::FileItem::new(&p).map_err(Error::FileItemError)?);
                    } else {
                        diritems.push(dir_item::DirItem::new(&p).map_err(Error::DirItemError)?);
                    };
                }

                for mut fi in fileitems {
                    if compare_items(&mut fi, item.deref_mut()) {
                        return Ok(AddResult::ExistedItem(fi.id().into()));
                    }
                }
                for mut di in diritems {
                    if compare_items(&mut di, item.deref_mut()) {
                        return Ok(AddResult::ExistedItem(di.id().into()));
                    }
                }
                let dstpath = Path::new("/data").join(entry).join(item.id());
                self.dockercli
                    .copy_in(&c.id, item.srcpath(), &dstpath)
                    .map_err(Error::DockerError)?;
                Ok(AddResult::Succ)
            })
    }
    fn delete(&self, entry: &str, itempath: &Path) -> Result<(), Error> {
        let item: Box<Item> = if itempath.is_file() {
            Box::new(file_item::FileItem::new(itempath).map_err(Error::FileItemError)?)
        } else {
            Box::new(dir_item::DirItem::new(itempath).map_err(Error::DirItemError)?)
        };

        let c = get_or_run(&self.dockercli, &self.image_name)?;
        let dstpath = Path::new("/data").join(entry).join(item.id());
        self.dockercli
            .remove_file(&c.id, &dstpath)
            .map_err(|e| Error::ExecError(format!("{:?}", e)))
    }

    fn export_to_dir(&self, dir: &Path, entry: &str) -> Result<(), Error> {
        let c = get_or_run(&self.dockercli, &self.image_name)?;
        let srcpath = Path::new("/data").join(entry);
        self.dockercli
            .copy_out(&c.id, &srcpath, dir)
            .map_err(Error::DockerError)
    }

    fn sync(&self) -> Result<(), Error> {
        // TODO: check image exist, if not , pull from registry

        // image existed, so push to registry
        self.dockercli
            .push(&self.image_name)
            .map_err(Error::DockerError)
    }
}

fn get_or_run(
    cli: &dockerclient::DockerClient,
    image: &str,
) -> Result<dockerclient::Container, Error> {
    let cs = cli.ps(false).map_err(Error::DockerError)?;
    for c in cs {
        if c.image.split(":").collect::<Vec<&str>>()[0]
            == image.split(":").collect::<Vec<&str>>()[0]
        {
            return Ok(c);
        }
    }
    // not found
    let c = cli.create(image).map_err(Error::DockerError)?;
    cli.start(&c.id).map_err(Error::DockerError)?;
    Ok(c)
}

fn ls(cli: &dockerclient::DockerClient, image: &str, dir: &Path) -> Result<Vec<String>, Error> {
    let c = get_or_run(cli, image)?;
    cli.exec(&c.id, &format!("ls {}", dir.display()))
        .map_err(Error::DockerError)
        .map(|(out, err)| {
            (
                String::from_utf8_lossy(&out).to_owned().to_string(),
                String::from_utf8_lossy(&err).to_owned().to_string(),
            )
        })
        .and_then(|(out, err)| {
            if err != "" {
                return Err(Error::ExecError(err));
            }
            if out.trim() == "" {
                return Ok(vec![]);
            }
            let dirs: Vec<&str> = out.trim().split(" ").collect();
            let mut r = vec![];
            for dir in dirs {
                r.push(dir.trim().to_owned().to_string());
            }
            Ok(r)
        })
}
