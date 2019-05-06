extern crate clap;
extern crate config;
extern crate db;
extern crate dirs;
extern crate front;
use clap::{App, Arg, SubCommand};
use db::docker::ImageDrive;

fn main() {
    let matches = App::new("ImageDrive")
        .version("0.1")
        .author("rcmerci@gmail.com")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file, default: ~/.imagedrive")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .help("Sets an optional output file")
                .index(1),
        )
        .subcommand(
            SubCommand::with_name("ls")
                .about("list entries or items")
                .arg(
                    Arg::with_name("entry")
                        .help("set it if list items")
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("put")
                .about("put host file to imagedrive")
                .arg(Arg::with_name("entry").help("entry name").required(true))
                .arg(Arg::with_name("file").help("file path").required(true))
                .arg(Arg::with_name("name").help("rename file")),
        )
        .subcommand(
            SubCommand::with_name("export")
                .about("export entry to host")
                .arg(Arg::with_name("entry").help("entry name").required(true))
                .arg(Arg::with_name("dir").help("dst dir path").required(true)),
        )
        .subcommand(
            SubCommand::with_name("sync")
                .about("sync localDB with remoteDB")
                .arg(
                    Arg::with_name("from_remote")
                        .help("sync remoteDB to localDB")
                        .short("r")
                        .long("remote"),
                ),
        )
        .subcommand(
            SubCommand::with_name("rm")
                .about("remove entry or file")
                .arg(Arg::with_name("entry").help("entry name").required(true))
                .arg(Arg::with_name("file").help("file name")),
        )
        .get_matches();

    let mut config_path;
    if matches.is_present("config") {
        config_path = Some(std::path::Path::new(matches.value_of("config").unwrap()).to_path_buf());
    } else {
        let home = dirs::home_dir().unwrap();
        config_path = Some(std::path::Path::new(&home).join(".imagedrive"));
    }
    let config_path = config_path.map(|p| {
        if !p.exists() {
            let errstr = format!("config file: '{:?}' not exists", p.display());
            panic!(errstr);
        }
        p
    });

    let cfg = config::get_config(config_path.unwrap());

    let username = &cfg.username;
    let server = &cfg.server;
    let password = &cfg.password;
    let image_name = &cfg.image_name;

    if let Some(matches) = matches.subcommand_matches("ls") {
        if matches.is_present("entry") {
            front::list_entry_item(
                &ImageDrive::new(image_name, server, username, password),
                matches.value_of("entry").unwrap(),
            );
        } else {
            front::list_entry(&ImageDrive::new(image_name, server, username, password));
        }
    } else if let Some(matches) = matches.subcommand_matches("put") {
        let entry = matches.value_of("entry").unwrap();
        let filepath = matches.value_of("file").unwrap();
        let rename = matches.value_of("name");
        front::put(
            &ImageDrive::new(image_name, server, username, password),
            entry,
            filepath,
            rename,
        );
    } else if let Some(matches) = matches.subcommand_matches("export") {
        let entry = matches.value_of("entry").unwrap();
        let filepath = matches.value_of("dir").unwrap();
        front::export(
            &ImageDrive::new(image_name, server, username, password),
            entry,
            filepath,
        );
    } else if let Some(matches) = matches.subcommand_matches("sync") {
        front::sync(
            &ImageDrive::new(image_name, server, username, password),
            matches.is_present("from_remote"),
        );
    } else if let Some(matches) = matches.subcommand_matches("rm") {
        let entry = matches.value_of("entry").unwrap();
        let file = matches.value_of("file");
        front::rm(
            &ImageDrive::new(image_name, server, username, password),
            entry,
            file,
        );
    } else {
        // default
        front::list_entry(&ImageDrive::new(image_name, server, username, password));
    }
}
