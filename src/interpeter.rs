use crate::data::ast::{Expression, ExpressionItem, Statement, StatementItem};
use crate::data::literals::Literal;
use crate::data::tokens::Token;
use crate::environment::Environment;
use std::fmt;
use std::rc::Rc;

pub struct Interpreter {
    environment: Environment,
}

#[derive(Debug)]
pub enum LoxData {
    ByValue(Literal),
    ByReference(Rc<Literal>),
}

impl LoxData {
    fn as_ref<'a>(&'a self) -> &'a Literal {
        match self {
            LoxData::ByValue(l) => &l,
            LoxData::ByReference(r) => r.as_ref(),
        }
    }
}

impl fmt::Display for LoxData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LoxData::ByValue(l) => write!(f, "{}", l),
            LoxData::ByReference(l) => write!(f, "{}", l),
        }
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            environment: Environment::new(),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Statement>) -> Result<(), String> {
        statements
            .iter()
            .map(|s| self.evaluate_statement(s))
            .collect::<Result<_, _>>()?;
        Ok(())
    }

    fn evaluate_statement(&mut self, statement: &Statement) -> Result<(), String> {
        match statement.item() {
            StatementItem::ExpressionStatement(expr) => {
                self.evaluate_expression(expr)?;
            }
            StatementItem::PrintStatement(expr) => {
                let result = self.evaluate_expression(expr)?;
                println!("{}", result);
            }
            StatementItem::Declaration { name, initializer } => {
                let value = self.evaluate_expression(initializer)?;
                self.environment.define(
                    name.clone(),
                    match value {
                        LoxData::ByValue(l) => l,
                        LoxData::ByReference(rc) => rc.as_ref().clone(),
                    },
                );
            }
            StatementItem::Block { statements } => {
                self.environment.fork();
                let mut result = None;
                for statement in statements.iter() {
                    if let Err(e) = self.evaluate_statement(statement) {
                        result = Some(e);
                        break;
                    }
                }

                self.environment
                    .join()
                    .expect("Failed to join on the environment!");

                if let Some(e) = result {
                    return Err(e);
                }
            }
            StatementItem::IfStatement { test, when_true, when_false } => {
                let eval_test = self.evaluate_expression(test)?;
                if as_boolean(eval_test.as_ref()) {
                    self.evaluate_statement(when_true)?;
                } else if let Some(when_false) = when_false {
                    self.evaluate_statement(when_false)?;
                }
            }
            StatementItem::WhileStatement { test, body } => {
                loop {
                    let test_result = self.evaluate_expression(test)?;
                    if as_boolean(test_result.as_ref()) {
                        self.evaluate_statement(body)?;
                    } else {
                        break
                    }
                }
            }
        }
        Ok(())
    }

    fn evaluate_expression(&mut self, expression: &Expression) -> Result<LoxData, String> {
        use LoxData::*;
        let expression_line = expression.line();
        match expression.item() {
            ExpressionItem::Grouping { expression } => {
                self.evaluate_expression(expression.as_ref())
            }
            ExpressionItem::Literal { value } => Ok(ByValue(value.clone())),
            ExpressionItem::Unary { operator, operand } => {
                let operand_line = operand.line();
                let eval_operand = self.evaluate_expression(operand)?;
                match operator {
                    Token::Plus => Ok(eval_operand),
                    Token::Minus => match eval_operand.as_ref() {
                        Literal::Integer(i) => Ok(ByValue(Literal::Integer(-i))),
                        Literal::Number(n) => Ok(ByValue(Literal::Number(-n))),
                        v => Err(format!(
                            "Line {}: Type Error: cannot negate {:?}",
                            operand_line, v
                        )),
                    },
                    Token::Bang => Ok(ByValue(from_boolean(negate(eval_operand.as_ref())))),
                    _ => Err(format!(
                        "Line {}: Unexpected unary operator {:?}",
                        operand_line, operator
                    )),
                }
            }
            ExpressionItem::Logical {
                left,
                operator,
                right
            } => {
                let line = left.line();
                let eval_left = self.evaluate_expression(left)?;
                let left_bool = as_boolean(eval_left.as_ref());
                match operator {
                    Token::And => {
                        if !left_bool {
                            return Ok(eval_left)
                        }
                    }
                    Token::Or => {
                        if left_bool {
                            return Ok(eval_left)
                        }
                    }
                    _ => return Err(format!("Line {}: {} is not supported as a logical operator", line, operator))
                }
                self.evaluate_expression(right)
            }
            ExpressionItem::Binary {
                left,
                operator,
                right,
            } => {
                let line = left.line();
                let eval_left = self.evaluate_expression(left)?;
                let eval_right = self.evaluate_expression(right)?;
                let eval_left_ref = eval_left.as_ref();
                let eval_right_ref = eval_right.as_ref();
                match operator {
                    Token::Star => Ok(match (eval_left_ref, eval_right_ref) {
                        (Literal::Integer(left), Literal::Integer(right)) => {
                            ByValue(Literal::Integer(left * right))
                        }
                        (Literal::Integer(_), Literal::Number(_))
                        | (Literal::Number(_), Literal::Integer(_))
                        | (Literal::Number(_), Literal::Number(_)) => {
                            let left = as_f64(eval_left_ref);
                            let right = as_f64(eval_right_ref);
                            ByValue(Literal::Number(left * right))
                        }
                        _ => return Err(format!("Line {}: {} cannot be applied to the given types", line, operator))
                    }),
                    Token::Minus => Ok(match (eval_left_ref, eval_right_ref) {
                        (Literal::Integer(left), Literal::Integer(right)) => {
                            ByValue(Literal::Integer(left - right))
                        }
                        (Literal::Integer(_), Literal::Number(_))
                        | (Literal::Number(_), Literal::Integer(_))
                        | (Literal::Number(_), Literal::Number(_)) => {
                            let left = as_f64(eval_left_ref);
                            let right = as_f64(eval_right_ref);
                            ByValue(Literal::Number(left - right))
                        }
                        _ => return Err(format!("Line {}: {} cannot be applied to the given types", line, operator))
                    }),
                    Token::Plus => Ok(match (eval_left_ref, eval_right_ref) {
                        (Literal::Integer(left), Literal::Integer(right)) => {
                            ByValue(Literal::Integer(left + right))
                        }
                        (Literal::Integer(_), Literal::Number(_))
                        | (Literal::Number(_), Literal::Integer(_))
                        | (Literal::Number(_), Literal::Number(_)) => {
                            let left = as_f64(eval_left_ref);
                            let right = as_f64(eval_right_ref);
                            ByValue(Literal::Number(left + right))
                        }
                        (Literal::StringT(left), Literal::StringT(right)) => ByValue(Literal::StringT(left.clone() + &right[..])),
                        (Literal::StringT(left), right) => ByValue(Literal::StringT(format!("{}{}", left, right))),
                        _ => return Err(format!("Line {}: {} cannot be applied to the given types", line, operator))
                    }),
                    Token::Slash => Ok(match (eval_left_ref, eval_right_ref) {
                        (Literal::Integer(left), Literal::Integer(right)) => {
                            ByValue(Literal::Number(*left as f64 / *right as f64))
                        }
                        (Literal::Integer(_), Literal::Number(_))
                        | (Literal::Number(_), Literal::Integer(_))
                        | (Literal::Number(_), Literal::Number(_)) => {
                            let left = as_f64(eval_left_ref);
                            let right = as_f64(eval_right_ref);
                            ByValue(Literal::Number(left / right))
                        }
                        _ => return Err(format!("Line {}: {} cannot be applied to the given types", line, operator))
                    }),
                    Token::Greater => Ok(match (eval_left_ref, eval_right_ref) {
                        (Literal::Integer(left), Literal::Integer(right)) => {
                            ByValue(from_boolean(left > right))
                        }
                        (Literal::Integer(_), Literal::Number(_))
                        | (Literal::Number(_), Literal::Integer(_))
                        | (Literal::Number(_), Literal::Number(_)) => {
                            let left = as_f64(eval_left_ref);
                            let right = as_f64(eval_right_ref);
                            ByValue(from_boolean(left > right))
                        }
                        _ => return Err(format!("Line {}: {} cannot be applied to the given types", line, operator))
                    }),
                    Token::GreaterEqual => Ok(match (eval_left_ref, eval_right_ref) {
                        (Literal::Integer(left), Literal::Integer(right)) => {
                            ByValue(from_boolean(left >= right))
                        }
                        (Literal::Integer(_), Literal::Number(_))
                        | (Literal::Number(_), Literal::Integer(_))
                        | (Literal::Number(_), Literal::Number(_)) => {
                            let left = as_f64(eval_left_ref);
                            let right = as_f64(eval_right_ref);
                            ByValue(from_boolean(left >= right))
                        }
                        _ => return Err(format!("Line {}: {} cannot be applied to the given types", line, operator))
                    }),
                    Token::Lesser => Ok(match (eval_left_ref, eval_right_ref) {
                        (Literal::Integer(left), Literal::Integer(right)) => {
                            ByValue(from_boolean(left < right))
                        }
                        (Literal::Integer(_), Literal::Number(_))
                        | (Literal::Number(_), Literal::Integer(_))
                        | (Literal::Number(_), Literal::Number(_)) => {
                            let left = as_f64(eval_left_ref);
                            let right = as_f64(eval_right_ref);
                            ByValue(from_boolean(left < right))
                        }
                        _ => return Err(format!("Line {}: {} cannot be applied to the given types", line, operator))
                    }),
                    Token::LesserEqual => Ok(match (eval_left_ref, eval_right_ref) {
                        (Literal::Integer(left), Literal::Integer(right)) => {
                            ByValue(from_boolean(left <= right))
                        }
                        
                        (Literal::Integer(_), Literal::Number(_))
                        | (Literal::Number(_), Literal::Integer(_))
                        | (Literal::Number(_), Literal::Number(_)) => {
                            let left = as_f64(eval_left_ref);
                            let right = as_f64(eval_right_ref);
                            ByValue(from_boolean(left <= right))
                        }
                        _ => return Err(format!("Line {}: {} cannot be applied to the given types", line, operator))
                    }),
                    Token::EqualEqual => Ok(ByValue(from_boolean(eval_left_ref == eval_right_ref))),
                    Token::BangEqual => Ok(ByValue(from_boolean(eval_left_ref != eval_right_ref))),
                    _ => Err(format!(
                        "Line {}: {} is not a valid operator",
                        line, operator
                    )),
                }
            }
            ExpressionItem::Ternary {
                test,
                when_true,
                when_false,
            } => {
                let result = as_boolean(self.evaluate_expression(test)?.as_ref());

                if result {
                    self.evaluate_expression(when_true)
                } else {
                    // can only be Literal::False
                    self.evaluate_expression(when_false)
                }
            }
            ExpressionItem::Variable { name } => self
                .environment
                .get(&name)
                .map(|rc| ByReference(rc))
                .ok_or(format!(
                    "Line {}: Variable '{}' referenced before assignment",
                    expression_line, name
                )),
            ExpressionItem::Assignment { name, value } => {
                let result = match self.evaluate_expression(value)? {
                    ByValue(l) => self
                        .environment
                        .assign(name.clone(), l)
                        .map(|rc| ByReference(rc)),
                    ByReference(rc) => self
                        .environment
                        .assign_reference(name.clone(), rc)
                        .map(|rc| ByReference(rc)),
                };
                if let Err(e) = result {
                    Err(format!("Line {}: {}", expression_line, e))
                } else {
                    result
                }
            }
        }
    }
}

fn negate(literal: &Literal) -> bool {
    !as_boolean(literal)
}

fn as_boolean(literal: &Literal) -> bool {
    match literal {
        Literal::False | Literal::Nil => false,
        Literal::Identifier(_) => true, // TODO! evaluate the identifier's contents!
        _ => true,
    }
}

fn from_boolean(boolean: bool) -> Literal {
    if boolean {
        Literal::True
    } else {
        Literal::False
    }
}

fn as_f64(literal: &Literal) -> f64 {
    match literal {
        Literal::Integer(i) => *i as f64,
        Literal::Number(n) => *n,
        _ => panic!("Cannot cast {:?} to f64!", literal),
    }
}
