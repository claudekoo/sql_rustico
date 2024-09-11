use super::custom_error::CustomError;
use super::expression::Expression;
use super::expression_parser::parse_expression;
use super::tokenizer::Token;
use std::collections::HashMap;

/// Parsea un comando INSERT que llega en forma de vector de tokens.
/// Modifica los parametros table_name, columns y values.
///
/// El formato del comando INSERT esperado es:
/// INSERT INTO <table_name> (<column1>, <column2>, ...) VALUES (<value1>, <value2>, ...);
pub fn parse_insert(
    tokens: &Vec<Token>,
    table_name: &mut String,
    columns: &mut Vec<String>,
    values: &mut Vec<HashMap<String, String>>,
) -> Result<(), CustomError> {
    let mut iter = tokens.iter().peekable();
    iter.next(); // salteo el INSERT
    if let Some(Token::Keyword(keyword)) = iter.next() {
        if keyword.as_str() == "INTO" {
            parse_insert_from_into(table_name, columns, values, &mut iter)?;
        } else {
            return CustomError::error_invalid_syntax("Expected INTO after INSERT");
        }
    } else {
        return CustomError::error_invalid_syntax("Expected INTO after INSERT");
    }
    Ok(())
}

fn parse_insert_from_into(
    table_name: &mut String,
    columns: &mut Vec<String>,
    values: &mut Vec<HashMap<String, String>>,
    iter: &mut std::iter::Peekable<std::slice::Iter<Token>>,
) -> Result<(), CustomError> {
    let name_option = iter.next();
    if let Some(Token::Identifier(name)) = name_option {
        *table_name = name.to_string();
    } else if let Some(Token::String(name)) = name_option {
        *table_name = name.to_string();
    } else {
        return CustomError::error_invalid_syntax("Expected table name after INTO");
    }
    parse_insert_into_columns(columns, iter)?;
    parse_values(values, iter, columns)?;
    check_ending_with_semicolon(iter)?;
    Ok(())
}

fn parse_insert_into_columns(
    columns: &mut Vec<String>,
    iter: &mut std::iter::Peekable<std::slice::Iter<Token>>,
) -> Result<(), CustomError> {
    if let Some(Token::Symbol('(')) = iter.peek() {
        iter.next();
        while let Some(token) = iter.next() {
            match token {
                Token::Identifier(name) => {
                    columns.push(name.to_string());
                }
                Token::Symbol(')') => {
                    break;
                }
                Token::Symbol(',') => {
                    // deberia chequear que su siguiente sea un nombre de columna
                }
                _ => {
                    return CustomError::error_invalid_syntax(
                        "Expected column name or ')' after '('",
                    );
                }
            }
        }
    }
    Ok(())
}

fn parse_values(
    values: &mut Vec<HashMap<String, String>>,
    iter: &mut std::iter::Peekable<std::slice::Iter<Token>>,
    columns: &Vec<String>,
) -> Result<(), CustomError> {
    if let Some(Token::Keyword(keyword)) = iter.peek() {
        if keyword.as_str() == "VALUES" {
            iter.next();
        } else {
            return CustomError::error_invalid_syntax("Expected VALUES after column names");
        }
    } else {
        return CustomError::error_invalid_syntax("Expected VALUES after column names");
    }
    parse_value(values, iter, columns)?;
    while let Some(Token::Symbol(',')) = iter.peek() {
        iter.next();
        parse_value(values, iter, columns)?;
    }
    Ok(())
}

fn parse_value(
    values: &mut Vec<HashMap<String, String>>,
    iter: &mut std::iter::Peekable<std::slice::Iter<Token>>,
    columns: &Vec<String>,
) -> Result<(), CustomError> {
    let mut row: HashMap<String, String> = HashMap::new();
    if let Some(Token::Symbol('(')) = iter.peek() {
        iter.next();
        let mut column_index = 0;
        while let Some(token) = iter.next() {
            match token {
                Token::Integer(value) | Token::String(value) => {
                    if column_index >= columns.len() {
                        return CustomError::error_invalid_syntax("Too many values for columns");
                    }
                    row.insert(columns[column_index].to_string(), value.to_string());
                    column_index += 1;
                }
                Token::Symbol(',') => {
                    // deberia chequear que su siguiente sea un valor
                }
                Token::Symbol(')') => {
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

/// Parsea un comando UPDATE que llega en forma de vector de tokens.
/// Modifica los parametros table_name, set_values y condition.
///
/// El formato del comando UPDATE esperado es:
/// UPDATE <table_name> SET <column1> = <value1>, <column2> = <value2>, ... WHERE <condition>;
/// donde WHERE es opcional.
pub fn parse_update(
    tokens: &Vec<Token>,
    table_name: &mut String,
    set_values: &mut HashMap<String, String>,
    condition: &mut Expression,
) -> Result<(), CustomError> {
    let mut iter = tokens.iter().peekable();
    iter.next(); // salteo el UPDATE
    let name_option = iter.next();
    if let Some(Token::Identifier(name)) = name_option {
        *table_name = name.to_string();
    } else if let Some(Token::String(name)) = name_option {
        *table_name = name.to_string();
    } else {
        return CustomError::error_invalid_syntax("Expected table name after UPDATE");
    }
    if let Some(Token::Keyword(keyword)) = iter.next() {
        if keyword.as_str() != "SET" {
            return CustomError::error_invalid_syntax("Expected SET after table name");
        }
    } else {
        return CustomError::error_invalid_syntax("Expected SET after table name");
    }
    parse_update_set_values(set_values, condition, &mut iter)?;
    Ok(())
}

fn parse_update_set_values(
    set_values: &mut HashMap<String, String>,
    condition: &mut Expression,
    iter: &mut std::iter::Peekable<std::slice::Iter<Token>>,
) -> Result<(), CustomError> {
    parse_set_value(set_values, iter)?;
    while let Some(Token::Symbol(',')) = iter.peek() {
        iter.next();
        parse_set_value(set_values, iter)?;
    }
    if let Some(Token::Keyword(keyword)) = iter.peek() {
        if keyword.as_str() == "WHERE" {
            iter.next();
            parse_condition(condition, iter)?;
        }
    } else if let Some(Token::Symbol(';')) = iter.peek() {
        iter.next();
        if let Some(_) = iter.peek() {
            return CustomError::error_invalid_syntax("Tokens found after ';'");
        }
        return Ok(());
    } else {
        return CustomError::error_invalid_syntax("Expected WHERE or ';' after set values");
    }
    Ok(())
}

fn parse_set_value(
    set_values: &mut HashMap<String, String>,
    iter: &mut std::iter::Peekable<std::slice::Iter<Token>>,
) -> Result<(), CustomError> {
    let column: String;
    let value: String;
    if let Some(Token::Identifier(name)) = iter.next() {
        column = name.to_string();
    } else {
        return CustomError::error_invalid_syntax("Expected column name to set value after SET");
    }
    if let Some(Token::ComparisonOperator(keyword)) = iter.next() {
        if keyword.as_str() == "=" {
            if let Some(Token::Integer(int)) = iter.peek() {
                value = int.to_string();
            } else if let Some(Token::String(string)) = iter.peek() {
                value = string.to_string();
            } else {
                return CustomError::error_invalid_syntax("Expected value after '='");
            }
        } else {
            return CustomError::error_invalid_syntax("Expected '=' after column name");
        }
    } else {
        return CustomError::error_invalid_syntax("Expected '=' after column name");
    }
    iter.next();
    set_values.insert(column, value);
    Ok(())
}

fn parse_condition(
    condition: &mut Expression,
    iter: &mut std::iter::Peekable<std::slice::Iter<Token>>,
) -> Result<(), CustomError> {
    *condition = parse_expression(iter)?;
    if let Some(token) = iter.peek() {
        if let Token::Symbol(';') = token {
            iter.next();
            if let Some(_) = iter.peek() {
                return CustomError::error_invalid_syntax("Tokens found after ';'");
            }
            return Ok(());
        } else {
            return CustomError::error_invalid_syntax("Expected ';' after WHERE condition");
        }
    } else {
        return CustomError::error_invalid_syntax("Expected ';' after WHERE condition");
    }
}

/// Parsea un comando DELETE que llega en forma de vector de tokens.
/// Modifica los parametros table_name y condition.
///
/// El formato del comando DELETE esperado es:
/// DELETE <table_name> WHERE <condition>;
/// donde WHERE es opcional.
pub fn parse_delete(
    tokens: &Vec<Token>,
    table_name: &mut String,
    condition: &mut Expression,
) -> Result<(), CustomError> {
    let mut iter = tokens.iter().peekable();
    iter.next(); // salteo el DELETE
    let name_option = iter.next();
    if let Some(Token::Identifier(name)) = name_option {
        *table_name = name.to_string();
    } else if let Some(Token::String(name)) = name_option {
        *table_name = name.to_string();
    } else {
        return CustomError::error_invalid_syntax("Expected table name after DELETE");
    }
    if let Some(Token::Keyword(keyword)) = iter.peek() {
        if keyword.as_str() == "WHERE" {
            iter.next();
            parse_condition(condition, &mut iter)?;
        }
    } else if let Some(Token::Symbol(';')) = iter.peek() {
        iter.next();
        if let Some(_) = iter.peek() {
            return CustomError::error_invalid_syntax("Tokens found after ';'");
        }
    } else {
        return CustomError::error_invalid_syntax("Expected WHERE or ';' after table name");
    }
    Ok(())
}

/// Parsea un comando SELECT que llega en forma de vector de tokens.
/// Modifica los parametros columns, table_name, condition y order_by.
///
/// El formato del comando SELECT esperado es:
/// SELECT <column1>, <column2>, ... FROM <table_name> WHERE <condition> ORDER BY <column> <order>, <column> <order>, ... ;
/// donde WHERE y ORDER BY son opcionales.
pub fn parse_select(
    tokens: &Vec<Token>,
    columns: &mut Vec<String>,
    table_name: &mut String,
    condition: &mut Expression,
    order_by: &mut Vec<(String, String)>,
) -> Result<(), CustomError> {
    let mut iter = tokens.iter().peekable();
    iter.next(); // salteo el SELECT
    parse_select_columns(columns, &mut iter)?;
    parse_select_from(table_name, &mut iter)?;
    if let Some(Token::Keyword(keyword)) = iter.peek() {
        if keyword.as_str() == "WHERE" {
            iter.next();
            *condition = parse_expression(&mut iter)?;
        }
    }
    if let Some(Token::Keyword(keyword)) = iter.peek() {
        if keyword.as_str() == "ORDER" {
            iter.next();
            if let Some(Token::Keyword(keyword)) = iter.next() {
                if keyword.as_str() != "BY" {
                    return CustomError::error_invalid_syntax("Expected BY after ORDER");
                }
            } else {
                return CustomError::error_invalid_syntax("Expected BY after ORDER");
            }
            parse_order(order_by, &mut iter)?;
        }
    }
    check_ending_with_semicolon(&mut iter)?;
    Ok(())
}

fn check_ending_with_semicolon(
    iter: &mut std::iter::Peekable<std::slice::Iter<Token>>,
) -> Result<(), CustomError> {
    if let Some(Token::Symbol(';')) = iter.peek() {
        iter.next();
        if let Some(_) = iter.peek() {
            return CustomError::error_invalid_syntax("Tokens found after ';'");
        }
    } else {
        return CustomError::error_invalid_syntax("Expected ';' at the end of the command");
    }
    Ok(())
}

fn parse_select_columns(
    columns: &mut Vec<String>,
    iter: &mut std::iter::Peekable<std::slice::Iter<Token>>,
) -> Result<(), CustomError> {
    if let Some(Token::Symbol('*')) = iter.peek() {
        // si columns esta vacio, se seleccionan todas las columnas
        iter.next();
        if let Some(Token::Keyword(keyword)) = iter.peek() {
            if keyword.as_str() == "FROM" {
                iter.next();
                return Ok(());
            }
            return CustomError::error_invalid_syntax("Expected FROM <tablename> after '*'");
        }
    }
    while let Some(token) = iter.next() {
        match token {
            Token::Identifier(name) => {
                columns.push(name.to_string());
            }
            Token::Keyword(keyword) => {
                if keyword.as_str() == "FROM" {
                    iter.next();
                    break;
                } else {
                    return CustomError::error_invalid_syntax(
                        "Expected column name or FROM <tablename> after column names",
                    );
                }
            }
            Token::Symbol(',') => {
                // deberia chequear que su siguiente sea un nombre de columna
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
    iter: &mut std::iter::Peekable<std::slice::Iter<Token>>,
) -> Result<(), CustomError> {
    let name_option = iter.next();
    if let Some(Token::Identifier(name)) = name_option {
        *table_name = name.to_string();
    } else if let Some(Token::String(name)) = name_option {
        *table_name = name.to_string();
    } else {
        return CustomError::error_invalid_syntax("Expected table name after FROM");
    }
    Ok(())
}

fn parse_order(
    order_by: &mut Vec<(String, String)>,
    iter: &mut std::iter::Peekable<std::slice::Iter<Token>>,
) -> Result<(), CustomError> {
    parse_order_by_column(order_by, iter)?;
    while let Some(Token::Symbol(',')) = iter.peek() {
        iter.next();
        parse_order_by_column(order_by, iter)?;
    }
    Ok(())
}

fn parse_order_by_column(
    order_by: &mut Vec<(String, String)>,
    iter: &mut std::iter::Peekable<std::slice::Iter<Token>>,
) -> Result<(), CustomError> {
    let order_by_tuple: (String, String);
    let order_by_column: String;
    if let Some(Token::Identifier(name)) = iter.next() {
        order_by_column = name.to_string();
    } else {
        return CustomError::error_invalid_syntax("Expected column name after ORDER BY or ','");
    }
    if let Some(Token::Keyword(keyword)) = iter.peek() {
        if keyword.as_str() == "DESC" {
            iter.next();
            order_by_tuple = (order_by_column, "DESC".to_string());
        } else {
            return CustomError::error_invalid_syntax("Expected DESC or nothing after column name");
        }
    } else {
        order_by_tuple = (order_by_column, "ASC".to_string());
    }
    order_by.push(order_by_tuple);
    Ok(())
}
