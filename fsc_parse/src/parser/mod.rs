mod error;

use crate::ast::{BinOp, Expr, FuncDef, Param, Stmt, Ty};
use crate::lexer::token::{Span, Token, TokenKind};
use crate::parser::error::{ParseError, ParseResult};

// TODO: rethink recursive descent parsing; good enough for the prototype for now

struct TokenStream {
    tokens: Vec<Token>,
    cursor: usize,
    source: String,
}

impl TokenStream {
    fn new(tokens: Vec<Token>, source: String) -> Self {
        Self {
            tokens,
            cursor: 0,
            source,
        }
    }
    fn is_at_end(&self) -> bool {
        self.cursor >= self.tokens.len()
    }

    fn offset(&self) -> usize {
        self.tokens
            .get(self.cursor)
            .map_or(usize::MAX, |t| t.span.start)
    }

    fn error_span(&self) -> Span {
        match self.tokens.get(self.cursor) {
            Some(t) => t.span,
            None => {
                let end = self.source.len();
                Span { start: end, end }
            }
        }
    }

    fn unexpected(&self, expected: &'static str) -> ParseError {
        match self.peek() {
            None => ParseError::UnexpectedEof {
                expected,
                span: self.error_span(),
            },
            Some(t) => ParseError::UnexpectedToken {
                got: t.description().into(),
                expected,
                span: self.error_span(),
            },
        }
    }

    fn peek(&self) -> Option<&TokenKind> {
        self.tokens.get(self.cursor).map(|t| &t.kind)
    }
    fn advance(&mut self) -> Option<&TokenKind> {
        let tok = self.tokens.get(self.cursor).map(|t| &t.kind);
        if !self.is_at_end() {
            self.cursor += 1;
        }
        tok
    }
    fn eat(&mut self, expected: &TokenKind) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, expected: &TokenKind, label: &'static str) -> ParseResult<()> {
        if self.eat(expected) {
            return Ok(());
        }
        let got = self.peek().map_or("EOF".into(), |t| t.description().into());
        Err(ParseError::UnexpectedToken {
            got,
            expected: label,
            span: self.error_span(),
        })
    }

    fn expect_ty_keyword(&mut self) -> ParseResult<(TokenKind)> {
        let token = self.peek();
        match token {
            Some(TokenKind::KwInt) => {
                self.expect(&TokenKind::KwInt, "type keyword");
                Ok((TokenKind::KwInt))
            }
            _ => Err(self.unexpected("type keyword")),
        }
    }
    fn eat_ident(&mut self) -> Option<String> {
        if let Some(TokenKind::Ident(_)) = self.peek() {
            if let Some(TokenKind::Ident(s)) = self.advance() {
                return Some(s.clone());
            }
        }
        None
    }
    fn expect_ident(&mut self) -> ParseResult<String> {
        self.eat_ident()
            .ok_or_else(|| self.unexpected("identifier"))
    }

    fn expect_int(&mut self) -> ParseResult<i32> {
        if let Some(TokenKind::IntLit(_)) = self.peek() {
            if let Some(TokenKind::IntLit(n)) = self.advance() {
                return Ok(*n);
            }
        }
        Err(self.unexpected("integer"))
    }
}

// TODO: higher level
pub fn parse(tokens: Vec<Token>, src: String) -> ParseResult<FuncDef> {
    let mut ts = TokenStream::new(tokens, src);
    let function = parse_function(&mut ts)?;
    Ok(function)
}

fn parse_function(ts: &mut TokenStream) -> ParseResult<FuncDef> {
    // TODO: replace hardcoded; find syntax for exported
    let ret_ty = retrieve_ty_ast(ts)?;
    // function name
    let name = ts.expect_ident()?;

    // parameters
    ts.expect(&TokenKind::LParen, "`(`")?;
    let params = parse_param_list(ts)?;

    // body
    ts.expect(&TokenKind::LBrace, "`{`")?;
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
        TokenKind::KwInt => Ok(Ty::Int),
        _ => Err(ts.unexpected("type keyword")),
    }
}

fn parse_param_list(ts: &mut TokenStream) -> ParseResult<Vec<Param>> {
    // TODO: correct parsing for  empty params
    let mut params: Vec<Param> = Vec::new();

    while !ts.eat(&TokenKind::RParen) {
        params.push(retrieve_param_ast(ts)?);
        ts.eat(&TokenKind::Comma);
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

    while !ts.eat(&TokenKind::RBrace) {
        body.push(parse_stmt(ts)?);
    }

    Ok(body)
}

fn parse_stmt(ts: &mut TokenStream) -> ParseResult<Stmt> {
    match ts.peek() {
        Some(TokenKind::KwReturn) => parse_return(ts),
        _ => Err(ts.unexpected("statement")),
    }
}

fn parse_return(ts: &mut TokenStream) -> ParseResult<Stmt> {
    ts.expect(&TokenKind::KwReturn, "`return`")?;

    if ts.eat(&TokenKind::Semicolon) {
        // TODO: Stmt::Return(None) when Ty::Void is added
        return Err(ts.unexpected("expression"));
    }

    let expr = parse_expr(ts)?;
    ts.expect(&TokenKind::Semicolon, "`;`")?;
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
            Some(TokenKind::Plus) => BinOp::Add,
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
        Some(TokenKind::Ident(_)) => {
            let name = ts.expect_ident()?;
            Ok(Expr::Var(name))
        }
        Some(TokenKind::IntLit(n)) => {
            let n = *n;
            ts.advance();
            Ok(Expr::IntLit(n))
        }
        _ => Err(ts.unexpected("expression")),
    }
}
