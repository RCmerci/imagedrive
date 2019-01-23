use crate::utils::get_or_run;
use crate::*;
use hex;
use std::fmt;
use std::path::Path;
#[derive(Debug)]
pub enum Error {
    NotExistItem(String),
    NotFoundEntry(String),
    DockerError(dockerclient::Error),
    ExecError(String),
    HostItemError(hostitem::Error),
    ContainerItemError(containeritem::Error),
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
        let dockercli = dockerclient::DockerClient::new_with_logininfo(
            Some(server),
            Some(username),
            Some(password),
        );
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
        if !itempath.exists() {
            return Err(Error::NotExistItem(format!("{}", itempath.display())));
        }
        let mut item = hostitem::HostItem::new(itempath).map_err(Error::HostItemError)?;

        let c = get_or_run(&self.dockercli, &self.image_name).map_err(Error::DockerError)?;
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
                    out.trim().split_whitespace().collect()
                };
                let mut ps = vec![];
                let mut fileitems = vec![];
                for item in items {
                    ps.push(Path::new("/data").join(entry).join(item));
                }
                for p in &ps {
                    fileitems.push(
                        containeritem::ContainerItem::new(&p, &self.dockercli, &self.image_name)
                            .map_err(Error::ContainerItemError)?,
                    );
                }

                for mut fi in fileitems {
                    if compare_items(&mut fi, &mut item) {
                        return Ok(AddResult::ExistedItem(fi.id().into()));
                    }
                }
                let dstpath = Path::new("/data").join(entry).join(item.id());
                self.dockercli
                    .copy_in(&c.id, item.srcpath(), &dstpath)
                    .map_err(Error::DockerError)?;

                self.dockercli
                    .exec(
                        &c.id,
                        &format!(
                            "mkdir -p {} && echo {} > {}",
                            Path::new("/checksum")
                                .join(dstpath.parent().unwrap().strip_prefix("/").unwrap())
                                .display(),
                            hex::encode(item.hash()),
                            Path::new("/checksum")
                                .join(dstpath.strip_prefix("/").unwrap())
                                .display()
                        ),
                    )
                    .expect("write checksum");
                Ok(AddResult::Succ)
            })
    }
    fn delete(&self, entry: &str, item: &str) -> Result<(), Error> {
        let c = get_or_run(&self.dockercli, &self.image_name).map_err(Error::DockerError)?;
        let dstpath = Path::new("/data").join(entry).join(item);
        self.dockercli
            .remove_file(&c.id, &dstpath)
            .map_err(|e| Error::ExecError(format!("{:?}", e)))
    }

    fn export_to_dir(&self, dir: &Path, entry: &str) -> Result<(), Error> {
        let c = get_or_run(&self.dockercli, &self.image_name).map_err(Error::DockerError)?;
        let srcpath = Path::new("/data").join(entry);
        self.dockercli
            .copy_out(&c.id, &srcpath, dir)
            .map_err(Error::DockerError)
    }

    fn sync(&self) -> Result<(), Error> {
        // TODO: check image exist, if not , pull from registry

        // image existed, so push to registry

        // 1. commit all changed data in container to image
        let c = get_or_run(&self.dockercli, &self.image_name).map_err(Error::DockerError)?;
        if self.dockercli.diff_container_image_content(&c.id) {
            let _ = self
                .dockercli
                .commit(&c.id, "commit by sync", &self.image_name)
                .map_err(Error::DockerError)?;
        }

        // 2. push image
        self.dockercli
            .push(&self.image_name)
            .map_err(Error::DockerError)
    }

    fn sync_from_remote(&self) -> Result<(), Error> {
        let _ = self
            .dockercli
            .remove_image(&self.image_name)
            .map_err(Error::DockerError);

        self.dockercli
            .pull(&self.image_name)
            .map_err(Error::DockerError)
    }
}

fn ls(cli: &dockerclient::DockerClient, image: &str, dir: &Path) -> Result<Vec<String>, Error> {
    let c = get_or_run(cli, image).map_err(Error::DockerError)?;
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
            let dirs: Vec<&str> = out.trim().split_whitespace().collect();
            let mut r = vec![];
            for dir in dirs {
                r.push(dir.trim().to_owned().to_string());
            }
            Ok(r)
        })
}
