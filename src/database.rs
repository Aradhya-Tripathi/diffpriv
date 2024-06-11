use mysql::Pool;
use regex::Regex;
use sqlite::Connection as SqliteConnection;
use std::path::Path;

const URI_PATTERN: &str = r"^mysql:\/\/([^:\/?#]+):([^@\/?#]+)@([^:\/?#]+):(\d+)\/([^\/?#]+)$";

pub struct Database {
    pub flavour: SupportedDatabases,
    // We are taking control of the connection at this point
    pub connection: ConnectionTypes,
}

#[derive(Debug)]
pub enum SupportedDatabases {
    MySQL,
    SQLite,
}

pub enum ConnectionTypes {
    SQLite(SqliteConnection),
    MySQL(Pool),
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
    pub fn new(path: &str, flavour: &str) -> Result<Self, String> {
        let processed_path = path.trim_end();
        match SupportedDatabases::from_string(flavour) {
            SupportedDatabases::MySQL => {
                if Regex::new(URI_PATTERN)
                    .unwrap()
                    .captures(processed_path)
                    .is_some()
                {
                    let connection_pool: Pool = Pool::new(processed_path).unwrap();
                    return Ok(Database {
                        flavour: SupportedDatabases::MySQL,
                        connection: ConnectionTypes::MySQL(connection_pool),
                    });
                }
                Err(format!("Invalid connection string: {path}"))
            }
            SupportedDatabases::SQLite => match Path::try_exists(Path::new(processed_path)) {
                Ok(found) => {
                    if found {
                        let connection: SqliteConnection =
                            SqliteConnection::open(processed_path).unwrap();
                        return Ok(Database {
                            flavour: SupportedDatabases::SQLite,
                            connection: ConnectionTypes::SQLite(connection),
                        });
                    }
                    Err(format!("Failed to find in-memory database {path}"))
                }
                Err(_) => Err(format!("Error occured while looking for database {path}")),
            },
        }
    }
}
