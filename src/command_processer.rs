use crate::row_parser;
use super::row::Row;
use super::custom_error::CustomError;
use super::expression::Expression;
use super::command_parser::{parse_insert, parse_update, parse_delete, parse_select};
use super::tokenizer::{tokenize, Token};
use std::collections::HashMap;
use std::vec;
use std::path::Path;
use std::fs::{self, ReadDir, File};
use std::io::{BufRead, BufWriter};

// recibo y parseo el input a struct
// cargo los archivos en cuestion linea a linea
// parseo los archivos a struct table
//              vector(columnas) y vector de vectores(valores)
// ejecuto la instruccion dada
// parser los escribe en formato csv

// Recibe un vector de argumentos y devuelve un Result: Ok(()) o Err(CustomError)
pub fn process_command(args: &Vec<String>) -> Result<(), CustomError> {
    let tokens = tokenize(args[2].as_str())?;
    let directory = Path::new(args[1].as_str());
    if let Some(Token::Keyword(keyword)) = tokens.get(0) {
        match keyword.as_str() {
            "INSERT" => {
                return process_insert(&tokens, directory);
            }
            "UPDATE" => {
                return process_update(&tokens, directory);
            }
            "DELETE" => {
                return process_delete(&tokens, directory);
            }
            "SELECT" => {
                return process_select(&tokens, directory);
            }
            other => {
                return Err(CustomError::InvalidSyntax {
                    message: format!("Invalid command: {}", other),
                });
            }
        }
    } else {
        return Err(CustomError::InvalidSyntax {
            message: ("Usage: <COMMAND> <...>".to_string()),
        });
    }
}

fn process_insert(tokens: &Vec<Token>, directory: &Path) -> Result<(), CustomError> {
    let mut table_name = String::new();
    let mut columns = vec![];
    let mut values = vec![];
    parse_insert(tokens, &mut table_name, &mut columns, &mut values)?;
    let table_path = find_table_csv(Path::new(directory), table_name.as_str())?;
    table_name.push_str(".csv");
    let tmp_path = table_path.trim_end_matches(table_name.as_str()).to_string() + "_tmp.csv";
    let create_file_result = fs::File::create(&tmp_path);
    if let Err(_) = create_file_result {
        return Err(CustomError::GenericError {
            message: "Couldn't create tmp file".to_string(),
        });
    }
    if let Ok(tmp_file) = create_file_result {
        let mut writer = BufWriter::new(tmp_file);
        let full_columns = copy_table(table_path.as_str(), &mut writer)?;
        for new_value in values {
            let new_row = Row::new(&full_columns, new_value);
            new_row.write_row(&mut writer)?;
        }
    }
    remove_file(&table_path)?;
    rename_file(&tmp_path, &table_path)?;
    Ok(())
}

fn remove_file(file_path: &str) -> Result<(), CustomError> {
    let remove_file_result = fs::remove_file(file_path);
    if let Err(_) = remove_file_result {
        return Err(CustomError::GenericError {
            message: "Couldn't remove file".to_string(),
        });
    }
    Ok(())
}

fn rename_file(from: &str, to: &str) -> Result<(), CustomError> {
    let rename_file_result = fs::rename(from, to);
    if let Err(_) = rename_file_result {
        return Err(CustomError::GenericError {
            message: "Couldn't rename file".to_string(),
        });
    }
    Ok(())
}

fn process_update(tokens: &Vec<Token>, directory: &Path) -> Result<(), CustomError> {
    let mut table_name = String::new();
    let mut set_values = HashMap::new();
    let mut condition = Expression::True;
    parse_update(tokens, &mut table_name, &mut set_values, &mut condition)?;
    let table_path = find_table_csv(Path::new(directory), table_name.as_str())?;
    Ok(())
}

fn process_delete(tokens: &Vec<Token>, directory: &Path) -> Result<(), CustomError> {
    let mut table_name = String::new();
    let mut condition = Expression::True;
    parse_delete(tokens, &mut table_name, &mut condition)?;
    let table_path = find_table_csv(Path::new(directory), table_name.as_str())?;
    Ok(())
}

fn process_select(tokens: &Vec<Token>, directory: &Path) -> Result<(), CustomError> {
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
    )?;
    let table_path = find_table_csv(Path::new(directory), table_name.as_str())?;
    Ok(())
}

fn find_table_csv(directory: &Path, table_name: &str) -> Result<String, CustomError> {
    let open_dir_result = fs::read_dir(directory); // abro el directorio
    if let Err(_) = open_dir_result {
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
        if let Err(_) = entry {
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
                        if let None = entry_path_to_str_option {
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

fn copy_table(table_path: &str, writer: &mut BufWriter<File>) -> Result<(Vec<String>), CustomError> {
    let table_file_result = fs::File::open(table_path);
    if let Err(_) = table_file_result {
        return Err(CustomError::GenericError {
            message: "Couldn't open table file".to_string(),
        });
    }
    let mut columns: Vec<String> = vec![];
    if let Ok(table_file) = table_file_result {
        let table_reader = std::io::BufReader::new(table_file);
        let mut first_line = true;
        for line in table_reader.lines() {
            if let Err(_) = line {
                return Err(CustomError::GenericError {
                    message: "Couldn't read table file".to_string(),
                });
            }
            if let Ok(line) = line {
                if first_line {
                    first_line = false;
                    columns = line.split(",").map(|s| s.to_string()).collect();
                } 
                let row = row_parser::parse_row(&columns, line.as_str())?;
                row.write_row(writer)?;
            }
        }   
    }
    Ok(columns)
}
