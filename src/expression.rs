use super::custom_error::CustomError;
use super::tokenizer::Token;
use std::iter::Peekable;

#[derive(Debug)]
pub enum Expression {
    True,
    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    Not(Box<Expression>),
    Comparison {
        left: Operand,
        operator: String,
        right: Operand,
    },
}

#[derive(Debug)]
pub enum Operand {
    Column(String),
    String(String),
    Integer(String),
}
