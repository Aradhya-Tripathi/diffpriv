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
    /// Constructs a new `Database` instance based on the provided database URI or path.
    ///
    /// # Arguments
    ///
    /// * `path` - A string slice that holds either a filesystem path for SQLite or a URI for MySQL.
    ///
    /// # Returns
    ///
    /// A `Result` containing `Self` on success, or an error message `String` on failure.
    ///
    /// # Errors
    ///
    /// Returns an error if the provided `path` is not valid or the database connection cannot be established.
    ///
    /// # Example
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
        Err(format!("Failed to process database URI: {processed_path} (make sure to add database name in the URI)"))
    }
    /// Converts a `ValueRef` from SQLite to its corresponding SQL representation as a `String`.
    ///
    /// # Arguments
    ///
    /// * `value_ref` - A `ValueRef` enum variant containing the SQLite value.
    ///
    /// # Returns
    ///
    /// A `String` representation of the SQLite value.
    ///
    fn as_sql_from_sqlite(value_ref: ValueRef) -> String {
        match value_ref {
            ValueRef::Null => "Null".to_string(),
            ValueRef::Real(val) => format!("{}", val),
            ValueRef::Blob(val) => {
                format!("{}", std::str::from_utf8(val).unwrap().to_string())
            }
            ValueRef::Integer(val) => format!("{}", val),
            ValueRef::Text(val) => {
                format!("{}", std::str::from_utf8(val).unwrap())
            }
        }
    }
    /// Executes a MySQL query using the provided database connection and SQL statement.
    ///
    /// # Arguments
    ///
    /// * `connector` - A mutable reference to a MySQL connection pool.
    /// * `sql` - A string slice containing the SQL query to execute.
    ///
    /// # Returns
    ///
    /// A vector of `HashMap<String, String>` where each `HashMap` represents a row of results.
    fn execute_mysql_query(
        connector: &mut PooledConn,
        sql: &str,
    ) -> Result<Vec<HashMap<String, String>>, String> {
        let rows: Vec<Row> = connector.query(sql).map_err(|e| e.to_string())?;

        let results: Vec<HashMap<String, String>> = rows
            .iter()
            .map(|row| {
                let mut column_value_map = HashMap::new();
                for (i, column) in row.columns().iter().enumerate() {
                    let column_name = column.name_str().into_owned();
                    let value = row.as_ref(i).unwrap().to_owned();
                    column_value_map.insert(column_name, value.as_sql(true).replace("'", ""));
                }
                column_value_map
            })
            .collect();
        Ok(results)
    }
    /// Executes a SQLite query using the provided database connection and SQL statement.
    ///
    /// # Arguments
    ///
    /// * `connector` - A mutable reference to a SQLite connection.
    /// * `sql` - A string slice containing the SQL query to execute.
    ///
    /// # Returns
    ///
    /// A vector of `HashMap<String, String>` where each `HashMap` represents a row of results.
    fn execute_sqlite_query(
        connector: &mut SqliteConnection,
        sql: &str,
    ) -> Result<Vec<HashMap<String, String>>, String> {
        let mut query_stmt = connector.prepare(sql).map_err(|e| e.to_string())?;
        let column_count = query_stmt.column_count();
        let column_names = query_stmt
            .column_names()
            .iter()
            .map(|r| r.to_string())
            .collect::<Vec<String>>();
        let mut rows = query_stmt.query([]).map_err(|e| e.to_string())?;
        let mut results: Vec<HashMap<String, String>> = vec![];
        while let Some(row) = rows.next().unwrap() {
            let mut column_val_map = HashMap::new();
            for i in 0..column_count {
                // Since ValueRef type only lives as long as the row lives,
                // The next iteration the row dies therefore we need a owned value.
                let row_value = Database::as_sql_from_sqlite(row.get_ref(i).unwrap());
                column_val_map.insert(column_names[i].to_owned(), row_value);
            }
            results.push(column_val_map.to_owned());
        }
        Ok(results)
    }
    /// Executes a SQL query using the appropriate database connection based on the `Database` instance.
    ///
    /// # Arguments
    ///
    /// * `sql` - A string slice containing the SQL query to execute.
    ///
    /// # Returns
    ///
    /// A vector of `HashMap<String, String>` where each `HashMap` represents a row of results.
    /// This function delegates to `execute_mysql_query` or `execute_sqlite_query` based on the connection type.
    pub fn execute_query(&mut self, sql: &str) -> Result<Vec<HashMap<String, String>>, String> {
        match &mut self.connection {
            ConnectionTypes::MySQL(ref mut connector) => {
                Database::execute_mysql_query(connector, sql)
            }
            ConnectionTypes::SQLite(connector) => Database::execute_sqlite_query(connector, sql),
        }
    }
}
