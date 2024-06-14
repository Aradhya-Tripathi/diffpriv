use core::fmt;

use crate::database::database::ConnectionTypes;
use crate::database::database::Database;
use mysql::prelude::Queryable;

pub struct Schema {
    pub tables: Vec<Table>,
}

#[derive(Debug)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
}

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {:?}", self.name, self.columns)
    }
}

#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub ctype: String,
    pub sensitivity: f64,
    pub usage: Option<String>,
}

impl Schema {
    pub fn from_connection(conn: &mut Database) -> Vec<Table> {
        match &mut conn.connection {
            // Only borrowing the connector and not moving it!
            ConnectionTypes::MySQL(ref mut connector) => {
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
                let table_names: Vec<String> =
                    connector.query_map(tables_query, |table| table).unwrap();
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
                        })
                        .unwrap();

                    tables.push(Table { name, columns })
                }
                return tables;
            }

            ConnectionTypes::SQLite(connector) => {
                let mut tables_stmt = connector
                    .prepare("SELECT name FROM sqlite_master WHERE type='table';")
                    .unwrap();
                let table_names = tables_stmt
                    .query_map([], |row| row.get::<_, String>(0))
                    .unwrap()
                    .map(|res| res.unwrap())
                    .collect::<Vec<String>>();
                let mut tables: Vec<Table> = vec![];
                for table_name in table_names {
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
                            });
                        })
                        .unwrap();
                    tables.push(Table {
                        name: table_name,
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
                    });
                }
                return tables;
            }
        }
    }
}
