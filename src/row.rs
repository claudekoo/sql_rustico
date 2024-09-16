use super::custom_error::CustomError;
use super::expression::{evaluate_expression, Expression};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};

/// Valor por defecto para una columna vacía.
const DEFAULT_VALUE: &str = "";

/// Una fila en esta implementación es un conjunto de valrores asociados a columnas. Convenientemente tiene un vector de columnas además de un HashMap de valores para tener referencia de orden de las columnas.
/// Puede escribirse en un archivo CSV actualizando su estado según una condición dada.
pub struct Row {
    columns_in_order: Vec<String>,
    values: HashMap<String, String>,
}

fn write_result(writer: &mut BufWriter<File>, string: &str) -> Result<(), CustomError> {
    if let Err(error) = write!(writer, "{}", string) {
        CustomError::error_generic(&format!("Error writing to file: {}", error))?;
    }
    Ok(())
}

impl Row {
    /// Crea una nueva fila dado un vector de columnas y un HashMap de valores.
    pub fn new(columns: &[String], values: HashMap<String, String>) -> Row {
        let mut columns_in_order = Vec::new();
        for item in columns.iter() {
            columns_in_order.push(item.to_string());
        }
        Row {
            columns_in_order,
            values,
        }
    }

    /// Se escribe a un archivo CSV.
    pub fn write_row(&self, writer: &mut BufWriter<File>) -> Result<(), CustomError> {
        let last_index = self.columns_in_order.len() - 1;

        for (actual_index, column) in self.columns_in_order.iter().enumerate() {
            let value_option = self.values.get(column);
            if let Some(value) = value_option {
                write_result(writer, value)?;
            } else {
                write_result(writer, DEFAULT_VALUE)?;
            }
            if actual_index != last_index {
                write_result(writer, ",")?;
            } else {
                write_result(writer, "\n")?;
            }
        }
        Ok(())
    }

    /// Actualiza los valores de una fila si cumple con una condición dada, dado un HashMap de columnas y valores a actualizar.
    pub fn update_row(
        &mut self,
        update_values: &HashMap<String, String>,
        condition: &Expression,
        writer: &mut BufWriter<File>,
    ) -> Result<(), CustomError> {
        let expression_is_true = evaluate_expression(condition, &self.values)?;
        if expression_is_true {
            for column_to_update in update_values.keys() {
                update_if_present(
                    &mut self.values,
                    column_to_update,
                    update_values[column_to_update].as_str(),
                )?;
            }
        }
        self.write_row(writer)?;
        Ok(())
    }

    /// Se escribe a un archivo CSV si cumple con una condición dada, de lo contrario se omite.
    pub fn delete_row(
        &self,
        condition: &Expression,
        writer: &mut BufWriter<File>,
    ) -> Result<(), CustomError> {
        let expression_is_true: bool = evaluate_expression(condition, &self.values)?;
        if !expression_is_true {
            self.write_row(writer)?;
        }
        Ok(())
    }

    /// Verifica si la fila cumple con una condición dada, devolviendo un booleano.
    pub fn check_condition(&self, condition: &Expression) -> Result<bool, CustomError> {
        let result: bool = evaluate_expression(condition, &self.values)?;
        Ok(result)
    }

    /// Devuelve un HashMap con los valores de la fila.
    pub fn hashmap(&self) -> HashMap<String, String> {
        let mut new_hashmap = HashMap::new();
        for (key, value) in self.values.iter() {
            new_hashmap.insert(key.to_string(), value.to_string());
        }
        new_hashmap
    }
}

fn update_if_present(
    map: &mut HashMap<String, String>,
    key: &str,
    value: &str,
) -> Result<(), CustomError> {
    if map.get(key).is_some() {
        map.insert(key.to_string(), value.to_string());
        Ok(())
    } else {
        CustomError::error_invalid_column("Column does not exist")
    }
}

#[cfg(test)]
mod tests {
    use super::super::expression::Operand;
    use super::*;

    const COLULMN1: &str = "column1";
    const COLUMN2: &str = "column2";
    const VALUE1: &str = "value1";
    const VALUE2: &str = "value2";
    const NEWVALUE1: &str = "new_value1";

    fn create_row_with_columns() -> Row {
        let columns = vec![COLULMN1.to_string(), COLUMN2.to_string()];
        let mut values = HashMap::new();
        values.insert(COLULMN1.to_string(), COLULMN1.to_string());
        values.insert(COLUMN2.to_string(), COLUMN2.to_string());
        Row::new(&columns, values)
    }

    fn create_row_with_values() -> Row {
        let columns = vec![COLULMN1.to_string(), COLUMN2.to_string()];
        let mut values = HashMap::new();
        values.insert(COLULMN1.to_string(), VALUE1.to_string());
        values.insert(COLUMN2.to_string(), VALUE2.to_string());
        Row::new(&columns, values)
    }

    #[test]
    fn test_new_row() {
        let columns = vec![COLULMN1.to_string(), COLUMN2.to_string()];
        let values = HashMap::new();
        let row = Row::new(&columns, values);
        assert_eq!(row.columns_in_order, columns);
        assert_eq!(row.values, HashMap::new());
    }

    #[test]
    fn test_write_row() {
        let row = create_row_with_values();

        let test_path = &format!("{:?}", std::thread::current().id());
        let file = File::create(test_path).unwrap();
        let mut writer = BufWriter::new(file);

        row.write_row(&mut writer).unwrap();
        writer.flush().unwrap();
        let contents = std::fs::read_to_string(test_path).unwrap();
        std::fs::remove_file(test_path).unwrap();

        assert_eq!(contents, format!("{},{}\n", VALUE1, VALUE2));
    }

    #[test]
    fn test_update_row() {
        let mut row_not_to_update = create_row_with_columns();
        let mut row_to_update = create_row_with_values();

        let test_path = &format!("{:?}", std::thread::current().id());
        let file = File::create(test_path).unwrap();
        let mut writer = BufWriter::new(file);

        let mut update_values = HashMap::new();
        update_values.insert(COLULMN1.to_string(), NEWVALUE1.to_string());
        let condition = Expression::Comparison {
            left: Operand::Column(COLULMN1.to_string()),
            operator: "=".to_string(),
            right: Operand::String(VALUE1.to_string()),
        };

        row_not_to_update
            .update_row(&update_values, &condition, &mut writer)
            .unwrap();
        row_to_update
            .update_row(&update_values, &condition, &mut writer)
            .unwrap();
        writer.flush().unwrap();
        let contents = std::fs::read_to_string(test_path).unwrap();
        std::fs::remove_file(test_path).unwrap();

        assert_eq!(
            contents,
            format!("{},{}\n{},{}\n", COLULMN1, COLUMN2, NEWVALUE1, VALUE2)
        );
    }

    #[test]
    fn test_delete_row() {
        let row_not_to_delete = create_row_with_columns();
        let row_to_delete = create_row_with_values();

        let test_path = &format!("{:?}", std::thread::current().id());
        let file = File::create(test_path).unwrap();
        let mut writer = BufWriter::new(file);

        let condition = Expression::Comparison {
            left: Operand::Column(COLULMN1.to_string()),
            operator: "=".to_string(),
            right: Operand::String(VALUE1.to_string()),
        };

        row_not_to_delete
            .delete_row(&condition, &mut writer)
            .unwrap();
        row_to_delete.delete_row(&condition, &mut writer).unwrap();
        writer.flush().unwrap();
        let contents = std::fs::read_to_string(test_path).unwrap();
        std::fs::remove_file(test_path).unwrap();

        assert_eq!(contents, format!("{},{}\n", COLULMN1, COLUMN2));
    }

    #[test]
    fn test_check_condition() {
        let row_true = create_row_with_values();
        let row_false = create_row_with_columns();
        let condition = Expression::Comparison {
            left: Operand::Column(COLULMN1.to_string()),
            operator: "=".to_string(),
            right: Operand::String(VALUE1.to_string()),
        };

        let result_true = row_true.check_condition(&condition).unwrap();
        let result_false = row_false.check_condition(&condition).unwrap();

        assert_eq!(result_true, true);
        assert_eq!(result_false, false);
    }

    #[test]
    fn test_hashmap() {
        let columns = vec![COLULMN1.to_string(), COLUMN2.to_string()];
        let mut values = HashMap::new();
        let mut same_values: HashMap<String, String> = HashMap::new();
        values.insert(COLULMN1.to_string(), VALUE1.to_string());
        values.insert(COLUMN2.to_string(), VALUE2.to_string());
        same_values.insert(COLULMN1.to_string(), VALUE1.to_string());
        same_values.insert(COLUMN2.to_string(), VALUE2.to_string());
        let row = Row::new(&columns, values);

        let hashmap = row.hashmap();

        assert_eq!(hashmap, same_values);
    }
}
