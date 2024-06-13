use mysql::Pool;
use mysql::PooledConn;
use regex::Regex;
use rusqlite::Connection as SqliteConnection;
use std::fmt;
use std::path::Path;

const URI_PATTERN: &str = r"^mysql:\/\/([^:\/?#]+):([^@\/?#]+)@([^:\/?#]+):(\d+)\/([^\/?#]+)$";

pub struct Database {
    pub flavour: SupportedDatabases,
    // We are taking control of the connection at this point
    pub connection: ConnectionTypes,
}

impl fmt::Display for Database {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Connected to {:?}", self.flavour)
    }
}

#[derive(Debug)]
pub enum SupportedDatabases {
    MySQL,
    SQLite,
}

pub enum ConnectionTypes {
    SQLite(SqliteConnection),
    MySQL(PooledConn),
}

impl SupportedDatabases {
    pub fn from_string(flavour: &str) -> Self {
        match flavour.to_ascii_lowercase().as_str() {
            "sqlite" => Self::SQLite,
            "mysql" => Self::MySQL,
            _ => panic!("{flavour} not supported!"),
        }
    }
}

impl Database {
    pub fn new(path: &str) -> Result<Self, String> {
        let processed_path = path.trim_end();
        if Path::exists(Path::new(processed_path)) {
            let connection: SqliteConnection = SqliteConnection::open(processed_path).unwrap();
            return Ok(Database {
                flavour: SupportedDatabases::SQLite,
                connection: ConnectionTypes::SQLite(connection),
            });
        } else if Regex::new(URI_PATTERN)
            .unwrap()
            .captures(processed_path)
            .is_some()
        {
            let connection_pool: PooledConn =
                Pool::new(processed_path).unwrap().get_conn().unwrap();
            return Ok(Database {
                flavour: SupportedDatabases::MySQL,
                connection: ConnectionTypes::MySQL(connection_pool),
            });
        }
        Err(format!("Failed to process database URI: {processed_path}"))
    }
}
