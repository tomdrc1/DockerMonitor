extern crate sqlite;

pub struct DBHelper
{
    connection: sqlite::Connection
}

impl DBHelper
{
    pub fn new(file_name: String) -> DBHelper
    {
        let connection = sqlite::open(file_name).unwrap();
        connection.execute("CREATE TABLE IF NOT EXISTS files (path TEXT NOT NULL, digest TEXT NOT NULL, hash TEXT NOT NULL)").unwrap();
        DBHelper{connection: connection}
    }

    /// Will insert a file entry to the db
    /// 
    /// # Arguments
    /// * `path` - The path to the file in the image
    /// * `digest` - The image digest, each image has a unique digest.format
    /// * `hash` - The hash of the file in the path. (md5)
    pub fn insert_file(&self, path: &String, digest: &String, hash: &String)
    {
        self.connection.execute(format!("INSERT INTO files (path, digest, hash) VALUES('{}', '{}', '{}')", path, digest, hash)).unwrap();
    }
}