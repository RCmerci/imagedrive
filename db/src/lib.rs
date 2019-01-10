extern crate dockerclient;
extern crate sha2;
extern crate walkdir;

mod dir_item;
mod docker;
mod file_item;
pub trait Item {
    /// Compare with other `Item`, true if same
    fn compare(&mut self, other: &mut Item) -> bool {
        self.hash() == other.hash()
    }
    /// Hash compute item's hash value
    fn hash(&mut self) -> Vec<u8>;
    /// id is Item's unqiue name
    fn id(&self) -> String;
}

pub trait DB<E>
where
    E: std::fmt::Debug + std::fmt::Display,
{
    /// Catalog return catalog of db
    fn catalog(&self) -> Result<Vec<String>, E>;
    /// Add `item` to DB under `title`
    fn add(&self, title: &str, item: &Item) -> Option<E>;
    /// Delete item from DB, which is located by title and reference
    fn delete(&self, title: &str, id: &str) -> Option<E>;
    /// Sync local DB to remote DB
    fn sync(&self) -> Option<E>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
