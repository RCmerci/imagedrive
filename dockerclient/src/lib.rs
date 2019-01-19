extern crate shiplift;
extern crate tokio;
extern crate tokio_core;

use tokio::prelude::Future;

#[derive(Debug)]
pub enum Error {
    SquashError(String),
    CommitError(String),
    CreateError(String),
    CopyError(String),
    PushError(String),
    LoginError(String),
    DefaultError(String),
}

pub type Container = shiplift::rep::Container;
pub type Image = shiplift::rep::ImageDetails;
pub struct DockerClient {
    inner_cli: shiplift::Docker,
    server: Option<String>,
    username: Option<String>,
    password: Option<String>,
}

impl DockerClient {
    pub fn new() -> Self {
        let inner_cli = shiplift::Docker::new();
        DockerClient {
            inner_cli: inner_cli,
            server: None,
            username: None,
            password: None,
        }
    }

    pub fn new_with_logininfo(server: &str, username: &str, password: &str) -> Self {
        let inner_cli = shiplift::Docker::new();
        let server = Some(server.into());
        let username = Some(username.into());
        let password = Some(password.into());
        DockerClient {
            inner_cli,
            server,
            username,
            password,
        }
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
    pub fn copy_in(
        &self,
        container: &str,
        src: &std::path::Path,
        dst: &std::path::Path,
    ) -> Result<(), Error> {
        copy(container, true, src, dst)
    }

    pub fn copy_out(
        &self,
        container: &str,
        src: &std::path::Path,
        dst: &std::path::Path,
    ) -> Result<(), Error> {
        copy(container, false, src, dst)
    }

    pub fn remove_file(&self, container: &str, path: &std::path::Path) -> Result<(), Error> {
        let cmd = format!("rm -rf {}", path.display());
        self.exec(container, &cmd).map(|_| ())
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
                        let e = format!("not found created container: {}", info.id);
                        Err(Error::CreateError(e))
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
        let cmd = format!("docker exec -i {} sh -c \"{}\"", container, cmd);
        std::process::Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()
            .map(|r| (r.stdout, r.stderr))
            .map_err(|e| Error::DefaultError(e.to_string()))
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

    pub fn push(&self, image: &str) -> Result<(), Error> {
        let cmd = format!("docker push {}", image);
        let r = std::process::Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .status()
            .map_err(|e| Error::DefaultError(e.to_string()))?;
        if r.success() {
            return Ok(());
        }
        if !r.success()
            && self.server.is_some()
            && self.username.is_some()
            && self.password.is_some()
        {
            let login_cmd = format!(
                "docker login {} --username={}, --password={}",
                self.server.as_ref().unwrap(),
                self.username.as_ref().unwrap(),
                self.password.as_ref().unwrap()
            );
            let output = std::process::Command::new("sh")
                .arg("-c")
                .arg(login_cmd)
                .output()
                .map_err(|e| Error::DefaultError(e.to_string()))?;
            if !output.status.success() {
                let errstr = String::from_utf8_lossy(&output.stderr).to_string();
                return Err(Error::LoginError(errstr));
            }
        } else if !r.success() {
            return Err(Error::PushError(format!("code: {:?}", r.code())));
        }

        // login succ, retry push

        std::process::Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .output()
            .map_err(|e| Error::DefaultError(e.to_string()))
            .and_then(|r| {
                if !r.status.success() {
                    let errstr = String::from_utf8_lossy(&r.stderr).to_string();
                    return Err(Error::PushError(errstr));
                }
                Ok(())
            })
    }
}

fn copy(
    container: &str,
    to_container: bool,
    src: &std::path::Path,
    dst: &std::path::Path,
) -> Result<(), Error> {
    let cmd = if to_container {
        format!(
            "docker cp {} {}:{}",
            src.display(),
            container,
            dst.display()
        )
    } else {
        format!(
            "docker cp {}:{} {}",
            container,
            src.display(),
            dst.display()
        )
    };
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
        let r_ = cli.exec(&c.id, "ls");
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
