use crate::database::connect::ConnectionTypes;
use mysql::prelude::Queryable;

pub struct Schema {
    pub tables: Vec<Table>,
}

#[derive(Debug)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
}

#[derive(Debug)]
pub struct Column {
    pub name: String,
    pub ctype: String,
    pub sensitivity: f64,
}

impl Schema {
    pub fn from_connection(conn: ConnectionTypes) {
        match conn {
            ConnectionTypes::MySQL(mut connector) => {
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
                        })
                        .unwrap();

                    tables.push(Table { name, columns })
                }
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
                let mut tables: Vec<Column> = vec![];
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
                            });
                        })
                        .unwrap();
                    for column in columns {
                        tables.push(column.unwrap())
                    }
                }
            }
        }
    }
}
