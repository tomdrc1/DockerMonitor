extern crate sqlite;

pub struct DBHelper
{
    connection: sqlite::Connection
}

impl DBHelper
{
    pub fn new(file_name: String) -> DBHelper
    {
        DBHelper{connection: sqlite::open(file_name).unwrap()}
    }

    /// Makes the wanted tables (images, files)
    pub fn create_tables(&self)
    {
        self.connection.execute("CREATE TABLE IF NOT EXISTS images (name TEXT PRIMARY KEY NOT NULL, digest TEXT NOT NULL, hashed INTEGER NOT NULL)").unwrap();
        self.connection.execute("CREATE TABLE IF NOT EXISTS files (path TEXT NOT NULL, digest TEXT NOT NULL, hash TEXT NOT NULL)").unwrap();
    }
}