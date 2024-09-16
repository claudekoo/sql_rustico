use super::custom_error::CustomError;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, PartialEq)]
/// Los Tokens son la unidad mínima de un comando SQL existen para facilitar su parseo.
pub enum Token {
    /// Los Keywords son palabras clave de un comando SQL, esta implementación incluye:
    /// INSERT, UPDATE, DELETE, SELECT, FROM, WHERE, SET, INTO, VALUES, ORDER, BY, DESC
    Keyword(String),
    /// Los LogicalOperators son operadores lógicos, en esta implementación incluye:
    /// AND, OR, NOT
    LogicalOperator(String),
    /// Los ComparisonOperators son operadores de comparación, en esta implementación incluye:
    /// =, >, <, >=, <=
    ComparisonOperator(String),
    /// Los Identifiers son nombres de tablas o columnas, pueden ser alfanuméricos.
    Identifier(String),
    /// Los Strings son cadenas de texto llegadas entre comillas simples.
    String(String),
    /// Los Integers son números enteros.
    Integer(String),
    /// Los Symbols son caracteres especiales, en esta implementación incluye:
    /// , ( ) ; *
    Symbol(char),
}

fn tokenize_integer_or_identifier_starting_with_integer(chars: &mut Peekable<Chars>) -> Token {
    let mut token_value = String::new();
    while let Some(&ch) = chars.peek() {
        if ch.is_ascii_digit() {
            token_value.push(ch);
            chars.next();
        } else if ch.is_alphabetic() {
            while let Some(&ch) = chars.peek() {
                if ch.is_alphanumeric() {
                    token_value.push(ch);
                    chars.next();
                } else {
                    break;
                }
            }
            return Token::Identifier(token_value);
        } else {
            break;
        }
    }
    Token::Integer(token_value)
}

fn tokenize_word(chars: &mut Peekable<Chars>) -> Token {
    let mut word = String::new();
    while let Some(&ch) = chars.peek() {
        if ch.is_alphanumeric() {
            word.push(ch);
            chars.next();
        } else {
            break;
        }
    }
    let word_upper = word.to_uppercase();
    if [
        "INSERT", "UPDATE", "DELETE", "SELECT", "FROM", "WHERE", "SET", "INTO", "VALUES", "ORDER",
        "BY", "DESC",
    ]
    .contains(&word_upper.as_str())
    {
        Token::Keyword(word_upper)
    } else if ["AND", "OR", "NOT"].contains(&word_upper.as_str()) {
        Token::LogicalOperator(word_upper)
    } else {
        Token::Identifier(word)
    }
}

fn tokenize_string(chars: &mut Peekable<Chars>) -> Token {
    chars.next();
    let mut string = String::new();
    while let Some(&ch) = chars.peek() {
        if ch != '\'' {
            string.push(ch);
            chars.next();
        } else {
            chars.next();
            break;
        }
    }
    Token::String(string)
}

fn tokenize_comparison_operator(chars: &mut Peekable<Chars>) -> Token {
    let mut comparison = String::new();
    if let Some(&ch) = chars.peek() {
        if '=' == ch {
            // no existen ==, =>, =<
            comparison.push(ch);
            chars.next();
        } else if ['>', '<'].contains(&ch) {
            // pueden ser >, <, >=, <=
            comparison.push(ch);
            chars.next();
            if let Some(&ch) = chars.peek() {
                if '=' == ch {
                    comparison.push(ch);
                    chars.next();
                }
            }
        }
    }
    Token::ComparisonOperator(comparison)
}

/// Tokeniza un string de entrada y retorna un vector de Tokens.
pub fn tokenize(input: &str) -> Result<Vec<Token>, CustomError> {
    let mut tokens = vec![];
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() {
            // ignorar espacios y newlines
            chars.next(); // esto ocurre solo cuando no esta entre comillas
        } else if ch.is_ascii_digit() {
            tokens.push(tokenize_integer_or_identifier_starting_with_integer(
                &mut chars,
            ));
        } else if ch.is_alphabetic() {
            tokens.push(tokenize_word(&mut chars)); // palabras clave o nombres
        } else if ch == '\'' {
            tokens.push(tokenize_string(&mut chars)); // strings
        } else if ['=', '>', '<'].contains(&ch) {
            tokens.push(tokenize_comparison_operator(&mut chars)); // operadores de comparacion
        } else if [',', '(', ')', ';', '*'].contains(&ch) {
            tokens.push(Token::Symbol(ch)); // no lo modulo porque siempre es un solo caracter
            chars.next();
        } else {
            CustomError::error_invalid_syntax(&format!("Invalid syntax near: {}", ch))?;
        }
    }
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let input = "SELECT * FROM table1 WHERE column1 = 'value1';";
        let expected_output = vec![
            Token::Keyword("SELECT".to_string()),
            Token::Symbol('*'),
            Token::Keyword("FROM".to_string()),
            Token::Identifier("table1".to_string()),
            Token::Keyword("WHERE".to_string()),
            Token::Identifier("column1".to_string()),
            Token::ComparisonOperator("=".to_string()),
            Token::String("value1".to_string()),
            Token::Symbol(';'),
        ];
        assert_eq!(tokenize(input).unwrap(), expected_output);
    }

    #[test]
    fn test_tokenize_with_identifies_starting_with_number() {
        let input = "SELECT * FROM table1 WHERE column1 = 'value1' AND 1column = 'value2';";
        let expected_output = vec![
            Token::Keyword("SELECT".to_string()),
            Token::Symbol('*'),
            Token::Keyword("FROM".to_string()),
            Token::Identifier("table1".to_string()),
            Token::Keyword("WHERE".to_string()),
            Token::Identifier("column1".to_string()),
            Token::ComparisonOperator("=".to_string()),
            Token::String("value1".to_string()),
            Token::LogicalOperator("AND".to_string()),
            Token::Identifier("1column".to_string()),
            Token::ComparisonOperator("=".to_string()),
            Token::String("value2".to_string()),
            Token::Symbol(';'),
        ];
        assert_eq!(tokenize(input).unwrap(), expected_output);
    }
}
