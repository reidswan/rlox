use crate::data::ast::{Expression, ExpressionItem, Statement, StatementItem};
use crate::data::literals::Literal;
use crate::data::tokens::{Token, TokenMeta};

pub struct Parser {
    tokens: Vec<TokenMeta>,
    current: usize,
}

type ParseResult<T> = Result<T, String>;

macro_rules! binary_expression_parser {
    ($name:ident,$next:path,$first_match:path,$($rest_match:path),*) => {
        fn $name(&mut self)-> ParseResult<Expression> {
            let mut expr = $next(self)?;
            while let Some(token_meta) = self.peek() {
                let token = token_meta.item_clone();
                if let $first_match = token {
                    self.advance();
                }
                $(
                else if let $rest_match = token {
                    self.advance();
                }
                )*
                else {
                    break
                }
                let right = Box::new($next(self)?);
                let line = expr.line();
                expr = Expression::new(ExpressionItem::Binary {
                    left: Box::new(expr),
                    operator: token.clone(),
                    right,
                }, line)
            }
            Ok(expr)
        }
    };
}

macro_rules! logical_expression_parser {
    ($name:ident,$next:path,$token:path) => {
        fn $name(&mut self)-> ParseResult<Expression> {
            let mut expr = $next(self)?;
            while let Some(token_meta) = self.peek() {
                let token = token_meta.item_clone();
                if let $token = token {
                    self.advance();
                }
                else {
                    break
                }
                let right = Box::new($next(self)?);
                let line = expr.line();
                expr = Expression::new(ExpressionItem::Logical {
                    left: Box::new(expr),
                    operator: token.clone(),
                    right,
                }, line)
            }
            Ok(expr)
        }
    };
}

macro_rules! match_head {
    ($self:ident,$token_type:path) => {{
        match $self.peek() {
            None => false,
            Some(token_meta) => {
                if let $token_type = token_meta.item() {
                    true
                } else {
                    false
                }
            }
        }
    }};
}

macro_rules! consume {
    ($self:ident, $token_type:path) => {{
        let should_match = $self
            .peek()
            .ok_or(format!("EOF: Expected {} but got EOF", $token_type))?;
        let line = should_match.line();
        let token = should_match.item_clone();
        if let $token_type = token {
            $self.advance();
            Ok(line)
        } else {
            Err(format!(
                "Line {}: Expected {} but got {}",
                line, $token_type, token
            ))
        }
    }};
}

impl Parser {
    pub fn new(src: Vec<TokenMeta>) -> Self {
        Parser {
            tokens: src,
            current: 0,
        }
    }

    pub fn parse(&mut self) -> ParseResult<Vec<Statement>> {
        let mut statements = vec![];
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        Ok(statements)
    }

    pub fn parse_top_level_expression(&mut self) -> ParseResult<Vec<Statement>> {
        self.expression()
            .map(|expr| vec![Statement::new(StatementItem::PrintStatement(expr), 1)])
    }

    fn synchronize(&mut self) {
        use Token::*;
        self.advance();
        while !self.is_at_end() {
            let token_meta = self.peek().unwrap();
            match token_meta.item() {
                Class | Fun | Var | For | If | While | Print | Return => return,
                Semicolon => {
                    self.advance();
                    return;
                }
                _ => (),
            }
            self.advance();
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    // declaration := <var_declaration> | <statement>
    fn declaration(&mut self) -> ParseResult<Statement> {
        let result = if match_head!(self, Token::Var) {
            self.var_declaration()
        } else {
            self.statement()
        };

        if let Err(_) = &result {
            self.synchronize();
        }

        result
    }

    // var_declaration := var <id> = <expression>;
    fn var_declaration(&mut self) -> ParseResult<Statement> {
        consume!(self, Token::Var)?;
        let identifier = self
            .peek()
            .ok_or(format!("EOF: Expected identifier but got EOF"))?;
        let line = identifier.line();
        match identifier.item_clone() {
            Token::Literal(Literal::Identifier(name)) => {
                self.advance();
                consume!(self, Token::Equal)?;
                let expression = self.expression()?;
                consume!(self, Token::Semicolon)?;
                Ok(Statement::new(
                    StatementItem::Declaration {
                        name: name.clone(),
                        initializer: expression,
                    },
                    line,
                ))
            }
            t => Err(format!("EOF: Expected identifier but got {}", t)),
        }
    }

    // statement := <print_statement> | <expression_statement> | <block> | <if_statement>
    fn statement(&mut self) -> ParseResult<Statement> {
        if match_head!(self, Token::Print) {
            self.print_statement()
        } else if match_head!(self, Token::LeftBrace) {
            self.block()
        } else if match_head!(self, Token::If) {
            self.if_statement()
        } else if match_head!(self, Token::While) {
            self.while_statement()
        } else if match_head!(self, Token::For) {
            self.for_statement()
        } else {
            self.expression_statement()
        }
    }

    // while_statement := while '(' <expression> ')' <statement>
    fn while_statement(&mut self) -> ParseResult<Statement> {
        let line = consume!(self, Token::While)?;
        consume!(self, Token::LeftParen)?;
        let test = self.expression()?;
        consume!(self, Token::RightParen)?;
        let body = Box::new(self.statement()?);

        Ok(Statement::new(
            StatementItem::WhileStatement { test, body },
            line,
        ))
    }

    // for_statement := for '(' ( <expression> | <var_declaration> )?; <expression>?; <expression>? ')' <statement>
    fn for_statement(&mut self) -> ParseResult<Statement> {
        // convert 
        // `for (<init>; <test>; <update>) <body>`
        // into the equivalent:
        // ```
        // <init>;
        // while ( <test> ) {
        //     <body>
        //     <update>
        // }
        // ```

        let line = consume!(self, Token::For)?;
        consume!(self, Token::LeftParen)?;
        let initializer = if match_head!(self, Token::Semicolon) {
            None
        } else if match_head!(self, Token::Var) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };
        consume!(self, Token::Semicolon);

        let condition = if !match_head!(self, Token::Semicolon) {
            self.expression()?
        } else {
            // the for loop has no condition, equiv to a while (true) {...}
            Expression::new(
                ExpressionItem::Literal {
                    value: Literal::True,
                },
                line,
            )
        };
        consume!(self, Token::Semicolon);

        let increment = if !match_head!(self, Token::RightParen) {
            let expression = self.expression()?;
            let expression_line = expression.line();
            Some(Statement::new(
                StatementItem::ExpressionStatement(expression),
                expression_line,
            ))
        } else {
            None
        };
        consume!(self, Token::RightParen);

        let for_body = self.statement()?;
        // if the for loop has an increment/update section, 
        // put it at the end of the while loop body
        let body = Box::new(if let Some(increment) = increment {
            let line = increment.line();
            Statement::new(
                StatementItem::Block {
                    statements: vec![for_body, increment],
                },
                line,
            )
        } else {
            for_body
        });
        let mut statements = vec![];
        if let Some(initializer) = initializer {
            statements.push(initializer);
        }

        statements.push(Statement::new(
            StatementItem::WhileStatement {
                test: condition,
                body,
            },
            line,
        ));
        Ok(Statement::new(StatementItem::Block { statements }, line))
    }

    // if_statement := if '(' <expression> ')' statement ( <else> statement )?
    fn if_statement(&mut self) -> ParseResult<Statement> {
        let start_line = consume!(self, Token::If)?;
        consume!(self, Token::LeftParen)?;
        let test = self.expression()?;
        consume!(self, Token::RightParen)?;
        let when_true = Box::new(self.statement()?);
        let when_false = if match_head!(self, Token::Else) {
            self.advance();
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Statement::new(
            StatementItem::IfStatement {
                test,
                when_true,
                when_false,
            },
            start_line,
        ))
    }

    // block := { <declaration>* }
    fn block(&mut self) -> ParseResult<Statement> {
        let start_line = consume!(self, Token::LeftBrace)?;
        let mut statements = vec![];
        while !match_head!(self, Token::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        consume!(self, Token::RightBrace)?;
        Ok(Statement::new(
            StatementItem::Block { statements },
            start_line,
        ))
    }

    // print_statement := print <expression>
    fn print_statement(&mut self) -> ParseResult<Statement> {
        consume!(self, Token::Print)?;
        let expression = self.expression()?;
        consume!(self, Token::Semicolon)?;
        let line = expression.line();
        Ok(Statement::new(
            StatementItem::PrintStatement(expression),
            line,
        ))
    }

    // expression_statement := <expression>;
    fn expression_statement(&mut self) -> ParseResult<Statement> {
        let expression = self.expression()?;
        consume!(self, Token::Semicolon)?;
        let line = expression.line();
        Ok(Statement::new(
            StatementItem::ExpressionStatement(expression),
            line,
        ))
    }

    // expression := <assignment>
    fn expression(&mut self) -> ParseResult<Expression> {
        self.assigment()
    }

    // assignment := <id> = <assignment> | <ternary>
    fn assigment(&mut self) -> ParseResult<Expression> {
        let lhs = self.ternary()?;
        if match_head!(self, Token::Equal) {
            let line = self.peek().unwrap().line();
            self.advance();
            let rhs = self.assigment()?;
            if let ExpressionItem::Variable { name } = lhs.item() {
                Ok(Expression::new(
                    ExpressionItem::Assignment {
                        name: name.clone(),
                        value: Box::new(rhs),
                    },
                    lhs.line(),
                ))
            } else {
                Err(format!(
                    "Line {}: Invalid assignment target: {:?}",
                    line, lhs
                ))
            }
        } else {
            Ok(lhs)
        }
    }

    fn ternary(&mut self) -> ParseResult<Expression> {
        let first = self.logical_or()?;
        // try to parse a ternary operator
        match self.peek() {
            None => return Ok(first),
            Some(token_meta) => {
                let line = token_meta.line();
                let token = token_meta.item_clone();
                match token {
                    Token::Question => {
                        self.advance();
                        let when_true = self.logical_or()?;
                        let should_be_colon = self
                            .peek()
                            .ok_or(String::from("EOF: Expected ':' but got EOF"))?;
                        if let Token::Colon = should_be_colon.item() {
                            self.advance();
                            let when_false = self.logical_or()?;
                            Ok(Expression::new(
                                ExpressionItem::Ternary {
                                    test: Box::new(first),
                                    when_true: Box::new(when_true),
                                    when_false: Box::new(when_false),
                                },
                                line,
                            ))
                        } else {
                            Err(format!("Line {}: Expected ':' but got {}", line, token))
                        }
                    }
                    _ => return Ok(first),
                }
            }
        }
    }

    // logical_or := <logical_and> (or <logical_and>)*
    logical_expression_parser!(logical_or, Self::logical_and, Token::Or);

    // logical_and := <equality> (and <equality>)*
    logical_expression_parser!(logical_and, Self::equality, Token::And);

    // equality := <comparison> ( (== | !=) <comparison>)*
    binary_expression_parser!(
        equality,
        Self::comparison,
        Token::EqualEqual,
        Token::BangEqual
    );

    // equality := <addition> ( (> | < | >= | <=) <addition>)*
    binary_expression_parser!(
        comparison,
        Self::addition,
        Token::Greater,
        Token::GreaterEqual,
        Token::Lesser,
        Token::LesserEqual
    );
    // addition := <multiplication> ( (+ | -) <multiplication>)*
    binary_expression_parser!(addition, Self::multiplication, Token::Plus, Token::Minus);
    // multiplication := <unary> ( (* | /) <comparison>)*
    binary_expression_parser!(multiplication, Self::unary, Token::Slash, Token::Star);
    // unary := (+ | - | !)? <primary>
    fn unary(&mut self) -> ParseResult<Expression> {
        let token_meta = self.peek().ok_or(String::from(
            "EOF: No more tokens while parsing a unary expression",
        ))?;
        let line = token_meta.line();
        let token = token_meta.item_clone();
        match token {
            Token::Plus | Token::Minus => {
                self.advance();
                let right = Box::new(self.unary()?);
                Ok(Expression::new(
                    ExpressionItem::Unary {
                        operator: token.clone(),
                        operand: right,
                    },
                    line,
                ))
            }
            t if t.is_operator() => {
                self.advance();
                self.primary().ok(); // try to consume the right operand
                Err(format!(
                    "Line {}: {} operator requires left operand",
                    line, t
                ))
            }
            _ => self.primary(),
        }
    }

    // primary := <literal> | <id> | ( <expression> )
    fn primary(&mut self) -> ParseResult<Expression> {
        let token_meta = self.peek().ok_or(String::from(
            "EOF: No more tokens while parsing a primary expression",
        ))?;
        let line_number = token_meta.line();
        let token = token_meta.item_clone();
        if let Token::Literal(l) = token {
            self.advance();
            return Ok(Expression::new(
                if let Literal::Identifier(name) = l {
                    ExpressionItem::Variable { name }
                } else {
                    ExpressionItem::Literal { value: l }
                },
                line_number,
            ));
        }

        match token {
            Token::LeftParen => {
                self.advance();
                let expr = self.expression()?;
                let _next = self
                    .peek()
                    .filter(|next_token_meta| match next_token_meta.item() {
                        Token::RightParen => true,
                        _ => false,
                    })
                    .ok_or(format!(
                        "Line {}: Expected closing parenthesis for expression",
                        line_number
                    ))?;
                self.advance();
                Ok(expr)
            }
            t => Err(String::from(format!(
                "Line {}: Failed to parse {}; expected expression",
                line_number, t
            ))),
        }
    }

    fn advance(&mut self) {
        self.current += 1
    }

    fn peek<'a>(&'a self) -> Option<&'a TokenMeta> {
        if self.is_at_end() {
            None
        } else {
            Some(&self.tokens[self.current])
        }
    }

    pub fn reset(&mut self) {
        self.current = 0;
    }
}
