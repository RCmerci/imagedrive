extern crate shiplift;
extern crate tokio;
extern crate tokio_core;

use std::sync::mpsc::channel;
use tokio::prelude::future::ok;
use tokio::prelude::{Future, Stream};

#[derive(Debug)]
pub enum Error {
    SquashError(String),
    CommitError(String),
    CreateError(String),
    CopyError(String),
    DefaultError(String),
}

type Container = shiplift::rep::Container;
type Image = shiplift::rep::ImageDetails;
pub struct DockerClient {
    inner_cli: shiplift::Docker,
}

impl DockerClient {
    pub fn new() -> Self {
        let inner_cli = shiplift::Docker::new();
        DockerClient { inner_cli }
    }

    pub fn ps(&self, all: bool) -> Result<Vec<Container>, Error> {
        let opt = if all {
            shiplift::ContainerListOptions::builder().all().build()
        } else {
            shiplift::ContainerListOptions::builder().build()
        };
        let fut = self
            .inner_cli
            .containers()
            .list(&opt)
            .map(move |containers| {
                let mut cs = Vec::new();
                for c in containers {
                    cs.push(c);
                }
                cs
            })
            .map_err(|e| Error::DefaultError(e.to_string()));
        tokio_core::reactor::Core::new().unwrap().run(fut)
    }

    pub fn image(&self, name: &str) -> Result<Image, Error> {
        let fut = self
            .inner_cli
            .images()
            .get(name)
            .inspect()
            .map_err(|e| Error::DefaultError(e.to_string()));
        tokio_core::reactor::Core::new().unwrap().run(fut)
    }
    /// Copy host file into container
    pub fn copy_in<'a>(
        &self,
        container: &str,
        src: &std::path::Path,
        dst: &std::path::Path,
    ) -> Result<(), Error> {
        let cmd = format!(
            "docker cp {} {}:{}",
            src.display(),
            container,
            dst.display()
        );
        let r = std::process::Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()
            .map_err(|e| Error::DefaultError(e.to_string()))?;
        if !r.status.success() {
            let errstr = String::from_utf8_lossy(&r.stderr).to_string();
            return Err(Error::CopyError(errstr));
        };
        Ok(())
    }

    pub fn create(&self, image: &str) -> Result<Container, Error> {
        let fut = self
            .inner_cli
            .containers()
            .create(
                &shiplift::ContainerOptions::builder(image)
                    .tty(true)
                    .attach_stdin(true)
                    .attach_stdout(true)
                    .attach_stderr(true)
                    .build(),
            )
            .map_err(|e| Error::DefaultError(e.to_string()))
            .and_then(|info| {
                self.inner_cli
                    .containers()
                    .list(&shiplift::ContainerListOptions::builder().all().build())
                    .map_err(|e| Error::DefaultError(e.to_string()))
                    .and_then(move |containers| {
                        for c in containers {
                            if c.id == info.id {
                                return Ok(c);
                            }
                        }
                        // let e = format!("not found created container: {}", info.id);
                        // Err(e)
                        panic!("xxx")
                    })
            });
        tokio_core::reactor::Core::new().unwrap().run(fut)
    }

    pub fn start(&self, container: &str) -> Result<(), Error> {
        let fut = self
            .inner_cli
            .containers()
            .get(container)
            .start()
            .map_err(|e| Error::DefaultError(e.to_string()));
        tokio_core::reactor::Core::new().unwrap().run(fut)
    }

    pub fn commit(&self, container: &str, message: &str, new_image: &str) -> Result<(), Error> {
        let cmd = format!(
            "docker commit -m \"{}\" {} {}",
            message, container, new_image
        );
        let r = std::process::Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()
            .map_err(|e| Error::DefaultError(e.to_string()))?;
        if !r.status.success() {
            let errstr = String::from_utf8_lossy(&r.stderr).to_string();
            return Err(Error::CommitError(errstr));
        }
        Ok(())
    }

    pub fn exec(&self, container: &str, cmd: &str) -> Result<(Vec<u8>, Vec<u8>), Error> {
        let (stdout_s, stdout_r) = channel();
        let (stderr_s, stderr_r) = channel();
        let fut = self
            .inner_cli
            .containers()
            .get(container)
            .exec(
                &shiplift::ExecContainerOptions::builder()
                    .cmd(vec!["sh", "-c", cmd])
                    .attach_stdout(true)
                    .attach_stderr(true)
                    .build(),
            )
            .for_each(move |chunk| {
                match chunk.stream_type {
                    shiplift::tty::StreamType::StdOut => {
                        println!("out: {:?}", &chunk);
                        stdout_s.send(chunk.data).unwrap();
                    }
                    shiplift::tty::StreamType::StdErr => {
                        println!("err: {:?}", &chunk);
                        stderr_s.send(chunk.data).unwrap();
                    }
                    _ => panic!(""),
                };
                ok(())
            })
            .map_err(|e| Error::DefaultError(e.to_string()));
        match tokio_core::reactor::Core::new().unwrap().run(fut) {
            Err(e) => Err(e),
            Ok(_) => {
                let stdout: Vec<u8> = stdout_r.iter().flatten().collect();
                let stderr: Vec<u8> = stderr_r.iter().flatten().collect();
                Ok((stdout, stderr))
            }
        }
    }

    pub fn remove(&self, container: &str) -> Result<(), Error> {
        let fut = self
            .inner_cli
            .containers()
            .get(container)
            .remove(shiplift::RmContainerOptions::builder().force(true).build())
            .map_err(|e| Error::DefaultError(e.to_string()));
        tokio_core::reactor::Core::new().unwrap().run(fut)
    }

    pub fn squash(&self, image: &str, new_image: &str) -> Result<(), Error> {
        let c = self.create(image)?;
        let cmd = format!("docker export {} | docker import - {}", &c.id, new_image);
        let r = std::process::Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()
            .map_err(|e| Error::DefaultError(e.to_string()))?;
        if !r.status.success() {
            let _ = self.remove(&c.id);
            let errstr = String::from_utf8_lossy(&r.stderr).to_string();
            return Err(Error::SquashError(errstr));
        }
        let _ = self.remove(&c.id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use DockerClient;
    #[test]
    fn ps() {
        let cli = DockerClient::new();
        let cs = cli.ps(true);
        assert!(cs.is_ok());
    }
    #[test]
    fn create() {
        let cli = DockerClient::new();
        let r = cli.create("busybox:latest");
        println!("{:?}", r);
        assert!(r.is_ok());
    }

    #[test]
    fn copy_in() {
        let cli = DockerClient::new();
        let c = cli.create("busybox:latest");
        assert!(c.is_ok());
        let r = cli.copy_in(
            &*c.unwrap().id,
            std::path::Path::new("Cargo.toml"),
            std::path::Path::new("/"),
        );
        println!("{:?}", r);
        assert!(r.is_ok());
    }
    #[test]
    fn commit() {
        let cli = DockerClient::new();
        let c_ = cli.create("busybox:latest");
        assert!(c_.is_ok());
        let c = c_.unwrap();
        let r = cli.copy_in(
            &*c.id,
            std::path::Path::new("Cargo.toml"),
            std::path::Path::new("/"),
        );
        assert!(r.is_ok());
        let r = cli.commit(&*c.id, "test commit", "test-commit:latest");
        assert!(r.is_ok());
    }

    #[test]
    fn exec() {
        let cli = DockerClient::new();
        let c_ = cli.create("busybox:latest");
        assert!(c_.is_ok());
        let c = c_.unwrap();
        cli.start(&c.id).unwrap();
        let r_ = cli.exec(&c.id, "ls -l");
        assert!(r_.is_ok());
        let r = r_.unwrap();
        let out: &[u8] = &r.0;
        println!("{:?}", String::from_utf8_lossy(out).to_owned());
    }
    #[test]
    fn squash() {
        let cli = DockerClient::new();
        let c = cli.create("busybox").unwrap();
        let _ = cli
            .copy_in(
                &c.id,
                std::path::Path::new("Cargo.toml"),
                std::path::Path::new("/"),
            )
            .unwrap();
        let _ = cli.commit(&c.id, "add cargo toml", "test-squash").unwrap();
        assert!(cli.squash("test-squash", "new-test-squash").is_ok());
    }

}
