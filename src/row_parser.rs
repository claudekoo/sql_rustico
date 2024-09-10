use super::custom_error::CustomError;
use super::row::Row;
use std::collections::HashMap;

/// Parsea una fila de un archivo CSV y la convierte en un objeto Row, dado un vector de columnas.
/// Si la cantidad de valores en la fila no coincide con la cantidad de columnas, retorna un error.
pub fn parse_row(columns: &Vec<String>, line: &str) -> Result<Row, CustomError> {
    let values: Vec<&str> = line.split(",").collect();
    if values.len() != columns.len() {
        return Err(CustomError::InvalidTable {
            message: "Columns size missmatch".to_string(),
        });
    }
    let mut row_values: HashMap<String, String> = HashMap::new();
    for (i, value) in values.iter().enumerate() {
        row_values.insert(columns[i].to_string(), value.to_string());
    }
    let row = Row::new(columns, row_values);
    Ok(row)
}
