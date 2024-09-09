use std::env;
mod command_processer;
use command_processer::process_command;
mod custom_error;
use custom_error::CustomError;
mod command_parser;
mod expression;
mod expression_parser;
mod row;
mod row_parser;
mod tokenizer;

// recibo y parseo el input a struct
// cargo los archivos en cuestion linea a linea
// parseo los archivos a struct table
//              vector(columnas) y vector de vectores(valores)
// ejecuto la instruccion dada
// parser los escribe en formato csv

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!(
            "{}",
            CustomError::GenericError {
                message: ("No command provided".to_string()),
            }
        );
    }
    let command_process_result = process_command(&args);
    if let Err(error) = command_process_result {
        println!("{}", error);
    }
}
