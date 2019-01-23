use crate::utils::get_or_run;
use hex::FromHex;
use std::io;
use std::path::Path;
#[derive(Debug)]
pub enum Error {
    BadPath(String),
    IOError(io::Error),
    DefaultError(String),
}

pub struct ContainerItem<'a, 'b> {
    path: &'a Path,
    id: String,
    dockercli: &'b dockerclient::DockerClient,
    image: String,
}

impl<'a, 'b> ContainerItem<'a, 'b> {
    pub fn new(
        path: &'a Path,
        dockercli: &'b dockerclient::DockerClient,
        image: &str,
    ) -> Result<Self, Error> {
        let mut filename = "".into();
        match path.file_name() {
            None => {
                return Err(Error::BadPath(path.display().to_string()));
            }
            Some(s) => filename = s.to_string_lossy().to_owned(),
        }
        let id = format!("{:?}", filename);
        let image = image.into();
        Ok(ContainerItem {
            path,
            id,
            dockercli,
            image,
        })
    }
}

impl<'a, 'b> crate::Item for ContainerItem<'a, 'b> {
    fn hash(&mut self) -> Vec<u8> {
        let c = get_or_run(self.dockercli, &self.image).expect("get or run container fail");
        let (out, err) = self
            .dockercli
            .exec(
                &c.id,
                &format!(
                    "cat {}",
                    Path::new("/checksum")
                        .join(self.path.strip_prefix("/").unwrap())
                        .display()
                ),
            )
            .map(|(out, err)| {
                (
                    (String::from_utf8_lossy(&out).to_owned().to_string()),
                    (String::from_utf8_lossy(&err).to_owned().to_string()),
                )
            })
            .expect("get checksum file fail");

        if err != "" {
            panic!(format!(
                "out:{}, err:{}, {}",
                out,
                err,
                format!(
                    "cat {}",
                    Path::new("/checksum")
                        .join(self.path.strip_prefix("/").unwrap())
                        .display()
                )
            ));
        }
        Vec::from_hex(out.trim()).expect("from_hex")
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn srcpath(&self) -> &Path {
        self.path
    }
}
