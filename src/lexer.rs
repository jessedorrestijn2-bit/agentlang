//! Lexer: source text -> tokens. Whitespace and newlines are not significant,
//! `#` starts a comment that runs to the end of the line.

use crate::errors::LangError;

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    Ident(String),
    Str(String),
    Num(f64),
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Assign,
    EqEq,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub tok: Tok,
    pub line: usize,
}

pub fn lex(src: &str) -> Result<Vec<Token>, LangError> {
    let chars: Vec<char> = src.chars().collect();
    let mut i = 0;
    let mut line = 1;
    let mut out: Vec<Token> = Vec::new();

    while i < chars.len() {
        let c = chars[i];

        if c == '\n' {
            line += 1;
            i += 1;
        } else if c.is_whitespace() {
            i += 1;
        } else if c == '#' {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
        } else if c == '"' {
            i += 1;
            let mut s = String::new();
            loop {
                if i >= chars.len() || chars[i] == '\n' {
                    return Err(LangError::new(
                        "unterminated_string",
                        Some(line),
                        "a string is missing its closing quote",
                        "add a \" at the end of the string",
                    ));
                }
                let ch = chars[i];
                if ch == '"' {
                    i += 1;
                    break;
                }
                if ch == '\\' && i + 1 < chars.len() {
                    match chars[i + 1] {
                        'n' => s.push('\n'),
                        't' => s.push('\t'),
                        other => s.push(other),
                    }
                    i += 2;
                } else {
                    s.push(ch);
                    i += 1;
                }
            }
            out.push(Token { tok: Tok::Str(s), line });
        } else if c.is_ascii_digit() {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            let text: String = chars[start..i].iter().collect();
            match text.parse::<f64>() {
                Ok(n) => out.push(Token { tok: Tok::Num(n), line }),
                Err(_) => {
                    return Err(LangError::new(
                        "invalid_number",
                        Some(line),
                        format!("'{}' is not a valid number", text),
                        "use digits with at most one decimal point, like 3 or 2.5",
                    ))
                }
            }
        } else if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            out.push(Token { tok: Tok::Ident(word), line });
        } else {
            let tok = match c {
                '(' => Tok::LParen,
                ')' => Tok::RParen,
                '{' => Tok::LBrace,
                '}' => Tok::RBrace,
                '[' => Tok::LBracket,
                ']' => Tok::RBracket,
                ',' => Tok::Comma,
                '=' => {
                    if i + 1 < chars.len() && chars[i + 1] == '=' {
                        i += 1;
                        Tok::EqEq
                    } else {
                        Tok::Assign
                    }
                }
                _ => {
                    return Err(LangError::new(
                        "unexpected_character",
                        Some(line),
                        format!("unexpected character '{}'", c),
                        "remove it, or put it inside a \"string\"",
                    ))
                }
            };
            i += 1;
            out.push(Token { tok, line });
        }
    }

    out.push(Token { tok: Tok::Eof, line });
    Ok(out)
}
