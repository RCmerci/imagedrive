use std::fmt;

use crate::*;

#[derive(Debug)]
enum Error {
    DockerError(dockerclient::Error),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "imagedrive error")
    }
}

struct ImageDrive<'a> {
    image_name: &'a str,
    dockercli: dockerclient::DockerClient,
}

impl<'a> ImageDrive<'a> {
    fn new(image_name: &str) -> ImageDrive {
        let dockercli = dockerclient::DockerClient::new();
        ImageDrive {
            image_name,
            dockercli,
        }
    }
}

impl<'a> fmt::Display for ImageDrive<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "imagedrive: {}", self.image_name)
    }
}

impl<'a> DB<Error> for ImageDrive<'a> {
    fn catalog(&self) -> Result<Vec<String>, Error> {
        // let c = get_or_run(self.image_name)?;
        // self.dockercli.exec(&c.id, "ls /data");
        Ok(vec![])
    }
    fn add(&self, _title: &str, _item: &Item) -> Option<Error> {
        None
    }
    fn delete(&self, _title: &str, _id: &str) -> Option<Error> {
        None
    }
    fn sync(&self) -> Option<Error> {
        None
    }
}

fn get_or_run(
    cli: &dockerclient::DockerClient,
    image: &str,
) -> Result<dockerclient::Container, Error> {
    let cs = (cli.ps(false).map_err(Error::DockerError)?);
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
