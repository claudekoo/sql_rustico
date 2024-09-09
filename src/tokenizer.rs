use super::custom_error::CustomError;

#[derive(Debug)]
pub enum Token {
    Keyword(String), // palabra clave: INSERT, UPDATE, DELETE, SELECT, FROM, WHERE, SET, INTO, VALUES, ORDER, BY, DESC
    LogicalOperator(String), // AND, OR, NOT
    ComparisonOperator(String), // =, >, <, >=, <=
    Identifier(String), // nombre de tabla o columna
    String(String),  // strings entre comillas
    Integer(String), // enteros
    Symbol(char),    // simbolos: , ( ) ; *
}

fn tokenize_integer(chars: &mut std::iter::Peekable<std::str::Chars>) -> Token {
    let mut number = String::new();
    while let Some(&ch) = chars.peek() {
        if ch.is_digit(10) {
            number.push(ch);
            chars.next();
        } else {
            break;
        }
    }
    Token::Integer(number)
}

// por como se tokeniza, no se va a poder tomar palabras empezando con numeros
fn tokenize_word(chars: &mut std::iter::Peekable<std::str::Chars>) -> Token {
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
        "INSERT", "UPDATE", "DELETE", "SELECT", "FROM", "WHERE", "SET", "INTO", "VALUES", "ORDER", "BY", "DESC",
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

fn tokenize_string(chars: &mut std::iter::Peekable<std::str::Chars>) -> Token {
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

fn tokenize_comparison_operator(chars: &mut std::iter::Peekable<std::str::Chars>) -> Token {
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

pub fn tokenize(input: &str) -> Result<Vec<Token>, CustomError> {
    let mut tokens = vec![];
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() {
            // ignorar espacios y newlines
            chars.next(); // esto ocurre solo cuando no esta entre comillas
        } else if ch.is_digit(10) {
            tokens.push(tokenize_integer(&mut chars));
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
            return Err(CustomError::InvalidSyntax {
                message: format!("Invalid syntax near: {}", ch),
            });
        }
    }

    Ok(tokens)
}
