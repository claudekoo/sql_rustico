use super::custom_error::CustomError;
use super::expression::{Expression, evaluate_expression};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
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
    pub fn new(columns: &Vec<String>, values: HashMap<String, String>) -> Row {
        Row {
            columns: columns.clone(),
            values,
        } // TODO: Borrar clone
    }

    // Writes row as it is to a file
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

    // Writes row to a file after updating it given a condition
    pub fn update_row(
        &mut self,
        update_values: &HashMap<String, String>,
        condition: &Expression,
        writer: &mut BufWriter<File>,
    ) -> Result<(), CustomError> {
        let result = evaluate_expression(condition, &self.values)?;
        if result == true {
            for column_to_update in update_values.keys() {
                update_if_present(&mut self.values, column_to_update, update_values[column_to_update].as_str())?;
            }
        }
        self.write_row(writer)?;
        Ok(())
        }

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

    pub fn select_row(
        &self,
        columns: &Vec<String>,
        condition: &Expression,
    ) -> Result<(), CustomError> {
        Ok(())
    }
}

fn update_if_present(map: &mut HashMap<String, String>, key: &str, value: &str) -> Result<(), CustomError> {
    if let Some(_) = map.get(key) {
        map.insert(key.to_string(), value.to_string());
        Ok(())
    } else {
        Err(CustomError::InvalidColumn {
            message: "Column does not exist".to_string(),
        })
    }
}