use super::custom_error::CustomError;
use super::expression::Expression;
use super::expression_parser::parse_expression;
use super::tokenizer::Token;
use std::collections::HashMap;
use std::iter::Peekable;
use std::slice::Iter;

/// Parsea un comando INSERT que llega en forma de vector de tokens.
/// Modifica los parametros table_name, columns y values.
///
/// El formato del comando INSERT esperado es:
/// INSERT INTO <table_name> (<column1>, <column2>, ...) VALUES (<value1>, <value2>, ...);
pub fn parse_insert(
    tokens: &[Token],
    table_name: &mut String,
    columns: &mut Vec<String>,
    values: &mut Vec<HashMap<String, String>>,
) -> Result<(), CustomError> {
    let mut iter = tokens.iter().peekable();
    iter.next(); // salteo el INSERT
    parse_insert_into(&mut iter, table_name)?;
    parse_insert_into_columns(columns, &mut iter)?;
    parse_insert_values(values, &mut iter, columns)?;
    check_ending_with_semicolon(&mut iter)?;
    Ok(())
}

fn parse_insert_into(
    iter: &mut Peekable<Iter<Token>>,
    table_name: &mut String,
) -> Result<(), CustomError> {
    if !matches!(iter.next(), Some(Token::Keyword(keyword)) if keyword.as_str() == "INTO") {
        // Verifico que haya INTO
        return CustomError::error_invalid_syntax("Expected INTO after INSERT");
    }
    if let Some(Token::Identifier(name)) | Some(Token::String(name)) = iter.next() {
        // Verifico que haya nombre de tabla
        *table_name = name.to_string();
    } else {
        return CustomError::error_invalid_syntax("Expected table name after INTO");
    }
    Ok(())
}

fn parse_insert_into_columns(
    columns: &mut Vec<String>,
    iter: &mut Peekable<Iter<Token>>,
) -> Result<(), CustomError> {
    if let Some(Token::Symbol('(')) = iter.next() {
        // Verifico que se abra parentesis
        while let Some(token) = iter.next() {
            // Este ciclo termina al encontrar un ')'
            match token {
                Token::Identifier(name) | Token::String(name) => {
                    // Si es nombre de columna, lo agrego
                    columns.push(name.to_string());
                    if let Some(Token::Symbol(')')) | Some(Token::Symbol(',')) = iter.peek() {
                    } else {
                        return CustomError::error_invalid_syntax(
                            "Expected ',' or ')' after column name",
                        );
                    }
                }
                Token::Symbol(',') => {
                    // Si es coma, verifico que su siguiente sea nombre de columna
                    if let Some(Token::Identifier(_)) = iter.peek() {
                    } else {
                        return CustomError::error_invalid_syntax("Expected column name after ','");
                    }
                }
                Token::Symbol(')') => {
                    // Si se cierra parentesis, termino
                    break;
                }
                _ => {
                    // Si no es un token esperado, devuelvo error
                    return CustomError::error_invalid_syntax(
                        "Expected column name or ')' after '('",
                    );
                }
            }
        }
    } else {
        return CustomError::error_invalid_syntax("Expected '(' after table name");
    }
    Ok(())
}

fn parse_insert_values(
    values: &mut Vec<HashMap<String, String>>,
    iter: &mut Peekable<Iter<Token>>,
    columns: &[String],
) -> Result<(), CustomError> {
    if !matches!(iter.next(), Some(Token::Keyword(keyword)) if keyword.as_str() == "VALUES") {
        // Verifico que haya VALUES
        return CustomError::error_invalid_syntax("Expected VALUES after column names");
    }
    parse_insert_value(values, iter, columns)?; // Parseo el primer valor
    while let Some(Token::Symbol(',')) = iter.peek() {
        // Si lo sigue una coma, parseo otro valor
        iter.next();
        parse_insert_value(values, iter, columns)?;
    }
    Ok(())
}

fn parse_insert_value(
    values: &mut Vec<HashMap<String, String>>,
    iter: &mut Peekable<Iter<Token>>,
    columns: &[String],
) -> Result<(), CustomError> {
    let mut row: HashMap<String, String> = HashMap::new(); // Hashmap de un VALUE para devolver: columna -> valor
    if let Some(Token::Symbol('(')) = iter.next() {
        // Verifico que se abra parentesis
        let mut column_index = 0; // Indice de la columna actual
        while let Some(token) = iter.next() {
            // Este ciclo termina al encontrar un ')'
            match token {
                Token::Integer(_) | Token::String(_) => {
                    // Si es un valor, lo agrego al hashmap
                    if let Some(Token::Symbol(')')) | Some(Token::Symbol(',')) = iter.peek() {
                    } else {
                        return CustomError::error_invalid_syntax(
                            "Expected ',' or ')' after value",
                        );
                    }
                    if column_index >= columns.len() {
                        // Si hay mas valores que columnas, devuelvo error
                        return CustomError::error_invalid_syntax("Too many values for columns");
                    }
                    let value = match token {
                        Token::Integer(int) => int.to_string(),
                        Token::String(string) => string.to_string(),
                        _ => return CustomError::error_invalid_syntax("Expected value after '('"),
                    };
                    row.insert(columns[column_index].to_string(), value); // Agrego el valor de la columna[i] al hashmap
                    column_index += 1;
                }
                Token::Symbol(',') => {
                    // Si es coma, verifico que su siguiente sea un valor
                    if let Some(Token::Integer(_)) | Some(Token::String(_)) = iter.peek() {
                    } else {
                        return CustomError::error_invalid_syntax("Expected value after ','");
                    }
                }
                Token::Symbol(')') => {
                    // Si se cierra parentesis, pusheo el hashmap al vector de valores y termino
                    values.push(row);
                    break;
                }
                _ => {
                    return CustomError::error_invalid_syntax("Expected value or ')' after '('");
                }
            }
        }
    }
    Ok(())
}

fn check_ending_with_semicolon(iter: &mut Peekable<Iter<Token>>) -> Result<(), CustomError> {
    if let Some(Token::Symbol(';')) = iter.next() {
        if iter.peek().is_some() {
            return CustomError::error_invalid_syntax("Tokens found after ';'");
        }
    } else {
        return CustomError::error_invalid_syntax("Expected ';' at the end of the command");
    }
    Ok(())
}

/// Parsea un comando UPDATE que llega en forma de vector de tokens.
/// Modifica los parametros table_name, set_values y condition.
///
/// El formato del comando UPDATE esperado es:
/// UPDATE <table_name> SET <column1> = <value1>, <column2> = <value2>, ... WHERE <condition>;
/// donde WHERE es opcional.
pub fn parse_update(
    tokens: &[Token],
    table_name: &mut String,
    set_values: &mut HashMap<String, String>,
    condition: &mut Expression,
) -> Result<(), CustomError> {
    let mut iter = tokens.iter().peekable();
    iter.next(); // salteo el UPDATE
    if let Some(Token::Identifier(name)) | Some(Token::String(name)) = iter.next() {
        // Verifico que haya nombre de tabla
        *table_name = name.to_string();
    } else {
        return CustomError::error_invalid_syntax("Expected table name after UPDATE");
    }
    parse_update_set_values(set_values, &mut iter)?;
    parse_condition(condition, &mut iter)?;
    check_ending_with_semicolon(&mut iter)?;
    Ok(())
}

fn parse_update_set_values(
    set_values: &mut HashMap<String, String>,
    iter: &mut Peekable<Iter<Token>>,
) -> Result<(), CustomError> {
    if !matches!(iter.next(), Some(Token::Keyword(keyword)) if keyword.as_str() == "SET") {
        // Verifico que haya SET
        return CustomError::error_invalid_syntax("Expected SET after table name");
    }
    parse_update_set_value(set_values, iter)?; // Parseo el primer valor
    while let Some(Token::Symbol(',')) = iter.peek() {
        // Si lo sigue una coma, parseo otro valor
        iter.next();
        parse_update_set_value(set_values, iter)?;
    }
    Ok(())
}

fn parse_update_set_value(
    set_values: &mut HashMap<String, String>,
    iter: &mut Peekable<Iter<Token>>,
) -> Result<(), CustomError> {
    let column: String;
    let value: String;
    if let Some(Token::Identifier(name)) = iter.next() {
        // Verifico que haya nombre de columna
        column = name.to_string();
    } else {
        return CustomError::error_invalid_syntax("Expected column name to set value after SET");
    }
    if matches!(iter.next(), Some(Token::ComparisonOperator(keyword)) if keyword.as_str() == "=") {
        // Verifico que haya '='
        if let Some(Token::Integer(string)) | Some(Token::String(string)) = iter.next() {
            // Verifico que haya valor
            value = string.to_string();
        } else {
            return CustomError::error_invalid_syntax("Expected value after '='");
        }
    } else {
        return CustomError::error_invalid_syntax("Expected '=' after column name");
    }
    set_values.insert(column, value); // Agrego el valor de la columna al hashmap set_values
    Ok(())
}

fn parse_condition(
    condition: &mut Expression,
    iter: &mut Peekable<Iter<Token>>,
) -> Result<(), CustomError> {
    if let Some(Token::Keyword(keyword)) = iter.peek() {
        // Verifico que haya WHERE
        if keyword.as_str() == "WHERE" {
            iter.next();
            *condition = parse_expression(iter)?; // Parseo la condicion
        }
    }
    Ok(())
}

/// Parsea un comando DELETE que llega en forma de vector de tokens.
/// Modifica los parametros table_name y condition.
///
/// El formato del comando DELETE esperado es:
/// DELETE <table_name> WHERE <condition>;
/// donde WHERE es opcional.
pub fn parse_delete(
    tokens: &[Token],
    table_name: &mut String,
    condition: &mut Expression,
) -> Result<(), CustomError> {
    let mut iter = tokens.iter().peekable();
    iter.next(); // salteo el DELETE
    if let Some(Token::Identifier(name)) | Some(Token::String(name)) = iter.next() {
        // Verifico que haya nombre de tabla
        *table_name = name.to_string();
    } else {
        return CustomError::error_invalid_syntax("Expected table name after DELETE");
    }
    parse_condition(condition, &mut iter)?;
    check_ending_with_semicolon(&mut iter)?;
    Ok(())
}

/// Parsea un comando SELECT que llega en forma de vector de tokens.
/// Modifica los parametros columns, table_name, condition y order_by.
///
/// El formato del comando SELECT esperado es:
/// SELECT <column1>, <column2>, ... FROM <table_name> WHERE <condition> ORDER BY <column> <order>, <column> <order>, ... ;
/// donde WHERE y ORDER BY son opcionales.
pub fn parse_select(
    tokens: &[Token],
    columns: &mut Vec<String>,
    table_name: &mut String,
    condition: &mut Expression,
    order_by: &mut Vec<(String, String)>,
) -> Result<(), CustomError> {
    let mut iter = tokens.iter().peekable();
    iter.next(); // salteo el SELECT
    parse_select_columns(columns, &mut iter)?;
    parse_select_from(table_name, &mut iter)?;
    parse_condition(condition, &mut iter)?;
    parse_order_by(order_by, &mut iter)?;
    check_ending_with_semicolon(&mut iter)?;
    Ok(())
}

fn parse_select_columns(
    columns: &mut Vec<String>,
    iter: &mut Peekable<Iter<Token>>,
) -> Result<(), CustomError> {
    if matches!(iter.peek(), Some(Token::Symbol('*'))) {
        // Si hay '*', lo dejo vacío, que indica que se seleccionan todas las columnas
        iter.next();
        return Ok(());
    }
    while let Some(token) = iter.peek() {
        // Este ciclo termina al encontrar un Keyword
        match token {
            Token::Identifier(name) => {
                // Si es nombre de columna, lo agrego
                columns.push(name.to_string());
                iter.next();
            }
            Token::Keyword(_) => {
                // Si es Keyword, termino
                break;
            }
            Token::Symbol(',') => {
                // Si es coma, verifico que su siguiente sea nombre de columna
                iter.next();
                if let Some(Token::Identifier(_)) = iter.peek() {
                } else {
                    return CustomError::error_invalid_syntax("Expected column name after ','");
                }
            }
            _ => {
                return CustomError::error_invalid_syntax(
                    "Expected column name or FROM <tablename> after column names",
                );
            }
        }
    }
    Ok(())
}

fn parse_select_from(
    table_name: &mut String,
    iter: &mut Peekable<Iter<Token>>,
) -> Result<(), CustomError> {
    if !matches!(iter.next(), Some(Token::Keyword(keyword)) if keyword.as_str() == "FROM") {
        // Verifico que haya FROM
        return CustomError::error_invalid_syntax("Expected FROM after column names");
    }
    if let Some(Token::Identifier(name)) | Some(Token::String(name)) = iter.next() {
        // Verifico que haya nombre de tabla
        *table_name = name.to_string();
    } else {
        return CustomError::error_invalid_syntax("Expected table name after FROM");
    }
    Ok(())
}

fn parse_order_by(
    order_by: &mut Vec<(String, String)>,
    iter: &mut Peekable<Iter<Token>>,
) -> Result<(), CustomError> {
    if matches!(iter.peek(), Some(Token::Keyword(keyword)) if keyword.as_str() == "ORDER") {
        // Verifico que haya ORDER
        iter.next();
        if !matches!(iter.next(), Some(Token::Keyword(keyword)) if keyword.as_str() == "BY") {
            // Verifico que haya BY
            return CustomError::error_invalid_syntax("Expected BY after ORDER");
        }
    } else {
        // Si no hay ORDER BY, no hay ningun orden que seguir
        return Ok(());
    }
    parse_order_by_column(order_by, iter)?; // Parseo la primera columna por la cual ordenar
    while let Some(Token::Symbol(',')) = iter.peek() {
        // Si lo sigue una coma, parseo otra columna
        iter.next();
        parse_order_by_column(order_by, iter)?;
    }
    Ok(())
}

fn parse_order_by_column(
    order_by: &mut Vec<(String, String)>,
    iter: &mut Peekable<Iter<Token>>,
) -> Result<(), CustomError> {
    let order_by_tuple: (String, String);
    let order_by_column: String;
    if let Some(Token::Identifier(name)) = iter.next() {
        // Verifico que haya nombre de columna
        order_by_column = name.to_string();
    } else {
        return CustomError::error_invalid_syntax("Expected column name after ORDER BY or ','");
    }
    if let Some(Token::Keyword(keyword)) = iter.peek() {
        // Verifico que haya DESC o nada
        if keyword.as_str() == "DESC" {
            iter.next();
            order_by_tuple = (order_by_column, "DESC".to_string());
        } else {
            return CustomError::error_invalid_syntax("Expected DESC or nothing after column name");
        }
    } else {
        order_by_tuple = (order_by_column, "ASC".to_string());
    }
    order_by.push(order_by_tuple); // Agrego la columna y su orden al vector order_by
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::expression::Operand;
    use super::*;

    #[test]
    fn test_parse_insert() {
        // INSERT INTO table (column1, column2) VALUES ('value1', 'value2');
        let tokens = vec![
            Token::Keyword("INSERT".to_string()),
            Token::Keyword("INTO".to_string()),
            Token::Identifier("table".to_string()),
            Token::Symbol('('),
            Token::Identifier("column1".to_string()),
            Token::Symbol(','),
            Token::Identifier("column2".to_string()),
            Token::Symbol(')'),
            Token::Keyword("VALUES".to_string()),
            Token::Symbol('('),
            Token::String("value1".to_string()),
            Token::Symbol(','),
            Token::String("value2".to_string()),
            Token::Symbol(')'),
            Token::Symbol(';'),
        ];
        let mut table_name = String::new();
        let mut columns = Vec::new();
        let mut values = Vec::new();

        let result = parse_insert(&tokens, &mut table_name, &mut columns, &mut values);

        assert!(result.is_ok());
        assert_eq!(table_name, "table");
        assert_eq!(columns, vec!["column1".to_string(), "column2".to_string(),]);
        assert_eq!(
            values,
            vec![{
                let mut row = HashMap::new();
                row.insert("column1".to_string(), "value1".to_string());
                row.insert("column2".to_string(), "value2".to_string());
                row
            }]
        );
    }

    #[test]
    fn test_parse_insert_invalid_syntax() {
        // INSERT INTO table (column1, column2)) VALUES ('value1', 'value2');
        let tokens = vec![
            Token::Keyword("INSERT".to_string()),
            Token::Keyword("INTO".to_string()),
            Token::Identifier("table".to_string()),
            Token::Symbol('('),
            Token::Identifier("column1".to_string()),
            Token::Symbol(','),
            Token::Identifier("column2".to_string()),
            Token::Symbol(')'),
            Token::Symbol(')'), // Agregue un parentesis de mas
            Token::Keyword("VALUES".to_string()),
            Token::Symbol('('),
            Token::String("value1".to_string()),
            Token::Symbol(','),
            Token::String("value2".to_string()),
            Token::Symbol(')'),
        ];
        let mut table_name = String::new();
        let mut columns = Vec::new();
        let mut values = Vec::new();

        let result = parse_insert(&tokens, &mut table_name, &mut columns, &mut values);

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            CustomError::InvalidSyntax {
                message: "Expected VALUES after column names".to_string()
            }
        );
    }

    #[test]
    fn test_parse_update() {
        // UPDATE table SET column1 = 'value1', column2 = 'value2' WHERE column3 = 'value3';
        let tokens = vec![
            Token::Keyword("UPDATE".to_string()),
            Token::Identifier("table".to_string()),
            Token::Keyword("SET".to_string()),
            Token::Identifier("column1".to_string()),
            Token::ComparisonOperator("=".to_string()),
            Token::String("value1".to_string()),
            Token::Symbol(','),
            Token::Identifier("column2".to_string()),
            Token::ComparisonOperator("=".to_string()),
            Token::String("value2".to_string()),
            Token::Keyword("WHERE".to_string()),
            Token::Identifier("column3".to_string()),
            Token::ComparisonOperator("=".to_string()),
            Token::String("value3".to_string()),
            Token::Symbol(';'),
        ];
        let mut table_name = String::new();
        let mut set_values = HashMap::new();
        let mut condition = Expression::Comparison {
            left: Operand::Column("column1".to_string()),
            operator: "=".to_string(),
            right: Operand::String("value1".to_string()),
        };

        let result = parse_update(&tokens, &mut table_name, &mut set_values, &mut condition);

        println!("{:?}", result);
        assert!(result.is_ok());
        assert_eq!(table_name, "table");
        assert_eq!(set_values, {
            let mut set_values = HashMap::new();
            set_values.insert("column1".to_string(), "value1".to_string());
            set_values.insert("column2".to_string(), "value2".to_string());
            set_values
        });
        assert_eq!(
            condition,
            Expression::Comparison {
                left: Operand::Column("column3".to_string()),
                operator: "=".to_string(),
                right: Operand::String("value3".to_string()),
            }
        );
    }

    #[test]
    fn test_parse_update_invalid_syntax() {
        // UPDATE table SET column1 = 'value1', = 'value2' WHERE column3 = 'value3';
        let tokens = vec![
            Token::Keyword("UPDATE".to_string()),
            Token::Identifier("table".to_string()),
            Token::Keyword("SET".to_string()),
            Token::Identifier("column1".to_string()),
            Token::ComparisonOperator("=".to_string()),
            Token::String("value1".to_string()),
            Token::Symbol(','), // Le falta la columna a setear
            Token::ComparisonOperator("=".to_string()),
            Token::String("value2".to_string()),
            Token::Keyword("WHERE".to_string()),
            Token::Identifier("column3".to_string()),
            Token::ComparisonOperator("=".to_string()),
            Token::String("value3".to_string()),
            Token::Symbol(';'),
        ];
        let mut table_name = String::new();
        let mut set_values = HashMap::new();
        let mut condition = Expression::Comparison {
            left: Operand::Column("column1".to_string()),
            operator: "=".to_string(),
            right: Operand::String("value1".to_string()),
        };

        let result = parse_update(&tokens, &mut table_name, &mut set_values, &mut condition);
        assert!(result.is_err());

        assert_eq!(
            result.err().unwrap(),
            CustomError::InvalidSyntax {
                message: "Expected column name to set value after SET".to_string()
            }
        );
    }

    #[test]
    fn test_parse_delete() {
        // DELETE table WHERE column1 = 'value1';
        let tokens = vec![
            Token::Keyword("DELETE".to_string()),
            Token::Identifier("table".to_string()),
            Token::Keyword("WHERE".to_string()),
            Token::Identifier("column1".to_string()),
            Token::ComparisonOperator("=".to_string()),
            Token::String("value1".to_string()),
            Token::Symbol(';'),
        ];
        let mut table_name = String::new();
        let mut condition = Expression::Comparison {
            left: Operand::Column("column1".to_string()),
            operator: "=".to_string(),
            right: Operand::String("value1".to_string()),
        };

        let result = parse_delete(&tokens, &mut table_name, &mut condition);

        assert!(result.is_ok());
        assert_eq!(table_name, "table");
        assert_eq!(
            condition,
            Expression::Comparison {
                left: Operand::Column("column1".to_string()),
                operator: "=".to_string(),
                right: Operand::String("value1".to_string()),
            }
        );
    }

    #[test]
    fn test_parse_delete_invalid_syntax() {
        // DELETE table column1 = 'value1'
        let tokens = vec![
            Token::Keyword("DELETE".to_string()),
            Token::Identifier("table".to_string()), // Le falta WHERE
            Token::Identifier("column1".to_string()),
            Token::ComparisonOperator("=".to_string()),
            Token::String("value1".to_string()),
        ];
        let mut table_name = String::new();
        let mut condition = Expression::Comparison {
            left: Operand::Column("column1".to_string()),
            operator: "=".to_string(),
            right: Operand::String("value1".to_string()),
        };

        let result = parse_delete(&tokens, &mut table_name, &mut condition);

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            CustomError::InvalidSyntax {
                message: "Expected ';' at the end of the command".to_string()
            }
        );
    }

    #[test]
    fn test_parse_select() {
        // SELECT column1, column2 FROM table WHERE column3 = 'value3' ORDER BY column4 DESC, column5 ASC;
        let tokens = vec![
            Token::Keyword("SELECT".to_string()),
            Token::Identifier("column1".to_string()),
            Token::Symbol(','),
            Token::Identifier("column2".to_string()),
            Token::Keyword("FROM".to_string()),
            Token::Identifier("table".to_string()),
            Token::Keyword("WHERE".to_string()),
            Token::Identifier("column3".to_string()),
            Token::ComparisonOperator("=".to_string()),
            Token::String("value3".to_string()),
            Token::Keyword("ORDER".to_string()),
            Token::Keyword("BY".to_string()),
            Token::Identifier("column4".to_string()),
            Token::Keyword("DESC".to_string()),
            Token::Symbol(','),
            Token::Identifier("column5".to_string()),
            Token::Symbol(';'),
        ];
        let mut columns = Vec::new();
        let mut table_name = String::new();
        let mut condition = Expression::Comparison {
            left: Operand::Column("column3".to_string()),
            operator: "=".to_string(),
            right: Operand::String("value3".to_string()),
        };
        let mut order_by = Vec::new();

        let result = parse_select(
            &tokens,
            &mut columns,
            &mut table_name,
            &mut condition,
            &mut order_by,
        );

        assert!(result.is_ok());
        assert_eq!(columns, vec!["column1".to_string(), "column2".to_string(),]);
        assert_eq!(table_name, "table");
        assert_eq!(
            condition,
            Expression::Comparison {
                left: Operand::Column("column3".to_string()),
                operator: "=".to_string(),
                right: Operand::String("value3".to_string()),
            }
        );
        assert_eq!(
            order_by,
            vec![
                ("column4".to_string(), "DESC".to_string()),
                ("column5".to_string(), "ASC".to_string()),
            ]
        );
    }

    #[test]
    fn test_parse_select_invalid_syntax() {
        // SELECT column1, column2 FROM table WHERE column3 = 'value3' ORDER BY column4 DESC, column5 ASC;
        let tokens = vec![
            Token::Keyword("SELECT".to_string()),
            Token::Identifier("column1".to_string()),
            Token::Symbol(','),
            Token::Identifier("column2".to_string()),
            Token::Keyword("FROM".to_string()),
            Token::Identifier("table".to_string()),
            Token::Keyword("WHERE".to_string()),
            Token::Identifier("column3".to_string()),
            Token::ComparisonOperator("=".to_string()),
            Token::String("value3".to_string()),
            Token::Keyword("ORDER".to_string()),
            Token::Keyword("BY".to_string()),
            Token::Identifier("column4".to_string()),
            Token::Keyword("DESC".to_string()),
            Token::Symbol(','),
            Token::Identifier("column5".to_string()),
            Token::Keyword("ASC".to_string()), // ASC no se pone
            Token::Symbol(';'),
        ];
        let mut columns = Vec::new();
        let mut table_name = String::new();
        let mut condition = Expression::Comparison {
            left: Operand::Column("column3".to_string()),
            operator: "=".to_string(),
            right: Operand::String("value3".to_string()),
        };
        let mut order_by = Vec::new();

        let result = parse_select(
            &tokens,
            &mut columns,
            &mut table_name,
            &mut condition,
            &mut order_by,
        );

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            CustomError::InvalidSyntax {
                message: "Expected DESC or nothing after column name".to_string()
            }
        );
    }
}
