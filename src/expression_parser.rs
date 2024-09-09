use super::custom_error::CustomError;
use super::expression::{Expression, Operand};
use super::tokenizer::Token;
use std::iter::Peekable;

pub fn parse_expression(
    tokens: &mut std::iter::Peekable<std::slice::Iter<Token>>,
) -> Result<Expression, CustomError> {
    parse_or_expression(tokens)
}

fn parse_or_expression(
    tokens: &mut Peekable<std::slice::Iter<Token>>,
) -> Result<Expression, CustomError> {
    let mut expression = parse_and_expression(tokens)?;
    while let Some(Token::LogicalOperator(op)) = tokens.peek() {
        if op == "OR" {
            tokens.next();
            let right = parse_and_expression(tokens)?;
            expression = Expression::Or {
                left: Box::new(expression),
                right: Box::new(right),
            };
        } else {
            break;
        }
    }
    Ok(expression)
}

fn parse_and_expression(
    tokens: &mut Peekable<std::slice::Iter<Token>>,
) -> Result<Expression, CustomError> {
    let mut expression = parse_not_expression(tokens)?;
    while let Some(Token::LogicalOperator(op)) = tokens.peek() {
        if op == "AND" {
            tokens.next();
            let right = parse_not_expression(tokens)?;
            expression = Expression::And {
                left: Box::new(expression),
                right: Box::new(right),
            };
        } else {
            break;
        }
    }
    Ok(expression)
}

fn parse_not_expression(
    tokens: &mut Peekable<std::slice::Iter<Token>>,
) -> Result<Expression, CustomError> {
    if let Some(Token::LogicalOperator(op)) = tokens.peek() {
        if op == "NOT" {
            tokens.next();
            let expression = parse_primary_expression(tokens)?;
            return Ok(Expression::Not {
                right: Box::new(expression),
            });
        }
    }
    parse_primary_expression(tokens)
}

fn parse_primary_expression(
    tokens: &mut Peekable<std::slice::Iter<Token>>,
) -> Result<Expression, CustomError> {
    if let Some(Token::Symbol('(')) = tokens.peek() {
        tokens.next();
        let expression = parse_expression(tokens)?;
        if let Some(Token::Symbol(')')) = tokens.next() {
            return Ok(expression);
        } else {
            return Err(CustomError::InvalidSyntax {
                message: "Missing closing ')'".to_string(),
            });
        }
    }
    parse_comparison_expression(tokens)
}

fn parse_comparison_expression(
    tokens: &mut Peekable<std::slice::Iter<Token>>,
) -> Result<Expression, CustomError> {
    if let Some(token) = tokens.peek() {
        match token {
            Token::Identifier(_) | Token::String(_) | Token::Integer(_) => {
                let left = parse_operand(tokens)?;
                if let Some(Token::ComparisonOperator(op)) = tokens.next() {
                    let right = parse_operand(tokens)?;
                    return Ok(Expression::Comparison {
                        left,
                        operator: op.to_string(),
                        right,
                    });
                }
            }
            _ => {
                return Err(CustomError::InvalidSyntax {
                    message: "Invalid expression".to_string(),
                })
            }
        }
    }
    Err(CustomError::InvalidSyntax {
        message: "Invalid expression".to_string(),
    })
}

fn parse_operand(tokens: &mut Peekable<std::slice::Iter<Token>>) -> Result<Operand, CustomError> {
    if let Some(token) = tokens.next() {
        match token {
            Token::Identifier(string) => return Ok(Operand::Column(string.to_string())),
            Token::String(string) => return Ok(Operand::String(string.to_string())),
            Token::Integer(int) => return Ok(Operand::Integer(int.to_string())),
            other => {
                return Err(CustomError::InvalidSyntax {
                    message: format!("Invalid operand {:?}", other),
                })
            }
        }
    }
    Err(CustomError::InvalidSyntax {
        message: "No operand provided".to_string(),
    })
}
