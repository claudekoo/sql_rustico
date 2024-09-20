use sql_rustico::command_processer::process_command;
use std::fs::File;
use std::io::Write;

#[test]
fn test_process_command_with_invalid_directory() {
    let args = vec![
        "sql".to_string(),
        "non_existent/".to_string(),
        "SELECT * FROM table1;".to_string(),
    ];
    let mut unused_output = vec![];
    let result = process_command(&args, &mut unused_output);
    assert!(result.is_err());
}

#[test]
fn test_process_command_with_invalid_command() {
    let args = vec![
        "sql".to_string(),
        "tables/".to_string(),
        "INVALID COMMAND".to_string(),
    ];
    let mut unused_output = vec![];
    let result = process_command(&args, &mut unused_output);
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
    let mut unused_output = vec![];
    let result = process_command(&args, &mut unused_output);
    assert!(result.is_ok());
    let contents = std::fs::read_to_string(&file_path).unwrap();
    assert_eq!(contents, "column1,column2\nvalue1,value2\n");
    std::fs::remove_file(file_path).expect("Error deleting file");
    std::fs::remove_dir(table_dir).expect("Error deleting directory");
}

#[test]
fn test_process_command_with_insert_columns_with_spaces() {
    let table_dir = "test_table_insert_columns_with_spaces/";
    let table_name = "tempProcessInsertColumnsWithSpaces";
    let file_path = format!("{}{}.csv", table_dir, table_name);
    std::fs::create_dir_all(table_dir).expect("Error creating directory");
    let mut file = File::create(&file_path).expect("Error creating temp file");
    writeln!(file, "column1 with spaces,column2 with spaces").expect("Error writing to temp file");
    let args = vec![
        "sql".to_string(),
        "test_table_insert_columns_with_spaces/".to_string(),
        format!(
            "INSERT INTO {} ('column1 with spaces', 'column2 with spaces') VALUES ('value1', 'value2');",
            table_name
        ),
    ];
    let mut unused_output = vec![];
    let result = process_command(&args, &mut unused_output);
    assert!(result.is_ok());
    let contents = std::fs::read_to_string(&file_path).unwrap();
    assert_eq!(
        contents,
        "column1 with spaces,column2 with spaces\nvalue1,value2\n"
    );
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
    let mut unused_output = vec![];
    let result = process_command(&args, &mut unused_output);
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
    let mut unused_output = vec![];
    let result = process_command(&args, &mut unused_output);
    assert!(result.is_ok());
    let contents = std::fs::read_to_string(&file_path).unwrap();
    assert_eq!(contents, "column1,column2\nvalue3,value4\n");
    std::fs::remove_file(file_path).expect("Error deleting file");
    std::fs::remove_dir(table_dir).expect("Error deleting directory");
}

#[test]
fn test_process_command_with_select() {
    let table_dir = "test_table_select/";
    let table_name = "tempProcessSelect";
    let file_path = format!("{}{}.csv", table_dir, table_name);
    std::fs::create_dir_all(table_dir).expect("Error creating directory");
    let mut file = File::create(&file_path).expect("Error creating temp file");
    writeln!(file, "column1,column2").expect("Error writing to temp file");
    writeln!(file, "value1,value2").expect("Error writing to temp file");
    writeln!(file, "value3,value4").expect("Error writing to temp file");
    let args = vec![
        "sql".to_string(),
        "test_table_select/".to_string(),
        format!(
            "SELECT column2 FROM {} WHERE column1 = 'value1';",
            table_name
        ),
    ];
    let mut output = vec![];
    let result = process_command(&args, &mut output);
    assert!(result.is_ok());
    let output_as_str = String::from_utf8(output).unwrap();
    assert_eq!(output_as_str, "column2\nvalue2\n");
    std::fs::remove_file(file_path).expect("Error deleting file");
    std::fs::remove_dir(table_dir).expect("Error deleting directory");
}
