pub(crate) mod error;

use crate::ast::{BinOp, Expr, FuncDef, Item, Param, Stmt, Ty, UnaryOp};
use crate::lexer::token::{Span, Token, TokenKind};
use crate::parser::error::{ParseError, ParseResult};

pub struct TokenStream {
    tokens: Vec<Token>,
    cursor: usize,
    source: String,
}

impl TokenStream {
    pub(crate) fn new(tokens: Vec<Token>, source: String) -> Self {
        Self {
            tokens,
            cursor: 0,
            source,
        }
    }
    pub(crate) fn is_at_end(&self) -> bool {
        self.cursor >= self.tokens.len()
    }

    fn error_span(&self) -> Span {
        if let Some(t) = self.tokens.get(self.cursor) {
            t.span
        } else {
            let end = self.source.len();
            Span { start: end, end }
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
        Err(self.unexpected(label))
    }

    fn eat_ident(&mut self) -> Option<String> {
        if let Some(TokenKind::Ident(_)) = self.peek()
            && let Some(TokenKind::Ident(s)) = self.advance()
        {
            return Some(s.clone());
        }
        None
    }
    fn expect_ident(&mut self) -> ParseResult<String> {
        self.eat_ident()
            .ok_or_else(|| self.unexpected("identifier"))
    }
}

pub fn parse_item(ts: &mut TokenStream) -> ParseResult<Item> {
    match ts.peek() {
        Some(TokenKind::KwStatic | TokenKind::KwInt | TokenKind::KwVoid | TokenKind::KwBool) => {
            Ok(Item::FuncDef(parse_function(ts)?))
        }
        _ => Err(ts.unexpected("top-level item")),
    }
}

fn parse_function(ts: &mut TokenStream) -> ParseResult<FuncDef> {
    let exported = !ts.eat(&TokenKind::KwStatic); // static = private
    let ret_ty = parse_type_keyword(ts)?;
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
        exported,
    })
}

fn parse_type_keyword(ts: &mut TokenStream) -> ParseResult<Ty> {
    let token = ts.peek();
    match token {
        Some(TokenKind::KwInt) => {
            ts.expect(&TokenKind::KwInt, "int")?;
            Ok(Ty::Int)
        }
        Some(TokenKind::KwVoid) => {
            ts.expect(&TokenKind::KwVoid, "void")?;
            Ok(Ty::Void)
        }
        Some(TokenKind::KwBool) => {
            ts.expect(&TokenKind::KwBool, "bool")?;
            Ok(Ty::Bool)
        }
        _ => Err(ts.unexpected("type keyword")),
    }
}

fn parse_param_list(ts: &mut TokenStream) -> ParseResult<Vec<Param>> {
    let mut params: Vec<Param> = Vec::new();

    while !ts.eat(&TokenKind::RParen) {
        params.push(parse_param(ts)?);
        ts.eat(&TokenKind::Comma);
    }
    Ok(params)
}

fn parse_param(ts: &mut TokenStream) -> ParseResult<Param> {
    let ty = parse_type_keyword(ts)?;
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
    let stmt = match ts.peek() {
        Some(TokenKind::KwReturn) => parse_return(ts),
        Some(TokenKind::Ident(_)) => parse_assign_or_expr_stmt(ts),
        Some(TokenKind::KwInt | TokenKind::KwBool) => parse_var_decl(ts),
        _ => Err(ts.unexpected("statement")),
    };
    ts.expect(&TokenKind::Semicolon, "`;`")?;
    stmt
}
fn parse_var_decl(ts: &mut TokenStream) -> ParseResult<Stmt> {
    let ty = parse_type_keyword(ts)?;
    let name = ts.expect_ident()?;
    let init = if ts.eat(&TokenKind::Eq) {
        Some(parse_expr(ts, 0)?)
    } else {
        None
    };
    Ok(Stmt::VarDecl { name, ty, init })
}

fn parse_assign_or_expr_stmt(ts: &mut TokenStream) -> ParseResult<Stmt> {
    let name = ts.expect_ident()?;
    if ts.eat(&TokenKind::Eq) {
        let expr = parse_expr(ts, 0)?;
        Ok(Stmt::Assign { name, expr })
    } else {
        Err(ts.unexpected("assignment or expression statement"))
    }
}

fn parse_return(ts: &mut TokenStream) -> ParseResult<Stmt> {
    ts.expect(&TokenKind::KwReturn, "`return`")?;

    if ts.peek() == Some(&TokenKind::Semicolon) {
        return Ok(Stmt::Return(None));
    }

    let expr = parse_expr(ts, 0)?;
    Ok(Stmt::Return(Option::from(expr)))
}

fn parse_expr(ts: &mut TokenStream, min_bp: u8) -> ParseResult<Expr> {
    let mut lhs = match ts.peek() {
        Some(TokenKind::IntLit(_)) => parse_int_literal(ts),
        Some(TokenKind::BoolLit(_)) => parse_bool_literal(ts),
        Some(TokenKind::Ident(_)) => parse_identifier(ts),
        Some(TokenKind::LParen) => parse_group(ts),
        Some(TokenKind::Bang) => parse_unary(ts, UnaryOp::Not),
        Some(TokenKind::Minus) => parse_unary(ts, UnaryOp::Neg),
        _ => Err(ts.unexpected("expression")),
    }?;

    while let Some(tok) = ts.peek() {
        let Some((bp, op)) = binding_power(tok) else {
            break;
        };
        if bp < min_bp {
            break;
        }
        ts.advance();
        let rhs = parse_expr(ts, bp + 1)?;
        lhs = Expr::BinOp {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        };
    }

    Ok(lhs)
}

fn parse_unary(ts: &mut TokenStream, op: UnaryOp) -> ParseResult<Expr> {
    ts.advance();
    let expr = parse_expr(ts, 6)?;
    Ok(Expr::Unary {
        op,
        expr: Box::new(expr),
    })
}

fn parse_bool_literal(ts: &mut TokenStream) -> ParseResult<Expr> {
    match ts.advance() {
        Some(TokenKind::BoolLit(b)) => Ok(Expr::BoolLit(*b)),
        _ => Err(ts.unexpected("boolean literal")),
    }
}

fn binding_power(tok: &TokenKind) -> Option<(u8, BinOp)> {
    match tok {
        // logical
        TokenKind::PipePipe => Some((1, BinOp::Or)),
        TokenKind::AmpAmp => Some((2, BinOp::And)),
        // comparison
        TokenKind::EqEq => Some((3, BinOp::Eq)),
        TokenKind::BangEq => Some((3, BinOp::Neq)),
        TokenKind::Lt => Some((3, BinOp::Lt)),
        TokenKind::Gt => Some((3, BinOp::Gt)),
        TokenKind::LtEq => Some((3, BinOp::Le)),
        TokenKind::GtEq => Some((3, BinOp::Ge)),
        // arithmetic
        TokenKind::Plus => Some((4, BinOp::Add)),
        TokenKind::Minus => Some((4, BinOp::Sub)),
        TokenKind::Star => Some((5, BinOp::Mul)),
        TokenKind::Slash => Some((5, BinOp::Div)),
        _ => None,
    }
}

fn parse_int_literal(ts: &mut TokenStream) -> ParseResult<Expr> {
    match ts.advance() {
        Some(TokenKind::IntLit(n)) => Ok(Expr::IntLit(*n)),
        _ => Err(ts.unexpected("`integer literal`")),
    }
}
fn parse_group(ts: &mut TokenStream) -> ParseResult<Expr> {
    ts.advance();
    let expr = parse_expr(ts, 0)?;
    ts.expect(&TokenKind::RParen, "`)`")?;
    Ok(expr)
}

fn parse_identifier(ts: &mut TokenStream) -> ParseResult<Expr> {
    let name = ts.expect_ident()?;
    match ts.peek() {
        Some(TokenKind::LParen) => {
            todo!("function calls are not supported yet");
        }
        _ => Ok(Expr::Var(name)),
    }
}

#[cfg(test)]
mod parse_expr_tests {
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::expect_used)]
    #![allow(clippy::panic)]
    use super::*;
    use crate::ast::{BinOp, Expr};
    use crate::lexer::tokenize;

    fn parse(src: &str) -> Expr {
        let lex_output = tokenize(src);
        assert!(
            lex_output.errors.is_empty(),
            "lex errors: {:?}",
            lex_output.errors
        );
        parse_expr(&mut TokenStream::new(lex_output.tokens, src.to_string()), 0)
            .expect("parse error")
    }

    #[test]
    fn comparison_lower_precedence_than_arithmetic() {
        // 1 + 2 == 3  →  (1 + 2) == 3
        let e = parse("1 + 2 == 3");

        match e {
            Expr::BinOp {
                op: BinOp::Eq,
                lhs: left,
                rhs: right,
            } => {
                assert!(matches!(*right, Expr::IntLit(3)));

                match *left {
                    Expr::BinOp {
                        op: BinOp::Add,
                        lhs: l,
                        rhs: r,
                    } => {
                        assert!(matches!(*l, Expr::IntLit(1)));
                        assert!(matches!(*r, Expr::IntLit(2)));
                    }
                    _ => panic!("left side should be addition"),
                }
            }
            _ => panic!("expected top-level equality"),
        }
    }

    #[test]
    fn logical_and_lower_than_comparison() {
        // a == b && c == d  →  (a == b) && (c == d)
        let e = parse("a == b && c == d");

        match e {
            Expr::BinOp {
                op: BinOp::And,
                lhs: left,
                rhs: right,
            } => {
                // left: a == b
                match *left {
                    Expr::BinOp {
                        op: BinOp::Eq,
                        lhs: l,
                        rhs: r,
                    } => {
                        assert!(matches!(*l, Expr::Var(ref s) if s == "a"));
                        assert!(matches!(*r, Expr::Var(ref s) if s == "b"));
                    }
                    _ => panic!("left side should be equality"),
                }

                // right: c == d
                match *right {
                    Expr::BinOp {
                        op: BinOp::Eq,
                        lhs: l,
                        rhs: r,
                    } => {
                        assert!(matches!(*l, Expr::Var(ref s) if s == "c"));
                        assert!(matches!(*r, Expr::Var(ref s) if s == "d"));
                    }
                    _ => panic!("right side should be equality"),
                }
            }
            _ => panic!("expected top-level AND"),
        }
    }

    #[test]
    fn unary_not() {
        // !x
        let e = parse("!x");

        match e {
            Expr::Unary {
                op: UnaryOp::Not,
                expr,
            } => {
                assert!(matches!(*expr, Expr::Var(ref s) if s == "x"));
            }
            _ => panic!("expected unary NOT"),
        }
    }

    #[test]
    fn bool_literal_true() {
        assert_eq!(parse("true"), Expr::BoolLit(true));
    }

    #[test]
    fn bool_literal_false() {
        assert_eq!(parse("false"), Expr::BoolLit(false));
    }
    #[test]
    fn multiplication_binds_tighter_than_addition() {
        assert_eq!(
            parse("1 + 2 * 3"),
            Expr::BinOp {
                op: BinOp::Add,
                lhs: Box::new(Expr::IntLit(1)),
                rhs: Box::new(Expr::BinOp {
                    op: BinOp::Mul,
                    lhs: Box::new(Expr::IntLit(2)),
                    rhs: Box::new(Expr::IntLit(3)),
                }),
            }
        );
    }

    #[test]
    fn variable_in_expression() {
        assert_eq!(
            parse("x + 1"),
            Expr::BinOp {
                op: BinOp::Add,
                lhs: Box::new(Expr::Var("x".to_string())),
                rhs: Box::new(Expr::IntLit(1)),
            }
        );
    }

    #[test]
    fn chained_mixed_precedence() {
        assert_eq!(
            parse("1 + 2 * 3 + 4"),
            Expr::BinOp {
                op: BinOp::Add,
                lhs: Box::new(Expr::BinOp {
                    op: BinOp::Add,
                    lhs: Box::new(Expr::IntLit(1)),
                    rhs: Box::new(Expr::BinOp {
                        op: BinOp::Mul,
                        lhs: Box::new(Expr::IntLit(2)),
                        rhs: Box::new(Expr::IntLit(3)),
                    }),
                }),
                rhs: Box::new(Expr::IntLit(4)),
            }
        );
    }

    #[test]
    fn nested_groups() {
        assert_eq!(
            parse("((1 + 2)) * ((3 + 4))"),
            Expr::BinOp {
                op: BinOp::Mul,
                lhs: Box::new(Expr::BinOp {
                    op: BinOp::Add,
                    lhs: Box::new(Expr::IntLit(1)),
                    rhs: Box::new(Expr::IntLit(2)),
                }),
                rhs: Box::new(Expr::BinOp {
                    op: BinOp::Add,
                    lhs: Box::new(Expr::IntLit(3)),
                    rhs: Box::new(Expr::IntLit(4)),
                }),
            }
        );
    }

    #[test]
    fn group_in_middle_of_chain() {
        assert_eq!(
            parse("1 * (2 + 3) * 4"),
            Expr::BinOp {
                op: BinOp::Mul,
                lhs: Box::new(Expr::BinOp {
                    op: BinOp::Mul,
                    lhs: Box::new(Expr::IntLit(1)),
                    rhs: Box::new(Expr::BinOp {
                        op: BinOp::Add,
                        lhs: Box::new(Expr::IntLit(2)),
                        rhs: Box::new(Expr::IntLit(3)),
                    }),
                }),
                rhs: Box::new(Expr::IntLit(4)),
            }
        );
    }
}
