use std::fmt;

use crate::*;

#[derive(Debug)]
enum Error {}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "imagedrive error")
    }
}

struct ImageDrive {
    image_name: String,
    dockercli: dockerclient::DockerClient,
}

impl ImageDrive {
    fn new(image_name: String) -> ImageDrive {
        let dockercli = dockerclient::DockerClient::new();
        ImageDrive {
            image_name,
            dockercli,
        }
    }
}

impl fmt::Display for ImageDrive {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "imagedrive: {}", self.image_name)
    }
}

impl DB<Error> for ImageDrive {
    fn catalog(&self) -> Vec<String> {
        println!("{}", self.dockercli.);
        vec![String::from("asd")]
    }
    fn add(&self, _title: String, _item: &Item) -> Option<Error> {
        None
    }
    fn delete(&self, _title: String, _reference: String) -> Option<Error> {
        None
    }
    fn sync(&self) -> Option<Error> {
        None
    }
}
