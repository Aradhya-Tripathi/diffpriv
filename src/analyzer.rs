pub struct SqlAnalyzer {
    pub sql: String,
}

trait Cleanable {
    fn clean(&self) -> String;
}

fn clean_results(res: &str) -> String {
    res.replace(" ", "").replace(",", "").replace(";", "")
}

impl Cleanable for String {
    fn clean(&self) -> String {
        clean_results(self.as_str())
    }
}

impl Cleanable for &str {
    fn clean(&self) -> String {
        clean_results(self)
    }
}

impl SqlAnalyzer {
    pub fn new(sql: &str) -> Self {
        SqlAnalyzer {
            sql: sql.to_ascii_lowercase().trim_end().to_string(),
        }
    }

    fn clean_results<T: Cleanable>(&self, results: Vec<T>) -> Vec<String> {
        results
            .iter()
            .map(|r: &T| r.clean())
            .collect::<Vec<String>>()
    }

    pub fn is_read(&self) -> bool {
        self.sql.starts_with("select")
    }

    pub fn tables_from_sql(&self) -> Vec<String> {
        let mut tables: Vec<String> = vec![];
        let parts = self.sql.split_whitespace().collect::<Vec<&str>>();
        let terminals: Vec<&str> = vec!["where", "group", "having", "order"];

        if let Some(position) = parts.iter().position(|&r| r == "from") {
            for index in position + 1..parts.len() {
                if terminals.contains(&parts[index]) {
                    break;
                }
                tables.push(parts[index].to_string());
            }
        } else {
            panic!("Invalid READ SQL")
        }

        // Handling alises is challening in this case, however we can do that when we run the SQL query.
        self.clean_results(tables)
    }

    /// List all the columns that are being used
    pub fn columns_from_sql(&self) -> Vec<String> {
        let mut columns: Vec<String> = vec![];
        let parts = self.sql.split_whitespace().collect::<Vec<&str>>();
        let mut index = 1;
        while index < parts.len() {
            if parts[index] == "from" {
                break;
            }
            columns.push(parts[index].to_owned());
            index += 1;
        }

        self.clean_results(columns)
    }
}

// Ok so this ensures that tests are only compiled when we run test
#[cfg(test)]
mod tests {
    use super::SqlAnalyzer;

    #[test]
    fn is_read() {
        let analyser = SqlAnalyzer::new("SELECT * FROM USERS;");
        assert_eq!(true, analyser.is_read());

        let analyser = SqlAnalyzer::new("insert into ");
        assert_eq!(false, analyser.is_read());
    }

    #[test]
    fn tables_from_sql() {
        let analyser = SqlAnalyzer::new("SELECT * FROM USERS, MODELS;");
        assert_eq!(["users", "models"].to_vec(), analyser.tables_from_sql());

        let analyser = SqlAnalyzer::new("SELECT * FROM USERS u, MODELS m;");
        assert_eq!(
            ["users", "u", "models", "m"].to_vec(),
            analyser.tables_from_sql()
        );
    }

    #[test]
    fn columns_from_sql() {
        let analyser = SqlAnalyzer::new("SELECT * FROM USERS, MODELS;");
        assert_eq!(["*"].to_vec(), analyser.columns_from_sql());

        let analyser = SqlAnalyzer::new("SELECT name, age FROM USERS, MODELS;");
        assert_eq!(["name", "age"].to_vec(), analyser.columns_from_sql());

        let analyser = SqlAnalyzer::new("SELECT Avg(Name), Sum(Age) FROM USERS, MODELS;");
        assert_eq!(
            ["avg(name)", "sum(age)"].to_vec(),
            analyser.columns_from_sql()
        );
    }
}
