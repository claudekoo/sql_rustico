use super::command_parser::{parse_delete, parse_insert, parse_select, parse_update};
use super::custom_error::CustomError;
use super::expression::Expression;
use super::row::Row;
use super::tokenizer::{tokenize, Token};
use crate::row_parser::{parse_columns, parse_row};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions, ReadDir};
use std::io::{BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;

// Recibe un vector de argumentos y devuelve un Result: Ok(()) o Err(CustomError)
/// Procesa el comando recibido recibiendo un vector de argumentos, donde el primer argumento es el directorio de los archivos csv, y el segundo argumento es el comando a procesar.
pub fn process_command(args: &[String]) -> Result<(), CustomError> {
    let tokens = tokenize(args[2].as_str())?;
    let directory = Path::new(args[1].as_str());
    if let Some(Token::Keyword(keyword)) = tokens.first() {
        match keyword.as_str() {
            "INSERT" => process_insert(&tokens, directory),
            "UPDATE" => process_update(&tokens, directory),
            "DELETE" => process_delete(&tokens, directory),
            "SELECT" => process_select(&tokens, directory),
            other => CustomError::error_invalid_syntax(&format!("Invalid command: {}", other)),
        }
    } else {
        CustomError::error_invalid_syntax("Usage: <COMMAND> <...>")
    }
}

fn create_file(file_path: &str) -> Result<File, CustomError> {
    let create_file_result = fs::File::create(file_path);
    if let Ok(file) = create_file_result {
        return Ok(file);
    }
    Err(CustomError::GenericError {
        message: "Couldn't create file".to_string(),
    })
}

fn process_insert(tokens: &[Token], directory: &Path) -> Result<(), CustomError> {
    let mut table_name = String::new();
    let mut columns = vec![];
    let mut values = vec![];
    parse_insert(tokens, &mut table_name, &mut columns, &mut values)?; // parseo los tokens
    let table_path = find_table_csv(Path::new(directory), table_name.as_str())?; // busco la tabla
    table_name.push_str(".csv");
    let table_file = open_table_path(&table_path)?;
    let mut table_reader = BufReader::new(table_file);
    let mut line = String::new();
    let full_columns: Vec<String> = if table_reader.read_line(&mut line).is_ok() {
        // leo la primera linea para obtener las columnas
        parse_columns(&line)?
    } else {
        return CustomError::error_invalid_table("Couldn't read table file");
    };

    if let Ok(file) = OpenOptions::new().append(true).open(&table_path) {
        let mut writer = BufWriter::new(file);
        add_newline_if_needed(&mut writer, &mut table_reader)?;
        for new_value in values {
            // escribo cada valor nuevo
            let row = Row::new(&full_columns, new_value);
            row.write_row(&mut writer)?;
        }
    } else {
        return CustomError::error_invalid_table("Couldn't open table file");
    }
    Ok(())
}

fn add_newline_if_needed(
    writer: &mut BufWriter<File>,
    reader: &mut BufReader<File>,
) -> Result<(), CustomError> {
    if reader.seek(SeekFrom::End(-1)).is_err() {
        return CustomError::error_invalid_table("Couldn't read end of table file");
    }
    let mut buffer = [0; 1];
    if reader.read_exact(&mut buffer).is_ok() {
        if buffer[0] != b'\n'  && writeln!(writer).is_err() {
            return CustomError::error_invalid_table("Couldn't add newline to table file");
        }
    } else {
        return CustomError::error_invalid_table("Couldn't read end of table file");
    }
    Ok(())
}

fn remove_file(file_path: &str) -> Result<(), CustomError> {
    let remove_file_result = fs::remove_file(file_path);
    if remove_file_result.is_err() {
        return CustomError::error_generic("Couldn't remove file");
    }
    Ok(())
}

fn rename_file(from: &str, to: &str) -> Result<(), CustomError> {
    let rename_file_result = fs::rename(from, to);
    if rename_file_result.is_err() {
        return CustomError::error_generic("Couldn't rename file");
    }
    Ok(())
}

fn process_update(tokens: &[Token], directory: &Path) -> Result<(), CustomError> {
    let mut table_name = String::new();
    let mut set_values = HashMap::new();
    let mut condition = Expression::True;
    parse_update(tokens, &mut table_name, &mut set_values, &mut condition)?; // parseo los tokens
    let table_path = find_table_csv(Path::new(directory), table_name.as_str())?;
    table_name.push_str(".csv");
    let tmp_path = table_path.trim_end_matches(table_name.as_str()).to_string() + "_tmp.csv"; // creo el path del archivo temporal
    let tmp_file = create_file(&tmp_path)?; // creo el archivo temporal
    let mut writer = BufWriter::new(tmp_file);
    update_table(table_path.as_str(), &mut writer, &condition, &set_values)?;
    remove_file(&table_path)?;
    rename_file(&tmp_path, &table_path)?;
    Ok(())
}

fn process_delete(tokens: &[Token], directory: &Path) -> Result<(), CustomError> {
    let mut table_name = String::new();
    let mut condition = Expression::True;
    parse_delete(tokens, &mut table_name, &mut condition)?; // parseo los tokens
    let table_path = find_table_csv(Path::new(directory), table_name.as_str())?;
    table_name.push_str(".csv");
    let tmp_path = table_path.trim_end_matches(table_name.as_str()).to_string() + "_tmp.csv"; // creo el path del archivo temporal
    let tmp_file = create_file(&tmp_path)?; // creo el archivo temporal
    let mut writer = BufWriter::new(tmp_file);
    delete_rows_table(table_path.as_str(), &mut writer, &condition)?;
    remove_file(&table_path)?;
    rename_file(&tmp_path, &table_path)?;
    Ok(())
}

fn process_select(tokens: &[Token], directory: &Path) -> Result<(), CustomError> {
    let mut columns = vec![];
    let mut table_name = String::new();
    let mut condition = Expression::True;
    let mut order_by = vec![];
    parse_select(
        tokens,
        &mut columns,
        &mut table_name,
        &mut condition,
        &mut order_by,
    )?; // parseo los tokens
    let table_path = find_table_csv(Path::new(directory), table_name.as_str())?;
    select_rows_table(table_path.as_str(), &condition, &mut columns, &order_by)?;
    Ok(())
}

fn find_table_csv(directory: &Path, table_name: &str) -> Result<String, CustomError> {
    let open_dir_result = fs::read_dir(directory); // abro el directorio
    if open_dir_result.is_err() {
        // si no se pudo abrir devuelvo error
        return Err(CustomError::GenericError {
            message: format!("Couldn't open directory {:?}", directory),
        });
    }
    if let Ok(open_dir) = open_dir_result {
        // si se pudo abrir, recorro los contenidos
        return handle_dir(table_name, open_dir);
    }
    Err(CustomError::GenericError {
        message: "File not found".to_string(),
    })
}

fn handle_dir(table_name: &str, open_dir: ReadDir) -> Result<String, CustomError> {
    for entry in open_dir {
        if entry.is_err() {
            // si no se pudo abrir el archivo, devuelvo error
            return Err(CustomError::GenericError {
                message: "Couldn't open directory".to_string(),
            });
        }
        if let Ok(entry) = entry {
            // si se pudo abrir
            let entry_path = entry.path();
            if entry_path.is_dir() {
                // si es un directorio, llamo recursivamente
                if let Ok(found) = find_table_csv(entry_path.as_path(), table_name) {
                    return Ok(found); // si se encontro en ese directorio, devuelvo ese path
                }
            } else if entry_path.is_file() {
                // si es un archivo, verifico si es el que busco
                let file_name = entry_path.file_name();
                if let Some(file_name) = file_name {
                    // si lo es, devuelvo el path
                    if file_name.to_str() == Some(format!("{}.csv", table_name).as_str()) {
                        let entry_path_to_str_option = entry_path.to_str();
                        if let Some(entry_path_to_str) = entry_path_to_str_option {
                            return Ok(entry_path_to_str.to_string());
                        }
                        if entry_path_to_str_option.is_none() {
                            return Err(CustomError::GenericError {
                                message: "Couldn't convert path to string".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }
    // si no se encontro en el loop, devuelvo error
    Err(CustomError::InvalidTable {
        message: "File not found".to_string(),
    })
}

fn open_table_path(table_path: &str) -> Result<File, CustomError> {
    let table_file_result = fs::File::open(table_path);
    if let Ok(table_file) = table_file_result {
        return Ok(table_file);
    }
    Err(CustomError::GenericError {
        message: "Couldn't open table file".to_string(),
    })
}

fn update_table(
    table_path: &str,
    writer: &mut BufWriter<File>,
    condition: &Expression,
    update_values: &HashMap<String, String>,
) -> Result<(), CustomError> {
    let table_file = open_table_path(table_path)?;
    let mut columns: Vec<String> = vec![];
    let table_reader = std::io::BufReader::new(table_file);
    let mut first_line = true; // flag para saber si es la primera linea = columnas
    for line in table_reader.lines() {
        if line.is_err() {
            return CustomError::error_generic("Couldn't read table file");
        }
        if let Ok(line) = line {
            if first_line {
                first_line = false;
                columns = line.split(",").map(|s| s.to_string()).collect();
                let row = parse_row(&columns, line.as_str())?;
                row.write_row(writer)?;
                continue;
            }
            let mut row = parse_row(&columns, line.as_str())?;
            row.update_row(update_values, condition, writer)?;
        }
    }
    Ok(())
}

fn delete_rows_table(
    table_path: &str,
    writer: &mut BufWriter<File>,
    condition: &Expression,
) -> Result<(), CustomError> {
    let table_file = open_table_path(table_path)?;
    let mut columns: Vec<String> = vec![];
    let table_reader = std::io::BufReader::new(table_file);
    let mut first_line = true; // flag para saber si es la primera linea = columnas
    for line in table_reader.lines() {
        if line.is_err() {
            return CustomError::error_generic("Couldn't read table file");
        }
        if let Ok(line) = line {
            if first_line {
                first_line = false;
                columns = line.split(",").map(|s| s.to_string()).collect();
                let row = parse_row(&columns, line.as_str())?;
                row.write_row(writer)?;
                continue;
            }
            let row = parse_row(&columns, line.as_str())?;
            row.delete_row(condition, writer)?;
        }
    }
    Ok(())
}

fn select_rows_default(
    table_reader: BufReader<File>,
    condition: &Expression,
    columns_to_print: &[String],
) -> Result<(), CustomError> {
    let mut first_line = true;
    let mut full_columns: Vec<String> = vec![];
    for line in table_reader.lines() {
        if line.is_err() {
            return Err(CustomError::GenericError {
                message: "Couldn't read table file".to_string(),
            });
        }
        if let Ok(line) = line {
            if first_line {
                // si es la primera linea, guardo las columnas
                first_line = false;
                full_columns = line.split(",").map(|s| s.to_string()).collect();
                let row = parse_row(&full_columns, line.as_str())?;
                row.print_row(columns_to_print)?;
                continue;
            }
            let row = parse_row(&full_columns, line.as_str())?;
            let selected = row.check_condition(condition)?;
            if selected {
                row.print_row(columns_to_print)?;
            }
        }
    }
    Ok(())
}

fn select_rows_ordered(
    table_reader: BufReader<File>,
    condition: &Expression,
    columns_to_print: &mut Vec<String>,
    order_by: &[(String, String)],
) -> Result<(), CustomError> {
    let mut first_line = true; // flag para saber si es la primera linea = columnas
    let mut selected_rows = vec![];
    let mut full_columns: Vec<String> = vec![];
    for line in table_reader.lines() {
        if line.is_err() {
            return Err(CustomError::GenericError {
                message: "Couldn't read table file".to_string(),
            });
        }
        if let Ok(line) = line {
            if first_line {
                first_line = false;
                full_columns = line.split(",").map(|s| s.to_string()).collect();
                if columns_to_print.is_empty() {
                    for column in &full_columns {
                        columns_to_print.push(column.to_string());
                    }
                }
                let row = parse_row(&full_columns, line.as_str())?;
                row.print_row(columns_to_print)?;
                continue;
            }
            let row = parse_row(&full_columns, line.as_str())?;
            let selected: bool = row.check_condition(condition)?;
            if selected {
                selected_rows.push(row);
            }
        }
    }
    order_rows(&mut selected_rows, order_by)?;
    for row in selected_rows {
        row.print_row(columns_to_print)?;
    }
    Ok(())
}

fn order_rows(rows: &mut [Row], order_by: &[(String, String)]) -> Result<(), CustomError> {
    for (column, order) in order_by.iter().rev() {
        rows.sort_by(|a, b| {
            if order == "ASC" {
                a.cmp_by_column(column, b)
            } else {
                b.cmp_by_column(column, a)
            }
        });
    }
    Ok(())
}

fn select_rows_table(
    table_path: &str,
    condition: &Expression,
    columns_to_print: &mut Vec<String>,
    order_by: &[(String, String)],
) -> Result<(), CustomError> {
    let table_file = open_table_path(table_path)?;
    let table_reader = std::io::BufReader::new(table_file);
    if order_by.is_empty() {
        select_rows_default(table_reader, condition, columns_to_print)?;
    } else {
        select_rows_ordered(table_reader, condition, columns_to_print, order_by)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_process_command_with_invalid_directory() {
        let args = vec![
            "sql".to_string(),
            "non_existent/".to_string(),
            "SELECT * FROM table1;".to_string(),
        ];
        let result = process_command(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_command_with_invalid_command() {
        let args = vec![
            "sql".to_string(),
            "tables/".to_string(),
            "INVALID COMMAND".to_string(),
        ];
        let result = process_command(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_command_with_insert() {
        let table_dir = "test_table_insert/";
        let table_name = "tempProcessInsert";
        let file_path = format!("{}{}.csv", table_dir, table_name);
        std::fs::create_dir_all(table_dir).expect("Error creating directory");
        let mut file = File::create(&file_path).expect("Error creating temp file");
        writeln!(file, "column1,column2").expect("Error writing to temp file");
        let args = vec![
            "sql".to_string(),
            "test_table_insert/".to_string(),
            format!(
                "INSERT INTO {} (column1, column2) VALUES ('value1', 'value2');",
                table_name
            ),
        ];

        let result = process_command(&args);

        println!("{:?}", result);
        assert!(result.is_ok());
        let contents = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(contents, "column1,column2\nvalue1,value2\n");

        std::fs::remove_file(file_path).expect("Error deleting file");
        std::fs::remove_dir(table_dir).expect("Error deleting directory");
    }

    #[test]
    fn test_process_command_with_update() {
        let table_dir = "test_table_update/";
        let table_name = "tempProcessUpdate";
        let file_path = format!("{}{}.csv", table_dir, table_name);
        std::fs::create_dir_all(table_dir).expect("Error creating directory");
        let mut file = File::create(&file_path).expect("Error creating temp file");
        writeln!(file, "column1,column2").expect("Error writing to temp file");
        writeln!(file, "value1,value2").expect("Error writing to temp file");
        writeln!(file, "value3,value4").expect("Error writing to temp file");
        let args = vec![
            "sql".to_string(),
            "test_table_update/".to_string(),
            format!(
                "UPDATE {} SET column1 = 'new_value1' WHERE column1 = 'value1';",
                table_name
            ),
        ];

        let result = process_command(&args);

        assert!(result.is_ok());
        let contents = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(
            contents,
            "column1,column2\nnew_value1,value2\nvalue3,value4\n"
        );

        std::fs::remove_file(file_path).expect("Error deleting file");
        std::fs::remove_dir(table_dir).expect("Error deleting directory");
    }

    #[test]
    fn test_process_command_with_delete() {
        let table_dir = "test_table_delete/";
        let table_name = "tempProcessDelete";
        let file_path = format!("{}{}.csv", table_dir, table_name);
        std::fs::create_dir_all(table_dir).expect("Error creating directory");
        let mut file = File::create(&file_path).expect("Error creating temp file");
        writeln!(file, "column1,column2").expect("Error writing to temp file");
        writeln!(file, "value1,value2").expect("Error writing to temp file");
        writeln!(file, "value3,value4").expect("Error writing to temp file");
        let args = vec![
            "sql".to_string(),
            "test_table_delete/".to_string(),
            format!("DELETE FROM {} WHERE column1 = 'value1';", table_name),
        ];

        let result = process_command(&args);

        assert!(result.is_ok());
        let contents = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(contents, "column1,column2\nvalue3,value4\n");

        std::fs::remove_file(file_path).expect("Error deleting file");
        std::fs::remove_dir(table_dir).expect("Error deleting directory");
    }

    // #[test]
    // fn test_process_command_with_select() {
    //     let table_dir = "test_table_select/";
    //     let table_name = "tempProcessSelect";
    //     let file_path = format!("{}{}.csv", table_dir, table_name);
    //     std::fs::create_dir_all(table_dir).expect("Error creating directory");
    //     let mut file = File::create(&file_path).expect("Error creating temp file");
    //     writeln!(file, "column1,column2").expect("Error writing to temp file");
    //     writeln!(file, "value1,value2").expect("Error writing to temp file");
    //     writeln!(file, "value3,value4").expect("Error writing to temp file");
    //     let args = vec![
    //         "sql".to_string(),
    //         "test_table_select/".to_string(),
    //         format!("SELECT column2 FROM {} WHERE column1 = 'value1';", table_name),
    //     ];

    //     let result = process_command(&args);

    //     assert!(result.is_ok());
    //     let stdout_output = std::io::stdout();
    //     assert_eq!(stdout_output, "column2\nvalue2\n");

    //     std::fs::remove_file(file_path).expect("Error deleting file");
    //     std::fs::remove_dir(table_dir).expect("Error deleting directory");
    // }
}
