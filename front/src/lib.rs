extern crate db;
#[macro_use]
extern crate prettytable;

use db::docker::ImageDrive;
use db::DB;
use prettytable::Table;
use std::path::Path;

pub fn list_entry(db: &ImageDrive) {
    match db.entries() {
        Err(e) => println!("list entries fail: {:?}", e),
        Ok(entries) => {
            let mut table = Table::new();
            table.add_row(row!["Entry", "Item Count"]);

            for entry in entries {
                table.add_row(row![entry, "<unknown>"]);
            }
            table.printstd();
        }
    }
}

pub fn list_entry_item(db: &ImageDrive, entry: &str) {
    match db.items(entry) {
        Err(e) => println!("list items fail: {:?}", e),
        Ok(items) => {
            let mut table = Table::new();
            table.add_row(row!["Item"]);
            for item in items {
                table.add_row(row![item]);
            }
            table.printstd();
        }
    }
}

pub fn put(db: &ImageDrive, entry: &str, item: &str, rename: Option<&str>) {
    match db.add(entry, Path::new(item), rename) {
        Err(e) => println!("put item fail: {:?}", e),
        Ok(r) => println!("{:?}", r),
    }
}

pub fn export(db: &ImageDrive, entry: &str, dstdir: &str) {
    match db.export_to_dir(Path::new(dstdir), entry) {
        Err(e) => println!("export entry fail: {:?}", e),
        Ok(_) => println!("export entry '{}' successfully", entry),
    }
}

pub fn sync(db: &ImageDrive, from_remote: bool) {
    match if from_remote {
        db.sync_from_remote()
    } else {
        db.sync()
    } {
        Err(e) => println!("sync fail: {:?}", e),
        Ok(_) => println!("sync localDB to remoteDB successfully"),
    }
}

pub fn rm(db: &ImageDrive, entry: &str, file: Option<&str>) {
    match db.delete(entry, file) {
        Err(e) => println!("rm entry (or file) fail: {:?}", e),
        Ok(_) => println!("rm [{:?}]", entry.to_owned() + "/" + file.unwrap_or("")),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
