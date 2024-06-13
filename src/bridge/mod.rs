use crate::database::schema::Column;

pub fn used_columns(requested: Vec<String>, mut existing: Vec<Column>) -> Vec<Column> {
    let mut used_columns: Vec<Column> = vec![];
    let aggregate_functions: Vec<&str> = vec!["sum(", "avg("];
    let mut index = 0;

    while index < existing.len() {
        for func in aggregate_functions.iter() {
            if requested.contains(&format!("{func}{})", existing[index].name)) {
                existing[index].usage = Some(func.replace("(", ""));
                used_columns.push(existing[index].to_owned());
            }
        }
        if requested.contains(&existing[index].name) {
            used_columns.push(existing[index].to_owned());
        }

        index += 1;
    }

    used_columns
}

pub fn set_sensitivity(mut column: Column, sensitivity: f64) -> Column {
    column.sensitivity = sensitivity;
    column
}
