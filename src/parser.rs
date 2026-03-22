use logos::Logos;
use crate::ast::{Function, Instruction, Program};
use crate::lexer::Token;
use crate::error::{ParseError, ParseResult};

struct TokenStream {
    tokens: Vec<(Token, std::ops::Range<usize>)>,
    cursor: usize,
}

impl TokenStream {
    fn new(src: &str) -> ParseResult<Self> {
        let mut tokens = Vec::new();
        let mut lex = Token::lexer(src);
        while let Some(result) = lex.next() {
            let span = lex.span();
            match result {
                Ok(tok) => tokens.push((tok, span)),
                Err(_) => return Err(ParseError::LexError { offset: span.start }),
            }
        }
        Ok(Self { tokens, cursor: 0 })
    }
    fn is_at_end(&self) -> bool { self.cursor >= self.tokens.len() }

    fn offset(&self) -> usize {
        self.tokens.get(self.cursor)
            .map(|(_, s)| s.start)
            .unwrap_or(usize::MAX)
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.cursor).map(|(t, _)| t)
    }
    fn advance(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.cursor).map(|(t, _)| t);
        if !self.is_at_end() { self.cursor += 1; }
        tok
    }
    fn eat(&mut self, expected: &Token) -> bool {
        if self.peek() == Some(expected) { self.advance(); true } else { false }
    }

    fn expect(&mut self, expected: &Token, label: &'static str) -> ParseResult<()> {
        if self.eat(expected) { return Ok(()); }
        let got = self.peek().map(|t| format!("{t:?}")).unwrap_or("EOF".into());
        Err(ParseError::UnexpectedToken { got, expected: label, offset: self.offset() })
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
            let got = self.peek().map(|t| format!("{t:?}")).unwrap_or("EOF".into());
            ParseError::UnexpectedToken { got, expected: "identifier", offset: self.offset() }
        })
    }

    fn expect_int(&mut self) -> ParseResult<i16> {
        if let Some(Token::Int(_)) = self.peek() {
            if let Some(Token::Int(n)) = self.advance() { return Ok(*n); }
        }
        let got = self.peek().map(|t| format!("{t:?}")).unwrap_or("EOF".into());
        Err(ParseError::UnexpectedToken { got, expected: "integer", offset: self.offset() })
    }
}

// Entrypoint
pub fn parse(src: &str) -> ParseResult<Program> {
    let mut ts = TokenStream::new(src)?;
    let mut functions = Vec::new();
    while !ts.is_at_end() {
        functions.push(parse_function(&mut ts)?);
    }
    Ok(Program { functions })
}

// Grammar
fn parse_function(ts: &mut TokenStream) -> ParseResult<Function> {
    let private = ts.eat(&Token::Private);
    ts.expect(&Token::Fn, "`fn`")?;
    let name = ts.expect_ident()?;
    ts.expect(&Token::LParen, "`(`")?;
    let params = parse_param_list(ts)?;
    ts.expect(&Token::RParen, "`)`")?;
    ts.expect(&Token::Colon, "`:`")?;
    let body = parse_body(ts)?;
    Ok(Function { name, _params: params, private, body })
}

fn parse_param_list(ts: &mut TokenStream) -> ParseResult<Vec<String>> {
    let mut params = Vec::new();
    if let Some(first) = ts.eat_ident() {
        params.push(first);
        while ts.eat(&Token::Comma) {
            params.push(ts.expect_ident()?);
        }
    }
    Ok(params)
}

fn parse_body(ts: &mut TokenStream) -> ParseResult<Vec<Instruction>> {
    let mut body = Vec::new();
    loop {
        match ts.peek() {
            None | Some(Token::Fn) | Some(Token::Private) => break,
            _ => body.push(parse_instruction(ts)?),
        }
    }
    Ok(body)
}

fn parse_instruction(ts: &mut TokenStream) -> ParseResult<Instruction> {
    let offset = ts.offset();
    match ts.peek() {
        Some(Token::GrowStack) => { ts.advance(); Ok(Instruction::GrowStack(ts.expect_int()?)) }
        Some(Token::LoadArg)   => { ts.advance(); Ok(Instruction::LoadArg(ts.expect_int()?)) }
        Some(Token::Add)       => { ts.advance(); Ok(Instruction::Add) }
        Some(Token::Sub)       => { ts.advance(); Ok(Instruction::Sub) }
        Some(Token::Push)       => { ts.advance(); Ok(Instruction::Push(ts.expect_int()?)) }
        Some(Token::Call) => {ts.advance(); Ok(Instruction::Call(ts.expect_ident()?))}
        Some(Token::Retv)      => { ts.advance(); Ok(Instruction::Retv(ts.expect_int()?)) }
        Some(Token::Ret)      => { ts.advance(); Ok(Instruction::Ret(ts.expect_int()?)) }
        other => {
            let got = other.map(|t| format!("{t:?}")).unwrap_or("EOF".into());
            Err(ParseError::UnexpectedToken { got, expected: "instruction", offset })
        }
    }
}