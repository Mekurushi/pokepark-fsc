mod error;

use crate::ast::{BinOp, Expr, FuncDef, Param, Stmt, Ty};
use crate::lexer::Token;
use crate::parser::error::{ParseError, ParseResult};

// TODO: rethink recursive descent parsing; good enough for the prototype for now

struct TokenStream {
    tokens: Vec<(Token, std::ops::Range<usize>)>,
    cursor: usize,
}

impl TokenStream {
    fn new(tokens: Vec<(Token, std::ops::Range<usize>)>) -> Self {
        Self { tokens, cursor: 0 }
    }
    fn is_at_end(&self) -> bool {
        self.cursor >= self.tokens.len()
    }

    fn offset(&self) -> usize {
        self.tokens
            .get(self.cursor)
            .map_or(usize::MAX, |(_, s)| s.start)
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.cursor).map(|(t, _)| t)
    }
    fn advance(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.cursor).map(|(t, _)| t);
        if !self.is_at_end() {
            self.cursor += 1;
        }
        tok
    }
    fn eat(&mut self, expected: &Token) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, expected: &Token, label: &'static str) -> ParseResult<()> {
        if self.eat(expected) {
            return Ok(());
        }
        let got = self.peek().map_or("EOF".into(), |t| format!("{t:?}"));
        Err(ParseError::UnexpectedToken {
            got,
            expected: label,
            offset: self.offset(),
        })
    }

    fn expect_ty_keyword(&mut self) -> ParseResult<(Token)> {
        let token = self.peek();
        match token {
            Some(Token::KwInt) => {
                self.expect(&Token::KwInt, "type keyword");
                Ok((Token::KwInt))
            }
            _ => {
                let got = self.peek().map_or("EOF".into(), |t| format!("{t:?}"));
                Err(ParseError::UnexpectedToken {
                    got,
                    expected: "type keyword",
                    offset: self.offset(),
                })
            }
        }
    }
    fn eat_ident(&mut self) -> Option<String> {
        if let Some(Token::Ident(_)) = self.peek() {
            if let Some(Token::Ident(s)) = self.advance() {
                return Some(s.clone());
            }
        }
        None
    }
    fn expect_ident(&mut self) -> ParseResult<String> {
        self.eat_ident().ok_or_else(|| {
            let got = self.peek().map_or("EOF".into(), |t| format!("{t:?}"));
            ParseError::UnexpectedToken {
                got,
                expected: "identifier",
                offset: self.offset(),
            }
        })
    }

    fn expect_int(&mut self) -> ParseResult<i32> {
        if let Some(Token::IntLit(_)) = self.peek() {
            if let Some(Token::IntLit(n)) = self.advance() {
                return Ok(*n);
            }
        }
        let got = self.peek().map_or("EOF".into(), |t| format!("{t:?}"));
        Err(ParseError::UnexpectedToken {
            got,
            expected: "integer",
            offset: self.offset(),
        })
    }
}

// TODO: higher level
pub fn parse(src: Vec<(Token, std::ops::Range<usize>)>) -> ParseResult<FuncDef> {
    let mut ts = TokenStream::new(src);
    let function = parse_function(&mut ts)?;
    Ok(function)
}

fn parse_function(ts: &mut TokenStream) -> ParseResult<FuncDef> {
    // TODO: replace hardcoded; find syntax for exported
    let ret_ty = retrieve_ty_ast(ts)?;
    // function name
    let name = ts.expect_ident()?;

    // parameters
    ts.expect(&Token::LParen, "`(`")?;
    let params = parse_param_list(ts)?;

    // body
    ts.expect(&Token::LBrace, "`{`")?;
    let body = parse_body(ts)?;
    Ok(FuncDef {
        name,
        params,
        ret_ty,
        body,
        exported: true,
    })
}

fn retrieve_ty_ast(ts: &mut TokenStream) -> ParseResult<Ty> {
    // ensure it's a ty keyword
    let token = ts.expect_ty_keyword()?;
    match token {
        Token::KwInt => Ok(Ty::Int),
        other => Err(ParseError::UnexpectedToken {
            got: format!("{other:?}"),
            expected: "type keyword",
            offset: ts.offset(),
        }),
    }
}

fn parse_param_list(ts: &mut TokenStream) -> ParseResult<Vec<Param>> {
    // TODO: correct parsing for  empty params
    let mut params: Vec<Param> = Vec::new();

    while !ts.eat(&Token::RParen) {
        params.push(retrieve_param_ast(ts)?);
        ts.eat(&Token::Comma);
    }
    Ok(params)
}

fn retrieve_param_ast(ts: &mut TokenStream) -> ParseResult<Param> {
    let ty = retrieve_ty_ast(ts)?;
    let name = ts.expect_ident()?;
    Ok(Param { name, ty })
}

fn parse_body(ts: &mut TokenStream) -> ParseResult<Vec<Stmt>> {
    let mut body = Vec::new();

    while !ts.eat(&Token::RBrace) {
        body.push(parse_stmt(ts)?);
    }

    Ok(body)
}

fn parse_stmt(ts: &mut TokenStream) -> ParseResult<Stmt> {
    match ts.peek() {
        Some(Token::KwReturn) => parse_return(ts),
        other => Err(ParseError::UnexpectedToken {
            got: format!("{other:?}"),
            expected: "statement",
            offset: ts.offset(),
        }),
    }
}

fn parse_return(ts: &mut TokenStream) -> ParseResult<Stmt> {
    ts.expect(&Token::KwReturn, "`return`")?;

    if ts.eat(&Token::Semicolon) {
        // TODO: Stmt::Return(None) when Ty::Void is added
        return Err(ParseError::UnexpectedToken {
            got: "`;`".into(),
            expected: "expression",
            offset: ts.offset(),
        });
    }

    let expr = parse_expr(ts)?;
    ts.expect(&Token::Semicolon, "`;`")?;
    Ok(Stmt::Return(expr))
}

fn parse_expr(ts: &mut TokenStream) -> ParseResult<Expr> {
    // TODO: handling for other expressions
    parse_additive(ts)
}

fn parse_additive(ts: &mut TokenStream) -> ParseResult<Expr> {
    let mut lhs = parse_primary(ts)?;

    loop {
        let op = match ts.peek() {
            Some(Token::Plus) => BinOp::Add,
            _ => break, // TODO: handle case
        };
        ts.advance();
        let rhs = parse_primary(ts)?;
        lhs = Expr::BinOp {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        };
    }

    Ok(lhs)
}

fn parse_primary(ts: &mut TokenStream) -> ParseResult<Expr> {
    match ts.peek() {
        Some(Token::Ident(_)) => {
            let name = ts.expect_ident()?;
            Ok(Expr::Var(name))
        }
        Some(Token::IntLit(n)) => {
            let n = *n;
            ts.advance();
            Ok(Expr::IntLit(n))
        }
        other => Err(ParseError::UnexpectedToken {
            got: format!("{other:?}"),
            expected: "expression",
            offset: ts.offset(),
        }),
    }
}
