extern crate dockerclient;

mod docker;

pub trait Item {
    /// Compare with other `Item`, true if same
    fn compare(&self, other: &Item) -> bool {
        self.hash() == other.hash()
    }
    /// Hash compute item's hash value
    fn hash(&self) -> i64;
    /// Copy Item to `dst`
    fn copy(&self, dst: String);
    /// Reference is Item's unqiue name
    fn reference(&self) -> String;
}

pub trait DB<E>
where
    E: std::fmt::Debug + std::fmt::Display,
{
    /// Catalog return catalog of db
    fn catalog(&self) -> Vec<String>;
    /// Add `item` to DB under `title`
    fn add(&self, title: String, item: &Item) -> Option<E>;
    /// Delete item from DB, which is located by title and reference
    fn delete(&self, title: String, reference: String) -> Option<E>;
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
