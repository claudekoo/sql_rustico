use super::custom_error::CustomError;
use std::collections::HashMap;

#[derive(Debug)]
pub enum Expression {
    True,
    And {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Or {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Not {
        right: Box<Expression>,
    },
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

pub fn evaluate_expression(
    expression: &Expression,
    row: &HashMap<String, String>,
) -> Result<bool, CustomError> {
    match expression {
        Expression::True => Ok(true),
        Expression::And { left, right } => {
            let left_result = evaluate_expression(left, row)?;
            let right_result = evaluate_expression(right, row)?;
            Ok(left_result && right_result)
        }
        Expression::Or { left, right } => {
            let left_result = evaluate_expression(left, row)?;
            let right_result = evaluate_expression(right, row)?;
            Ok(left_result || right_result)
        }
        Expression::Not { right } => {
            let right_result = evaluate_expression(right, row)?;
            Ok(!right_result)
        }
        Expression::Comparison {
            left,
            operator,
            right,
        } => {
            let left_value = evaluate_operand(left, row)?;
            let right_value = evaluate_operand(right, row)?;
            match operator.as_str() {
                "=" => Ok(left_value == right_value),
                "!=" => Ok(left_value != right_value),
                ">" => Ok(left_value > right_value),
                "<" => Ok(left_value < right_value),
                ">=" => Ok(left_value >= right_value),
                "<=" => Ok(left_value <= right_value),
                _ => Err(CustomError::GenericError {
                    message: format!("Invalid operator: {}", operator),
                }),
            }
        }
    }
}

fn evaluate_operand(
    operand: &Operand,
    row: &HashMap<String, String>,
) -> Result<String, CustomError> {
    match operand {
        Operand::Column(column_name) => {
            if let Some(value) = row.get(column_name) {
                Ok(value.to_string())
            } else {
                Err(CustomError::GenericError {
                    message: format!("Column not found: {}", column_name),
                })
            }
        }
        Operand::String(value) => Ok(value.to_string()),
        Operand::Integer(value) => Ok(value.to_string()),
    }
}
