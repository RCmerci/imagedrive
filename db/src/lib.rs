extern crate dockerclient;
extern crate hex;
extern crate sha2;
extern crate walkdir;

mod containeritem;
pub mod docker;
mod hostitem;
mod utils;
pub trait Item {
    /// Hash compute item's hash value
    fn hash(&mut self) -> Vec<u8>;
    /// id is Item's unqiue name
    fn id(&self) -> &str;
    /// path is item's path
    fn srcpath(&self) -> &std::path::Path;
}

/// compare_items compare 2 items, true if same
fn compare_items<T1: Item, T2: Item + ?Sized>(i1: &mut T1, i2: &mut T2) -> bool {
    i1.hash() == i2.hash()
}

#[derive(Debug)]
pub enum AddResult {
    ExistedItem(String),
    Succ,
}

pub trait DB<E>
where
    E: std::fmt::Debug + std::fmt::Display,
{
    /// entries return catalog of db
    fn entries(&self) -> Result<Vec<String>, E>;
    /// items list items under `entry`
    fn items(&self, entry: &str) -> Result<Vec<String>, E>;
    /// add `item` to DB under `entry`
    fn add(&self, entry: &str, itempath: &std::path::Path) -> Result<AddResult, E>;
    /// delete item from DB, which is located by entry and reference
    fn delete(&self, entry: &str, item: &str) -> Result<(), E>;
    /// export_to_dir export `entry` to `dir`
    fn export_to_dir(&self, dir: &std::path::Path, entry: &str) -> Result<(), E>;
    /// sync local DB to remote DB
    /// or sync remote DB to local DB if localDB not exists
    fn sync(&self) -> Result<(), E>;
    /// sync remote DB to local DB even localDB exists, so it will overwrite localDB
    fn sync_from_remote(&self) -> Result<(), E>;
}
