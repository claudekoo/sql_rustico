use std::env;
mod command_parser;
mod command_processer;
mod custom_error;
mod expression;
mod expression_parser;
mod row;
mod row_parser;
mod tokenizer;
use command_processer::process_command;
use custom_error::CustomError;

/// Recibe los argumentos de la línea de comandos y los procesa.
/// Se espera como argumentos el directorio de las tablas y el comando SQL a ejecutar.
///
/// # Ejemplo
/// ```sh
/// cargo run tables/ "SELECT * FROM table1;"
/// ```
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
