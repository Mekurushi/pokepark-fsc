pub(crate) mod error;
use crate::ast::{BinOp, Expr, ExprKind, FuncDef, Item, Param, Stmt, Ty, UnaryOp};
use crate::ast::{NodeId, StmtKind};
use crate::lexer::token::{Span, Token, TokenKind};
use crate::parser::error::{ParseError, ParseResult};
//TODO: better way for groups e.g. is type check in top-level item check
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

#[derive(Default)]
pub struct NodeIdGen {
    next: u32,
}

impl NodeIdGen {
    pub fn new() -> Self {
        Self { next: 0 }
    }

    pub fn alloc(&mut self) -> NodeId {
        let id = NodeId(self.next);
        self.next += 1;
        id
    }
}

pub struct Parser {
    ts: TokenStream,
    ids: NodeIdGen,
}

impl Parser {
    pub fn new(ts: TokenStream) -> Self {
        Self {
            ts,
            ids: NodeIdGen::new(),
        }
    }
    pub fn parse_item(&mut self) -> ParseResult<Item> {
        match self.ts.peek() {
            Some(
                TokenKind::KwStatic
                | TokenKind::KwInt
                | TokenKind::KwVoid
                | TokenKind::KwBool
                | TokenKind::KwString,
            ) => Ok(Item::FuncDef(self.parse_function()?)),
            _ => Err(self.ts.unexpected("top-level item")),
        }
    }
    pub fn is_at_end(&self) -> bool {
        self.ts.is_at_end()
    }

    fn parse_function(&mut self) -> ParseResult<FuncDef> {
        let exported = !self.ts.eat(&TokenKind::KwStatic); // static = private
        let ret_ty = self.parse_type_keyword()?;
        // function name
        let name = self.ts.expect_ident()?;

        // parameters
        self.ts.expect(&TokenKind::LParen, "`(`")?;
        let params = self.parse_param_list()?;

        // body
        self.ts.expect(&TokenKind::LBrace, "`{`")?;
        let body = self.parse_body()?;
        Ok(FuncDef {
            id: self.ids.alloc(),
            name,
            params,
            ret_ty,
            body,
            exported,
        })
    }

    fn parse_type_keyword(&mut self) -> ParseResult<Ty> {
        let token = self.ts.peek();
        match token {
            Some(TokenKind::KwInt) => {
                self.ts.expect(&TokenKind::KwInt, "int")?;
                Ok(Ty::Int)
            }
            Some(TokenKind::KwVoid) => {
                self.ts.expect(&TokenKind::KwVoid, "void")?;
                Ok(Ty::Void)
            }
            Some(TokenKind::KwBool) => {
                self.ts.expect(&TokenKind::KwBool, "bool")?;
                Ok(Ty::Bool)
            }
            Some(TokenKind::KwString) => {
                self.ts.expect(&TokenKind::KwString, "string")?;
                Ok(Ty::Str)
            }
            _ => Err(self.ts.unexpected("type keyword")),
        }
    }

    fn parse_param_list(&mut self) -> ParseResult<Vec<Param>> {
        let mut params: Vec<Param> = Vec::new();

        while !self.ts.eat(&TokenKind::RParen) {
            params.push(self.parse_param()?);
            self.ts.eat(&TokenKind::Comma);
        }
        Ok(params)
    }

    fn parse_param(&mut self) -> ParseResult<Param> {
        let ty = self.parse_type_keyword()?;
        let name = self.ts.expect_ident()?;
        Ok(Param { name, ty })
    }

    fn parse_body(&mut self) -> ParseResult<Vec<Stmt>> {
        let mut body = Vec::new();

        while !self.ts.eat(&TokenKind::RBrace) {
            body.push(self.parse_stmt()?);
        }

        Ok(body)
    }

    fn parse_stmt(&mut self) -> ParseResult<Stmt> {
        match self.ts.peek() {
            Some(TokenKind::KwIf) => self.parse_if(),
            Some(TokenKind::KwWhile) => self.parse_while(),
            _ => {
                let stmt = self.parse_stmt_inner()?;
                self.ts.expect(&TokenKind::Semicolon, "`;`")?;
                Ok(stmt)
            }
        }
    }

    fn parse_stmt_inner(&mut self) -> ParseResult<Stmt> {
        match self.ts.peek() {
            Some(TokenKind::KwReturn) => self.parse_return(),
            Some(TokenKind::Ident(_)) => self.parse_assign_or_expr_stmt(),
            Some(TokenKind::KwSysCall) => Ok(Stmt::new(
                self.ids.alloc(),
                StmtKind::ExprStmt(self.parse_syscall()?),
            )),
            Some(TokenKind::KwInt | TokenKind::KwBool | TokenKind::KwString) => {
                self.parse_var_decl()
            }
            _ => Err(self.ts.unexpected("statement")),
        }
    }

    fn parse_if(&mut self) -> ParseResult<Stmt> {
        self.ts.expect(&TokenKind::KwIf, "`if`")?;
        self.ts.expect(&TokenKind::LParen, "`(`")?;
        let cond = self.parse_expr(0)?;
        self.ts.expect(&TokenKind::RParen, "`)`")?;

        let then_body = self.parse_block()?;

        let else_body = if self.ts.eat(&TokenKind::KwElse) {
            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(Stmt::new(
            self.ids.alloc(),
            StmtKind::If {
                cond,
                then_body,
                else_body,
            },
        ))
    }

    fn parse_while(&mut self) -> ParseResult<Stmt> {
        self.ts.expect(&TokenKind::KwWhile, "`while`")?;
        self.ts.expect(&TokenKind::LParen, "`(`")?;

        let cond = self.parse_expr(0)?;

        self.ts.expect(&TokenKind::RParen, "`)`")?;

        let body = self.parse_block()?;

        Ok(Stmt::new(self.ids.alloc(), StmtKind::While { cond, body }))
    }

    fn parse_block(&mut self) -> ParseResult<Vec<Stmt>> {
        self.ts.expect(&TokenKind::LBrace, "`{`")?;
        let mut stmts = Vec::new();
        while !self.ts.eat(&TokenKind::RBrace) {
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }

    fn parse_var_decl(&mut self) -> ParseResult<Stmt> {
        let ty = self.parse_type_keyword()?;
        let name = self.ts.expect_ident()?;
        let init = if self.ts.eat(&TokenKind::Eq) {
            Some(self.parse_expr(0)?)
        } else {
            None
        };
        Ok(Stmt::new(
            self.ids.alloc(),
            StmtKind::VarDecl { name, ty, init },
        ))
    }

    fn parse_assign_or_expr_stmt(&mut self) -> ParseResult<Stmt> {
        let name = self.ts.expect_ident()?;
        if self.ts.eat(&TokenKind::Eq) {
            let expr = self.parse_expr(0)?;
            Ok(Stmt::new(
                self.ids.alloc(),
                StmtKind::Assign {
                    target: Expr::new(self.ids.alloc(), ExprKind::Var(name)),
                    expr,
                },
            ))
        } else if self.ts.peek() == Some(&TokenKind::LParen) {
            let call = self.parse_call(name)?;
            Ok(Stmt::new(self.ids.alloc(), StmtKind::ExprStmt(call)))
        } else {
            Err(self.ts.unexpected("assignment or call statement"))
        }
    }

    fn parse_return(&mut self) -> ParseResult<Stmt> {
        self.ts.expect(&TokenKind::KwReturn, "`return`")?;

        if self.ts.peek() == Some(&TokenKind::Semicolon) {
            return Ok(Stmt::new(self.ids.alloc(), StmtKind::Return(None)));
        }

        let expr = self.parse_expr(0)?;
        Ok(Stmt::new(
            self.ids.alloc(),
            StmtKind::Return(Option::from(expr)),
        ))
    }

    fn parse_expr(&mut self, min_bp: u8) -> ParseResult<Expr> {
        let mut lhs = match self.ts.peek() {
            Some(TokenKind::IntLit(_)) => self.parse_int_literal(),
            Some(TokenKind::BoolLit(_)) => self.parse_bool_literal(),
            Some(TokenKind::StrLit(_)) => self.parse_string_literal(),
            Some(TokenKind::Ident(_)) => self.parse_identifier(),
            Some(TokenKind::LParen) => self.parse_group(),
            Some(TokenKind::Bang) => self.parse_unary(UnaryOp::Not),
            Some(TokenKind::Minus) => self.parse_unary(UnaryOp::Neg),
            Some(TokenKind::KwSysCall) => self.parse_syscall(),
            _ => Err(self.ts.unexpected("expression")),
        }?;

        while let Some(tok) = self.ts.peek() {
            let Some((bp, op)) = binding_power(tok) else {
                break;
            };
            if bp < min_bp {
                break;
            }
            self.ts.advance();
            let rhs = self.parse_expr(bp + 1)?;

            lhs = Expr::new(
                self.ids.alloc(),
                ExprKind::BinOp {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
            );
        }

        Ok(lhs)
    }
    //TODO: cleanup logic in parser general, so less duplications
    fn parse_syscall(&mut self) -> ParseResult<Expr> {
        self.ts.advance();
        self.ts.expect(&TokenKind::LParen, "`(`")?;
        let mut args = Vec::new();
        while !self.ts.eat(&TokenKind::RParen) {
            args.push(self.parse_expr(0)?);
            self.ts.eat(&TokenKind::Comma);
        }
        Ok(Expr::new(self.ids.alloc(), ExprKind::SysCall { args }))
    }

    fn parse_unary(&mut self, op: UnaryOp) -> ParseResult<Expr> {
        self.ts.advance();
        let expr = self.parse_expr(6)?;

        Ok(Expr::new(
            self.ids.alloc(),
            ExprKind::Unary {
                op,
                expr: Box::new(expr),
            },
        ))
    }

    fn parse_bool_literal(&mut self) -> ParseResult<Expr> {
        match self.ts.advance() {
            Some(TokenKind::BoolLit(b)) => Ok(Expr::new(self.ids.alloc(), ExprKind::BoolLit(*b))),
            _ => Err(self.ts.unexpected("boolean literal")),
        }
    }

    fn parse_int_literal(&mut self) -> ParseResult<Expr> {
        match self.ts.advance() {
            Some(TokenKind::IntLit(n)) => Ok(Expr::new(self.ids.alloc(), ExprKind::IntLit(*n))),
            _ => Err(self.ts.unexpected("`integer literal`")),
        }
    }
    fn parse_string_literal(&mut self) -> ParseResult<Expr> {
        match self.ts.advance() {
            Some(TokenKind::StrLit(n)) => {
                Ok(Expr::new(self.ids.alloc(), ExprKind::StringLit(n.clone())))
            }
            _ => Err(self.ts.unexpected("`string literal`")),
        }
    }
    fn parse_group(&mut self) -> ParseResult<Expr> {
        self.ts.advance();
        let expr = self.parse_expr(0)?;
        self.ts.expect(&TokenKind::RParen, "`)`")?;
        Ok(expr)
    }

    fn parse_identifier(&mut self) -> ParseResult<Expr> {
        let name = self.ts.expect_ident()?;
        match self.ts.peek() {
            Some(TokenKind::LParen) => self.parse_call(name),
            _ => Ok(Expr::new(self.ids.alloc(), ExprKind::Var(name))),
        }
    }

    fn parse_call(&mut self, name: String) -> ParseResult<Expr> {
        self.ts.advance();
        let args = self.parse_arg_list()?;
        Ok(Expr::new(
            self.ids.alloc(),
            ExprKind::Call { callee: name, args },
        ))
    }
    fn parse_arg_list(&mut self) -> ParseResult<Vec<Expr>> {
        let mut args = Vec::new();
        while !self.ts.eat(&TokenKind::RParen) {
            args.push(self.parse_expr(0)?);
            self.ts.eat(&TokenKind::Comma);
        }
        Ok(args)
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
        let mut parser = Parser::new(TokenStream::new(lex_output.tokens, src.to_string()));
        parser.parse_expr(0).expect("parse error")
    }

    #[test]
    fn comparison_lower_precedence_than_arithmetic() {
        // 1 + 2 == 3  →  (1 + 2) == 3
        let e = parse("1 + 2 == 3");

        match e.kind {
            ExprKind::BinOp {
                op: BinOp::Eq,
                lhs: left,
                rhs: right,
            } => {
                assert!(matches!(right.kind, ExprKind::IntLit(3)));

                match left.kind {
                    ExprKind::BinOp {
                        op: BinOp::Add,
                        lhs: l,
                        rhs: r,
                    } => {
                        assert!(matches!(l.kind, ExprKind::IntLit(1)));
                        assert!(matches!(r.kind, ExprKind::IntLit(2)));
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

        match e.kind {
            ExprKind::BinOp {
                op: BinOp::And,
                lhs: left,
                rhs: right,
            } => {
                // left: a == b
                match left.kind {
                    ExprKind::BinOp {
                        op: BinOp::Eq,
                        lhs: l,
                        rhs: r,
                    } => {
                        assert!(matches!(l.kind, ExprKind::Var(ref s) if s == "a"));
                        assert!(matches!(r.kind, ExprKind::Var(ref s) if s == "b"));
                    }
                    _ => panic!("left side should be equality"),
                }

                // right: c == d
                match right.kind {
                    ExprKind::BinOp {
                        op: BinOp::Eq,
                        lhs: l,
                        rhs: r,
                    } => {
                        assert!(matches!(l.kind, ExprKind::Var(ref s) if s == "c"));
                        assert!(matches!(r.kind, ExprKind::Var(ref s) if s == "d"));
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

        match e.kind {
            ExprKind::Unary {
                op: UnaryOp::Not,
                expr,
            } => {
                assert!(matches!(expr.kind, ExprKind::Var(ref s) if s == "x"));
            }
            _ => panic!("expected unary NOT"),
        }
    }

    #[test]
    fn bool_literal_true() {
        assert_eq!(parse("true").kind, ExprKind::BoolLit(true));
    }

    #[test]
    fn bool_literal_false() {
        assert_eq!(parse("false").kind, ExprKind::BoolLit(false));
    }
    #[test]
    fn multiplication_binds_tighter_than_addition() {
        let e = parse("1 + 2 * 3");

        match e.kind {
            ExprKind::BinOp {
                op: BinOp::Add,
                lhs: left,
                rhs: right,
            } => {
                assert!(matches!(left.kind, ExprKind::IntLit(1)));
                match right.kind {
                    ExprKind::BinOp {
                        op: BinOp::Mul,
                        lhs: l,
                        rhs: r,
                    } => {
                        assert!(matches!(l.kind, ExprKind::IntLit(2)));
                        assert!(matches!(r.kind, ExprKind::IntLit(3)));
                    }
                    _ => panic!("left side should be addition"),
                }
            }
            _ => panic!("expected unary NOT"),
        }
    }

    #[test]
    fn variable_in_expression() {
        let e = parse("x + 1");
        match e.kind {
            ExprKind::BinOp {
                op: BinOp::Add,
                lhs: left,
                rhs: right,
            } => {
                assert!(matches!(left.kind, ExprKind::Var(ref s) if s == "x"));
                assert!(matches!(right.kind, ExprKind::IntLit(1)));
            }
            _ => panic!("expected add"),
        }
    }

    #[test]
    fn chained_mixed_precedence() {
        let e = parse("1 + 2 * 3 + 4");

        match e.kind {
            ExprKind::BinOp {
                op: BinOp::Add,
                lhs,
                rhs,
            } => {
                match lhs.kind {
                    ExprKind::BinOp {
                        op: BinOp::Add,
                        lhs: l1,
                        rhs: r1,
                    } => {
                        assert!(matches!(l1.kind, ExprKind::IntLit(1)));

                        match r1.kind {
                            ExprKind::BinOp {
                                op: BinOp::Mul,
                                lhs: m1,
                                rhs: m2,
                            } => {
                                assert!(matches!(m1.kind, ExprKind::IntLit(2)));
                                assert!(matches!(m2.kind, ExprKind::IntLit(3)));
                            }
                            _ => panic!("expected multiplication in middle"),
                        }
                    }
                    _ => panic!("expected left nested addition"),
                }

                assert!(matches!(rhs.kind, ExprKind::IntLit(4)));
            }
            _ => panic!("expected top-level addition"),
        }
    }

    #[test]
    fn nested_groups() {
        let e = parse("((1 + 2)) * ((3 + 4))");

        match e.kind {
            ExprKind::BinOp {
                op: BinOp::Mul,
                lhs,
                rhs,
            } => {
                match lhs.kind {
                    ExprKind::BinOp {
                        op: BinOp::Add,
                        lhs: l1,
                        rhs: r1,
                    } => {
                        assert!(matches!(l1.kind, ExprKind::IntLit(1)));
                        assert!(matches!(r1.kind, ExprKind::IntLit(2)));
                    }
                    _ => panic!("expected left side to be addition"),
                }

                match rhs.kind {
                    ExprKind::BinOp {
                        op: BinOp::Add,
                        lhs: l2,
                        rhs: r2,
                    } => {
                        assert!(matches!(l2.kind, ExprKind::IntLit(3)));
                        assert!(matches!(r2.kind, ExprKind::IntLit(4)));
                    }
                    _ => panic!("expected right side to be addition"),
                }
            }
            _ => panic!("expected top-level multiplication"),
        }
    }

    #[test]
    fn group_in_middle_of_chain() {
        let e = parse("1 * (2 + 3) * 4");
        match e.kind {
            ExprKind::BinOp {
                op: BinOp::Mul,
                lhs,
                rhs,
            } => {
                match lhs.kind {
                    ExprKind::BinOp {
                        op: BinOp::Mul,
                        lhs: l1,
                        rhs: r1,
                    } => {
                        assert!(matches!(l1.kind, ExprKind::IntLit(1)));

                        match r1.kind {
                            ExprKind::BinOp {
                                op: BinOp::Add,
                                lhs: a1,
                                rhs: a2,
                            } => {
                                assert!(matches!(a1.kind, ExprKind::IntLit(2)));
                                assert!(matches!(a2.kind, ExprKind::IntLit(3)));
                            }
                            _ => panic!("expected (2 + 3) in middle"),
                        }
                    }
                    _ => panic!("expected left multiplication"),
                }

                assert!(matches!(rhs.kind, ExprKind::IntLit(4)));
            }
            _ => panic!("expected top-level multiplication"),
        }
    }
}
