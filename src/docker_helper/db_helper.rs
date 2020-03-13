extern crate sqlite;

pub struct DBHelper
{
    connection: sqlite::Connection
}

impl DBHelper
{
    /// Returns a new db object, also creating the needed tables (files, images)
    /// 
    /// # Arguemtns
    /// * `file_name` - The name of the db file.
    pub fn new(file_name: String) -> DBHelper
    {
        let connection = sqlite::open(file_name).unwrap();
        connection.execute("CREATE TABLE IF NOT EXISTS files (path TEXT NOT NULL, digest TEXT NOT NULL, hash TEXT NOT NULL)").unwrap();
        connection.execute("CREATE TABLE IF NOT EXISTS images (digest TEST PRIMARY KEY NOT NULL, hashed INTEGER NOT NULL)").unwrap();
        DBHelper{connection: connection}
    }

    /// Will insert an image entry to the db
    /// 
    /// # Arguments
    /// * `digest` - The image digest, each image has a unique digest.
    /// * `hashed` - Weather the image was hashed or not.
    pub fn insert_image(&self, digest: &String, hashed: i32)
    {
        self.connection.execute(format!("INSERT INTO images (digest, hashed) VALUES('{}', {})", digest, hashed)).unwrap();
    }

    /// Will get the hashed status of the image
    /// 
    /// # Arguments
    /// * `digest` - The image digest, each iamge has a unique digest.
    pub fn is_image_hashed(&self, digest: &String) -> i64
    {
        let mut statement = self.connection.prepare(format!("SELECT hashed FROM images WHERE digest='{}'", digest)).unwrap().cursor();
        let mut hashed = 0;

        while let Some(row) = statement.next().unwrap()
        {
            hashed = row[0].as_integer().unwrap();
        }

        hashed
    }

    /// Will insert a file entry to the db
    /// 
    /// # Arguments
    /// * `path` - The path to the file in the image
    /// * `digest` - The image digest, each image has a unique digest.
    /// * `hash` - The hash of the file in the path. (md5)
    pub fn insert_file(&self, path: &String, digest: &String, hash: &String)
    {
        self.connection.execute(format!("INSERT INTO files (path, digest, hash) VALUES('{}', '{}', '{}')", path, digest, hash)).unwrap();
    }

    pub fn get_hash(&self, path: &String, digest: &String) -> String
    {
        let mut statement = self.connection.prepare(format!("SELECT hash FROM files WHERE path='{}' AND digest='{}'", path, digest)).unwrap().cursor();
        let mut hash: String = String::new();

        while let Some(row) = statement.next().unwrap()
        {
            hash = row[0].as_string().unwrap().to_string();
        }

        hash
    }
}