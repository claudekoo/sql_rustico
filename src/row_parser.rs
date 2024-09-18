use super::custom_error::CustomError;
use super::row::Row;
use std::collections::HashMap;

/// Parsea una línea de un archivo CSV y la convierte en un vector de Strings.
pub fn parse_columns(line: &str) -> Result<Vec<String>, CustomError> {
    Ok(line.split(",").map(|s| s.trim().to_string()).collect())
}

/// Parsea una fila de un archivo CSV y la convierte en un objeto Row, dado un vector de columnas.
/// Si la cantidad de valores en la fila no coincide con la cantidad de columnas, retorna un error.
pub fn parse_row(columns: &[String], line: &str) -> Result<Row, CustomError> {
    let values: Vec<&str> = line.split(",").collect();
    if values.len() != columns.len() {
        CustomError::error_invalid_table("Columns size missmatch")?;
    }
    let mut row_values: HashMap<String, String> = HashMap::new();
    for (i, value) in values.iter().enumerate() {
        row_values.insert(columns[i].to_string(), value.to_string());
    }
    let row = Row::new(columns, row_values);
    Ok(row)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_row_invalid_columns_size() {
        let columns = vec!["column1".to_string(), "column2".to_string()];
        let line = format!("{},{},{}", "value1", "value2", "value3");
        let result = parse_row(&columns, &line);
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            CustomError::InvalidTable {
                message: "Columns size missmatch".to_string()
            }
        );
    }
}
