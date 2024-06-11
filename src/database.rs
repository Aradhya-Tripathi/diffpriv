use std::path::Path;

pub struct Database {
    pub path: String,
    pub flavour: SupportedDatabases,
}

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
            SupportedDatabases::MySQL => Ok(Database {
                // Implement some logic to check database URI
                path: path.to_string(),
                flavour: SupportedDatabases::MySQL,
            }),
            SupportedDatabases::SQLite => match Path::try_exists(Path::new(path)) {
                Ok(found) => {
                    if found {
                        return Ok(Database {
                            path: path.to_string(),
                            flavour: SupportedDatabases::SQLite,
                        });
                    }
                    return Err("Failed to find in-memory database {path}".to_string());
                }
                Err(_) => return Err("Error occured while looking for database {path}".to_string()),
            },
        }
    }
}
