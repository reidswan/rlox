use super::literals::Literal;
use super::tokens::Token;
use super::meta::MetaContainer;
use std::fmt;

#[derive(Clone)]
pub enum ExpressionItem {
    Binary { left: Box<Expression>, operator: Token, right: Box<Expression> },
    Logical { left: Box<Expression>, operator: Token, right: Box<Expression> },
    Grouping { expression: Box<Expression> },
    Literal { value: Literal },
    Unary { operator: Token, operand: Box<Expression> },
    Ternary { test: Box<Expression>, when_true: Box<Expression>, when_false: Box<Expression> },
    Variable { name: String },
    Assignment { name: String, value: Box<Expression> }
}

impl fmt::Debug for ExpressionItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ExpressionItem::*;
        match self {
            Binary {left, operator, right} => write!(f, "({} {:?} {:?})", operator, left, right),
            Logical { left, operator, right } => write!(f, "({} {:?} {:?})", operator, left, right),
            Grouping {expression} => write!(f, "(group {:?})", expression),
            Literal { value } => write!(f, "(literal {})", value),
            Unary {operator, operand} => write!(f, "({} {:?})", operator, operand),
            Ternary {test, when_true, when_false} => write!(f, "(?: {:?} {:?} {:?})", test, when_true, when_false),
            Variable { name } => write!(f, "(var {})", name),
            Assignment { name, value } => write!(f, "(set! {} {:?})", name, value)
        }
    }
}

#[derive(Debug, Clone)]
pub enum StatementItem {
    ExpressionStatement(Expression),
    PrintStatement(Expression),
    Declaration { name: String, initializer: Expression },
    Block { statements: Vec<Statement> },
    IfStatement { test: Expression, when_true: Box<Statement>, when_false: Option<Box<Statement>> },
    WhileStatement { test: Expression, body: Box<Statement> }
}

pub type Expression = MetaContainer<ExpressionItem>;
pub type Statement = MetaContainer<StatementItem>;
