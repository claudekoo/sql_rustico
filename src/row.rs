use super::custom_error::CustomError;
use super::expression::{evaluate_expression, Expression};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};

/// Una fila en esta implementación es un conjunto de valrores asociados a columnas. Convenientemente tiene un vector de columnas además de un HashMap de valores para tener referencias a columnas que no existen en su tabla de valores.
/// Puede escribirse en un archivo CSV actualizando su estado según una condición dada.
pub struct Row {
    columns: Vec<String>,
    values: HashMap<String, String>,
}

fn write_result(writer: &mut BufWriter<File>, string: &str) -> Result<(), CustomError> {
    if let Err(error) = write!(writer, "{}", string) {
        return Err(CustomError::GenericError {
            message: format!("Error writing to file: {}", error),
        });
    }
    Ok(())
}

impl Row {
    /// Crea una nueva fila dado un vector de columnas y un HashMap de valores.
    pub fn new(columns: &Vec<String>, values: HashMap<String, String>) -> Row {
        let mut new_columns = Vec::new();
        for item in columns.iter() {
            new_columns.push(item.to_string());
        }
        Row {
            columns: new_columns,
            values,
        }
    }

    /// Se escribe a un archivo CSV.
    pub fn write_row(&self, writer: &mut BufWriter<File>) -> Result<(), CustomError> {
        let last_index = self.columns.len() - 1;
        let mut actual_index = 0;
        for column in &self.columns {
            let value_option = self.values.get(column);
            if let Some(value) = value_option {
                write_result(writer, value)?;
            } else {
                write_result(writer, "")?;
            }
            if actual_index != last_index {
                write_result(writer, ",")?;
            } else {
                write_result(writer, "\n")?;
            }
            actual_index += 1;
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
        let result = evaluate_expression(condition, &self.values)?;
        if result == true {
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
        let result: bool = evaluate_expression(condition, &self.values)?;
        if result == false {
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
    if let Some(_) = map.get(key) {
        map.insert(key.to_string(), value.to_string());
        Ok(())
    } else {
        Err(CustomError::InvalidColumn {
            message: "Column does not exist".to_string(),
        })
    }
}
