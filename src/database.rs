use regex::Regex;
use std::path::Path;

const URI_PATTERN: &str = r"^mysql:\/\/([^:\/?#]+):([^@\/?#]+)@([^:\/?#]+):(\d+)\/([^\/?#]+)$";

#[derive(Debug)]
pub struct Database {
    pub path: String,
    pub flavour: SupportedDatabases,
}

#[derive(Debug)]
pub enum SupportedDatabases {
    MySQL,
    SQLite,
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
        match SupportedDatabases::from_string(flavour) {
            SupportedDatabases::MySQL => {
                if Regex::new(URI_PATTERN).unwrap().captures(path).is_some() {
                    return Ok(Database {
                        path: path.to_string(),
                        flavour: SupportedDatabases::MySQL,
                    });
                }
                Err(format!("Invalid connection string: {path}"))
            }
            SupportedDatabases::SQLite => match Path::try_exists(Path::new(path.trim_end())) {
                Ok(found) => {
                    if found {
                        return Ok(Database {
                            path: path.to_string(),
                            flavour: SupportedDatabases::SQLite,
                        });
                    }
                    Err(format!("Failed to find in-memory database {path}"))
                }
                Err(_) => Err(format!("Error occured while looking for database {path}")),
            },
        }
    }
}
