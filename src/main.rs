use sql_rustico::command_processer::process_command;
use sql_rustico::custom_error::CustomError;
use std::env;

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
    let mut stdout = std::io::stdout();
    let command_process_result = process_command(&args, &mut stdout);
    if let Err(error) = command_process_result {
        println!("{}", error);
    }
}
