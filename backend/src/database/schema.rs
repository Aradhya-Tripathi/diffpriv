use crate::database::database::ConnectionTypes;
use crate::database::database::Database;
use core::fmt;
use mysql::prelude::Queryable;
use mysql::PooledConn;
use rusqlite::Connection as SqliteConnection;
use serde::Serialize;

pub struct Schema {
    pub tables: Vec<Table>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
    pub privacy_budget: f64,
}

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {:?}", self.name, self.columns)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Column {
    pub name: String,
    pub ctype: String,
    pub sensitivity: f64,
    pub usage: Option<String>,
    pub table_name: String, // We need this down the line to make things simple.
}

impl Schema {
    fn generate_mysql_schema(connector: &mut PooledConn) -> Vec<Table> {
        let current_db = connector
            .query_first::<String, &str>("SELECT Database()")
            .unwrap()
            .unwrap(); // I know for a fact that at this point we have a database connection
                       // Therefore no need to check if current_db is some
        let tables_query = format!(
            r"
            SELECT TABLE_NAME
            FROM INFORMATION_SCHEMA.TABLES
            WHERE TABLE_SCHEMA = '{current_db}';"
        );
        let mut tables: Vec<Table> = vec![];
        let table_names: Vec<String> = connector.query_map(tables_query, |table| table).unwrap();
        for name in table_names {
            let column_query = format!(
                r"SELECT COLUMN_NAME, DATA_TYPE
                FROM INFORMATION_SCHEMA.COLUMNS
                WHERE TABLE_SCHEMA = '{current_db}'
                AND TABLE_NAME = '{name}';"
            );
            let columns = connector
                .query_map(column_query, |(column_name, data_type)| Column {
                    name: column_name,
                    ctype: data_type,
                    sensitivity: 0.0, // To be decided!
                    usage: None,
                    table_name: name.clone(),
                })
                .unwrap();
            tables.push(Table {
                name,
                columns,
                privacy_budget: 0.0, // To be decided
            })
        }
        tables
    }

    fn generate_sqlite_schema(connector: &mut SqliteConnection) -> Vec<Table> {
        let mut tables_stmt = connector
            .prepare("SELECT name FROM sqlite_master WHERE type='table';")
            .unwrap();
        let table_names = tables_stmt
            .query_map([], |row| row.get::<_, String>(0))
            .unwrap()
            .map(|res| res.unwrap())
            .collect::<Vec<String>>();
        let mut tables: Vec<Table> = vec![];
        for table_name in table_names.iter() {
            let mut column_stmt = connector
                .prepare(format!("PRAGMA table_info({table_name})").as_str())
                .unwrap();
            let columns = column_stmt
                .query_map([], |row| {
                    return Ok(Column {
                        name: row.get::<_, String>(1).unwrap(),
                        ctype: row.get::<_, String>(2).unwrap(),
                        sensitivity: 0.0,
                        usage: None,
                        table_name: table_name.clone(),
                    });
                })
                .unwrap();
            tables.push(Table {
                name: table_name.clone(),
                columns: columns
                    .into_iter()
                    .map(|column| {
                        let mut column = column.unwrap();
                        if column.ctype == "" {
                            // Column affinity towards Varchar
                            column.ctype = "Varchar".to_string();
                        }
                        column
                    })
                    .collect::<Vec<Column>>(),
                privacy_budget: 0.0, // To be decided
            });
        }
        tables
    }

    pub fn from_connection(conn: &mut Database) -> Vec<Table> {
        match &mut conn.connection {
            ConnectionTypes::MySQL(ref mut connector) => Schema::generate_mysql_schema(connector),
            ConnectionTypes::SQLite(connector) => Schema::generate_sqlite_schema(connector),
        }
    }
}
