//! Parser: tokens -> syntax tree.
//!
//! Rules enforced here, before anything runs:
//!   1. `needs` (permission) lines may only appear at the top of a program.
//!   2. `retry` must have a fixed upper limit between 1 and MAX_RETRIES.
//!   3. `for` only loops over a finite list, so it always stops.

use crate::errors::LangError;
use crate::lexer::{Tok, Token};

const MAX_RETRIES: u32 = 10;

#[derive(Debug, Clone)]
pub enum Expr {
    Str(String),
    Num(f64),
    Var(String),
    List(Vec<Expr>),
    Call(String, Vec<Expr>),
    Eq(Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum StmtKind {
    Needs { action: String, target: String },
    Let(String, Expr),
    Do(Expr),
    Verify(Expr),
    Retry(u32, Vec<Stmt>),
    For(String, Expr, Vec<Stmt>),
}

#[derive(Debug, Clone)]
pub struct Stmt {
    pub line: usize,
    pub kind: StmtKind,
}

struct Parser {
    toks: Vec<Token>,
    pos: usize,
}

pub fn parse(toks: Vec<Token>) -> Result<Vec<Stmt>, LangError> {
    let mut p = Parser { toks, pos: 0 };
    let mut stmts = Vec::new();
    let mut seen_code = false;

    while !p.at_eof() {
        let stmt = p.statement()?;
        if matches!(stmt.kind, StmtKind::Needs { .. }) {
            if seen_code {
                return Err(LangError::new(
                    "needs_after_code",
                    Some(stmt.line),
                    "permissions must be declared before any other code",
                    "move all `needs ...` lines to the top of the program",
                ));
            }
        } else {
            seen_code = true;
        }
        stmts.push(stmt);
    }
    Ok(stmts)
}

impl Parser {
    fn peek(&self) -> &Tok {
        &self.toks[self.pos].tok
    }

    fn line(&self) -> usize {
        self.toks[self.pos].line
    }

    fn at_eof(&self) -> bool {
        *self.peek() == Tok::Eof
    }

    fn next(&mut self) -> Token {
        let t = self.toks[self.pos].clone();
        if self.pos < self.toks.len() - 1 {
            self.pos += 1;
        }
        t
    }

    fn expect(&mut self, want: Tok, what: &str) -> Result<(), LangError> {
        if *self.peek() == want {
            self.next();
            Ok(())
        } else {
            Err(LangError::new(
                "syntax_error",
                Some(self.line()),
                format!("expected {} but found {:?}", what, self.peek()),
                format!("write {} here", what),
            ))
        }
    }

    fn ident(&mut self, what: &str) -> Result<String, LangError> {
        let line = self.line();
        match self.next().tok {
            Tok::Ident(s) => Ok(s),
            other => Err(LangError::new(
                "syntax_error",
                Some(line),
                format!("expected {} but found {:?}", what, other),
                "check the spelling",
            )),
        }
    }

    fn statement(&mut self) -> Result<Stmt, LangError> {
        let line = self.line();
        let kind = match self.peek().clone() {
            Tok::Ident(word) if word == "needs" => {
                self.next();
                self.needs()?
            }
            Tok::Ident(word) if word == "let" => {
                self.next();
                self.let_stmt()?
            }
            Tok::Ident(word) if word == "verify" => {
                self.next();
                StmtKind::Verify(self.expr()?)
            }
            Tok::Ident(word) if word == "retry" => {
                self.next();
                self.retry()?
            }
            Tok::Ident(word) if word == "for" => {
                self.next();
                self.for_stmt()?
            }
            _ => StmtKind::Do(self.expr()?),
        };
        Ok(Stmt { line, kind })
    }

    fn needs(&mut self) -> Result<StmtKind, LangError> {
        let line = self.line();
        let action = self.ident("a permission name such as read or write")?;
        self.expect(Tok::LParen, "'('")?;
        let target = match self.next().tok {
            Tok::Str(s) => s,
            other => {
                return Err(LangError::new(
                    "syntax_error",
                    Some(line),
                    format!("a permission needs a quoted target, found {:?}", other),
                    "example: needs read(\"notes.txt\")",
                ))
            }
        };
        self.expect(Tok::RParen, "')'")?;
        if action != "read" && action != "write" && action != "fetch" {
            return Err(LangError::new(
                "unknown_permission",
                Some(line),
                format!("'{}' is not a known permission", action),
                "available permissions: read(\"path\"), write(\"path\"), fetch(\"domain\")",
            ));
        }
        Ok(StmtKind::Needs { action, target })
    }

    fn let_stmt(&mut self) -> Result<StmtKind, LangError> {
        let name = self.ident("a variable name")?;
        self.expect(Tok::Assign, "'='")?;
        let value = self.expr()?;
        Ok(StmtKind::Let(name, value))
    }

    /// Parses `{ statements }`. Permissions cannot be declared inside a block.
    fn block(&mut self, line: usize) -> Result<Vec<Stmt>, LangError> {
        self.expect(Tok::LBrace, "'{'")?;
        let mut body = Vec::new();
        while *self.peek() != Tok::RBrace {
            if self.at_eof() {
                return Err(LangError::new(
                    "syntax_error",
                    Some(line),
                    "this block is never closed",
                    "add a } at the end of the block",
                ));
            }
            let stmt = self.statement()?;
            if matches!(stmt.kind, StmtKind::Needs { .. }) {
                return Err(LangError::new(
                    "needs_in_block",
                    Some(stmt.line),
                    "permissions cannot be declared inside a block",
                    "move the `needs ...` line to the top of the program",
                ));
            }
            body.push(stmt);
        }
        self.expect(Tok::RBrace, "'}'")?;
        Ok(body)
    }

    fn retry(&mut self) -> Result<StmtKind, LangError> {
        let line = self.line();
        let times = match self.next().tok {
            Tok::Num(n) if n.fract() == 0.0 && n >= 1.0 && n <= MAX_RETRIES as f64 => n as u32,
            _ => {
                return Err(LangError::new(
                    "invalid_retry",
                    Some(line),
                    format!("retry needs a whole number between 1 and {}", MAX_RETRIES),
                    "every retry has a fixed upper limit, for example: retry 3 { ... }",
                ))
            }
        };
        let body = self.block(line)?;
        Ok(StmtKind::Retry(times, body))
    }

    fn for_stmt(&mut self) -> Result<StmtKind, LangError> {
        let line = self.line();
        let var = self.ident("a loop variable name")?;
        match self.next().tok {
            Tok::Ident(word) if word == "in" => {}
            other => {
                return Err(LangError::new(
                    "syntax_error",
                    Some(line),
                    format!("expected 'in' but found {:?}", other),
                    "write it like: for item in [\"a\", \"b\"] { ... }",
                ))
            }
        }
        let list = self.expr()?;
        let body = self.block(line)?;
        Ok(StmtKind::For(var, list, body))
    }

    fn expr(&mut self) -> Result<Expr, LangError> {
        let left = self.primary()?;
        if *self.peek() == Tok::EqEq {
            self.next();
            let right = self.primary()?;
            return Ok(Expr::Eq(Box::new(left), Box::new(right)));
        }
        Ok(left)
    }

    fn primary(&mut self) -> Result<Expr, LangError> {
        let line = self.line();
        match self.next().tok {
            Tok::Str(s) => Ok(Expr::Str(s)),
            Tok::Num(n) => Ok(Expr::Num(n)),
            Tok::LBracket => {
                let mut items = Vec::new();
                if *self.peek() != Tok::RBracket {
                    loop {
                        items.push(self.expr()?);
                        if *self.peek() == Tok::Comma {
                            self.next();
                        } else {
                            break;
                        }
                    }
                }
                self.expect(Tok::RBracket, "']'")?;
                Ok(Expr::List(items))
            }
            Tok::Ident(name) => {
                if *self.peek() == Tok::LParen {
                    self.next();
                    let mut args = Vec::new();
                    if *self.peek() != Tok::RParen {
                        loop {
                            args.push(self.expr()?);
                            if *self.peek() == Tok::Comma {
                                self.next();
                            } else {
                                break;
                            }
                        }
                    }
                    self.expect(Tok::RParen, "')'")?;
                    Ok(Expr::Call(name, args))
                } else {
                    Ok(Expr::Var(name))
                }
            }
            other => Err(LangError::new(
                "syntax_error",
                Some(line),
                format!("expected a value but found {:?}", other),
                "a value is a \"text\", a number, a [list], a variable, or a call like read(\"file.txt\")",
            )),
        }
    }
}
