use crate::ast::{Function, Instruction, Program, Statement};
use crate::lexer::Token;
use crate::error::{ParseError, ParseResult};

struct TokenStream {
    tokens: Vec<(Token, std::ops::Range<usize>)>,
    cursor: usize,
}

impl TokenStream {
    fn new(tokens: Vec<(Token, std::ops::Range<usize>)>) -> Self {
        Self { tokens, cursor: 0 }
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
    fn eat_label(&mut self) -> Option<String> {
        if let Some(Token::LabelDef(_)) = self.peek() {
            if let Some(Token::LabelDef(name)) = self.advance() {
                return Some(name.clone());
            }
        }
        None
    }

    fn expect_label(&mut self) -> ParseResult<String> {
        self.eat_label().ok_or_else(|| {
            let got = self.peek().map(|t| format!("{t:?}")).unwrap_or("EOF".into());
            ParseError::UnexpectedToken { got, expected: "label", offset: self.offset() }
        })
    }

    pub fn expect_colon(&mut self) -> ParseResult<()> {
        match self.peek() {
            Some(Token::Colon) => { self.advance(); Ok(()) }
            other => {
                let got = other.map(|t| format!("{t:?}")).unwrap_or("EOF".into());
                Err(ParseError::UnexpectedToken { got, expected: "':'", offset: self.offset() })
            }
        }
    }

    fn expect_int(&mut self) -> ParseResult<i16> {
        if let Some(Token::Int(_)) = self.peek() {
            if let Some(Token::Int(n)) = self.advance() { return Ok(*n); }
        }
        let got = self.peek().map(|t| format!("{t:?}")).unwrap_or("EOF".into());
        Err(ParseError::UnexpectedToken { got, expected: "integer", offset: self.offset() })
    }

    fn expect_int32(&mut self) -> ParseResult<i32> {
        if let Some(Token::Int32(_)) = self.peek() {
            if let Some(Token::Int32(n)) = self.advance() { return Ok(*n); }
        }
        let got = self.peek().map(|t| format!("{t:?}")).unwrap_or("EOF".into());
        Err(ParseError::UnexpectedToken { got, expected: "integer", offset: self.offset() })
    }

    fn expect_string(&mut self) -> ParseResult<String> {
        if let Some(Token::StringLiteral(_)) = self.peek() {
            if let Some(Token::StringLiteral(s)) = self.advance() {
                return Ok(s.clone());
            }
        }
        let got = self.peek().map(|t| format!("{t:?}")).unwrap_or("EOF".into());
        Err(ParseError::UnexpectedToken { got, expected: "string literal", offset: self.offset() })
    }
}

// Entrypoint
pub fn parse(src: Vec<(Token, std::ops::Range<usize>)>) -> ParseResult<Program> {
    let mut ts = TokenStream::new(src);
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

fn parse_body(ts: &mut TokenStream) -> ParseResult<Vec<Statement>> {
    let mut body = Vec::new();
    loop {
        match ts.peek() {
            None | Some(Token::Fn) | Some(Token::Private) => break,
            _ => body.push(parse_statement(ts)?),
        }
    }
    Ok(body)
}

fn parse_statement(ts: &mut TokenStream) -> ParseResult<Statement> {
    match ts.peek() {
        Some(Token::LabelDef(_)) => Ok(Statement::Label(ts.expect_label()?)),
        _ => Ok(Statement::Instruction(parse_instruction(ts)?)),
    }
}

fn parse_instruction(ts: &mut TokenStream) -> ParseResult<Instruction> {
    let offset = ts.offset();
    match ts.peek() {
        Some(Token::GrowStack) => { ts.advance(); Ok(Instruction::GrowStack(ts.expect_int()?)) }
        Some(Token::LoadArg)   => { ts.advance(); Ok(Instruction::LoadArg(ts.expect_int()?)) }
        Some(Token::StoreArg)   => { ts.advance(); Ok(Instruction::StoreArg(ts.expect_int()?)) }
        Some(Token::Add)       => { ts.advance(); Ok(Instruction::Add) }
        Some(Token::Sub)       => { ts.advance(); Ok(Instruction::Sub) }
        Some(Token::Push)       => { ts.advance(); Ok(Instruction::Push(ts.expect_int()?)) }
        Some(Token::PushImm)       => { ts.advance(); Ok(Instruction::PushImm(ts.expect_int32()?)) }
        Some(Token::PushResult)       => { ts.advance(); Ok(Instruction::PushResult) }
        Some(Token::Call) => {ts.advance(); Ok(Instruction::Call(ts.expect_ident()?))}
        Some(Token::Jmp) => {ts.advance(); Ok(Instruction::Jmp(ts.expect_ident()?))}
        Some(Token::Jnz) => {ts.advance(); Ok(Instruction::Jnz(ts.expect_ident()?))}
        Some(Token::JnzPause) => {ts.advance(); Ok(Instruction::JnzPause(ts.expect_ident()?))}
        Some(Token::JnzSet) => {ts.advance(); Ok(Instruction::JnzSet(ts.expect_ident()?))}
        Some(Token::JzSet) => {ts.advance(); Ok(Instruction::JzSet(ts.expect_ident()?))}
        Some(Token::JzPause) => {ts.advance(); Ok(Instruction::JzPause(ts.expect_ident()?))}
        Some(Token::Jz) => {ts.advance(); Ok(Instruction::Jz(ts.expect_ident()?))}
        Some(Token::Eq0) => {ts.advance(); Ok(Instruction::Eq0)}
        Some(Token::Eq) => {ts.advance(); Ok(Instruction::Eq)}
        Some(Token::Retv)      => { ts.advance(); Ok(Instruction::Retv(ts.expect_int()?)) }
        Some(Token::Ret)      => { ts.advance(); Ok(Instruction::Ret(ts.expect_int()?)) }
        Some(Token::LStr) => { ts.advance(); Ok(Instruction::LStr(ts.expect_string()?)) }
        Some(Token::DelayLoad) => { ts.advance(); Ok(Instruction::DelayLoad) }
        Some(Token::DelayNeq0) => { ts.advance(); Ok(Instruction::DelayNeq0) }
        Some(Token::Exit1) => { ts.advance(); Ok(Instruction::Exit1) }
        Some(Token::Exit2) => { ts.advance(); Ok(Instruction::Exit2) }
        Some(Token::SetArgMode) => { ts.advance(); Ok(Instruction::SetArgMode) }
        Some(Token::SysCall(argc)) => {
            let argc = *argc;
            ts.advance();
            let page = ts.expect_int()? as u8;
            ts.expect_colon()?;
            let func = ts.expect_int()? as u8;
            Ok(Instruction::SysCall { argc, page, func })
        }
        other => {
            let got = other.map(|t| format!("{t:?}")).unwrap_or("EOF".into());
            Err(ParseError::UnexpectedToken { got, expected: "instruction", offset })
        }
    }
}