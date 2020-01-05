use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Identifier(String),
    StringT(String),
    Integer(i64),
    Number(f64),
    True,
    False,
    Nil
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Literal::*;
        write!(
            f,
            "{}",
            match self {
                Identifier(s) => format!("id:{}", s),
                StringT(s) => format!("{}", s),
                Integer(i) => format!("{}", i),
                Number(f) => format!("{}", f),
                True => "true".to_owned(),
                False => "false".to_owned(),
                Nil => "nil".to_owned()
            }
        )
    }
}
