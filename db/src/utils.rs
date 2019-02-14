use crate::*;

pub fn get_or_run(
    cli: &dockerclient::DockerClient,
    image: &str,
) -> Result<dockerclient::Container, dockerclient::Error> {
    let cs = cli.ps(false)?;
    for c in cs {
        if c.image.split(":").collect::<Vec<&str>>()[0]
            == image.split(":").collect::<Vec<&str>>()[0]
        {
            return Ok(c);
        }
    }
    // not found
    let c = cli.create(image)?;
    cli.start(&c.id)?;
    Ok(c)
}
pub fn run(
    cli: &dockerclient::DockerClient,
    image: &str,
) -> Result<dockerclient::Container, dockerclient::Error> {
    let c = cli.create(image)?;
    cli.start(&c.id)?;
    Ok(c)
}
