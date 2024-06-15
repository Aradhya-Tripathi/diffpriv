use mysql::prelude::Queryable;
use mysql::{Pool, PooledConn, Row};
use regex::Regex;
use rusqlite::types::ValueRef;
use rusqlite::Connection as SqliteConnection;
use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use std::str;
const URI_PATTERN: &str = r"^mysql:\/\/([^:\/?#]+):([^@\/?#]+)@([^:\/?#]+):(\d+)\/([^\/?#]+)$";

pub struct Database {
    pub flavour: SupportedDatabases,
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
    /// Executes SQL query and gives any sort of response in simple String format
    /// Makes it easy to work with since String can be converted into required form for transforms
    pub fn execute_query(&mut self, sql: &str) -> Vec<HashMap<String, String>> {
        match &mut self.connection {
            ConnectionTypes::MySQL(ref mut connector) => {
                // Why have they made it so hard to work with databases in rust?
                let rows: Vec<Row> = connector.query(sql).unwrap();
                let results: Vec<HashMap<String, String>> = rows
                    .iter()
                    .map(|row| {
                        // Why the fuck is everything interpreted as bytes?
                        let mut column_value_map = HashMap::new();
                        for (i, column) in row.columns().iter().enumerate() {
                            let column_name = column.name_str().into_owned();
                            let value = row.as_ref(i).unwrap().to_owned();
                            column_value_map.insert(column_name, value.as_sql(false));
                        }
                        column_value_map
                    })
                    .collect();
                results
            }
            ConnectionTypes::SQLite(connector) => {
                let mut query_stmt = connector.prepare(sql).unwrap();
                let column_count = query_stmt.column_count();
                let column_names = query_stmt
                    .column_names()
                    .iter()
                    .map(|r| r.to_string())
                    .collect::<Vec<String>>();
                let mut rows = query_stmt.query([]).unwrap();
                let mut results: Vec<HashMap<String, String>> = vec![];
                while let Some(row) = rows.next().unwrap() {
                    let mut column_val_map = HashMap::new();
                    for i in 0..column_count {
                        // Since ValueRef type only lives as long as the row lives,
                        // The next iteration the row dies therefore we need a owned value.
                        let row_value = match row.get_ref(i).unwrap() {
                            ValueRef::Null => "Null".to_string(),
                            ValueRef::Real(val) => format!("{}", val),
                            ValueRef::Blob(val) => {
                                format!("{}", std::str::from_utf8(val).unwrap().to_string())
                            }
                            ValueRef::Integer(val) => format!("{}", val),
                            ValueRef::Text(val) => {
                                format!("{}", std::str::from_utf8(val).unwrap())
                            }
                        };
                        column_val_map.insert(column_names[i].to_owned(), row_value);
                    }
                    results.push(column_val_map.to_owned());
                }
                results
            }
        }
    }
}
