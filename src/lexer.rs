use crate::diag::{Diagnostic, Pos};

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    // Literals
    Ident(String),
    Int(i64), // i64 so an oversized decimal literal lexes; sema range-checks to i32
    // Keywords
    Array,
    Begin,
    By,
    Case,
    Const,
    Div,
    Do,
    Else,
    Elsif,
    End,
    False,
    For,
    If,
    Import,
    In,
    Is,
    Mod,
    Module,
    Nil,
    Of,
    Or,
    Pointer,
    Procedure,
    Record,
    Repeat,
    Return,
    Then,
    To,
    True,
    Type,
    Until,
    Var,
    While,
    // Operators and delimiters
    Plus,
    Minus,
    Star,
    Slash,
    Tilde,
    Amp,
    Dot,
    DotDot,
    Comma,
    Semi,
    Colon,
    Assign,
    Bar,
    Caret,
    Eq,
    Hash,
    Lt,
    Le,
    Gt,
    Ge,
    LParen,
    RParen,
    LBrack,
    RBrack,
    LBrace,
    RBrace,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub tok: Tok,
    pub pos: Pos,
}

pub fn lex(src: &str, diags: &mut Vec<Diagnostic>) -> Vec<Token> {
    Lexer {
        chars: src.chars().collect(),
        i: 0,
        line: 1,
        col: 1,
    }
    .run(diags)
}

struct Lexer {
    chars: Vec<char>,
    i: usize,
    line: u32,
    col: u32,
}

impl Lexer {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.i).copied()
    }

    fn peek2(&self) -> Option<char> {
        self.chars.get(self.i + 1).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.i += 1;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(c)
    }

    fn pos(&self) -> Pos {
        Pos {
            line: self.line,
            col: self.col,
        }
    }

    fn run(mut self, diags: &mut Vec<Diagnostic>) -> Vec<Token> {
        let mut toks = Vec::new();
        loop {
            self.skip_ws_and_comments(diags);
            let pos = self.pos();
            let Some(c) = self.peek() else {
                toks.push(Token { tok: Tok::Eof, pos });
                return toks;
            };
            let tok = if c.is_ascii_alphabetic() {
                Some(self.ident_or_keyword())
            } else if c.is_ascii_digit() {
                self.number(pos, diags)
            } else {
                self.operator(pos, diags)
            };
            if let Some(tok) = tok {
                toks.push(Token { tok, pos });
            }
        }
    }

    fn skip_ws_and_comments(&mut self, diags: &mut Vec<Diagnostic>) {
        loop {
            while matches!(self.peek(), Some(c) if c.is_whitespace()) {
                self.bump();
            }
            if self.peek() == Some('(') && self.peek2() == Some('*') {
                let start = self.pos();
                self.bump();
                self.bump();
                let mut depth = 1; // (* comments nest *)
                while depth > 0 {
                    match (self.peek(), self.peek2()) {
                        (Some('('), Some('*')) => {
                            self.bump();
                            self.bump();
                            depth += 1;
                        }
                        (Some('*'), Some(')')) => {
                            self.bump();
                            self.bump();
                            depth -= 1;
                        }
                        (Some(_), _) => {
                            self.bump();
                        }
                        (None, _) => {
                            diags.push(Diagnostic::new(start, "unterminated comment"));
                            return;
                        }
                    }
                }
            } else {
                return;
            }
        }
    }

    fn ident_or_keyword(&mut self) -> Tok {
        let mut s = String::new();
        while matches!(self.peek(), Some(c) if c.is_ascii_alphanumeric()) {
            s.push(self.bump().unwrap());
        }
        match s.as_str() {
            "ARRAY" => Tok::Array,
            "BEGIN" => Tok::Begin,
            "BY" => Tok::By,
            "CASE" => Tok::Case,
            "CONST" => Tok::Const,
            "DIV" => Tok::Div,
            "DO" => Tok::Do,
            "ELSE" => Tok::Else,
            "ELSIF" => Tok::Elsif,
            "END" => Tok::End,
            "FALSE" => Tok::False,
            "FOR" => Tok::For,
            "IF" => Tok::If,
            "IMPORT" => Tok::Import,
            "IN" => Tok::In,
            "IS" => Tok::Is,
            "MOD" => Tok::Mod,
            "MODULE" => Tok::Module,
            "NIL" => Tok::Nil,
            "OF" => Tok::Of,
            "OR" => Tok::Or,
            "POINTER" => Tok::Pointer,
            "PROCEDURE" => Tok::Procedure,
            "RECORD" => Tok::Record,
            "REPEAT" => Tok::Repeat,
            "RETURN" => Tok::Return,
            "THEN" => Tok::Then,
            "TO" => Tok::To,
            "TRUE" => Tok::True,
            "TYPE" => Tok::Type,
            "UNTIL" => Tok::Until,
            "VAR" => Tok::Var,
            "WHILE" => Tok::While,
            _ => Tok::Ident(s),
        }
    }

    // number = digit {digit} | digit {hexDigit} "H"
    // Hex digits are uppercase only, and hex literals start with a decimal digit.
    fn number(&mut self, pos: Pos, diags: &mut Vec<Diagnostic>) -> Option<Tok> {
        let mut s = String::new();
        while matches!(self.peek(), Some(c) if c.is_ascii_digit() || ('A'..='F').contains(&c)) {
            s.push(self.bump().unwrap());
        }
        match self.peek() {
            Some('H') => {
                self.bump();
                // A hex literal is a 32-bit pattern, not a magnitude: 0FFFFFFFFH
                // is -1, not 4294967295. cf. ORS.Mod Number, which accumulates
                // into a 32-bit LONGINT and marks the loop "(*no overflow check*)".
                match u32::from_str_radix(&s, 16) {
                    Ok(v) => Some(Tok::Int(v as i32 as i64)),
                    Err(_) => {
                        diags.push(Diagnostic::new(
                            pos,
                            format!("hex literal '{s}H' too large"),
                        ));
                        None
                    }
                }
            }
            _ => {
                if !s.bytes().all(|b| b.is_ascii_digit()) {
                    diags.push(Diagnostic::new(
                        pos,
                        format!("malformed number '{s}' (missing H or X?)"),
                    ));
                    return None;
                }
                match s.parse::<i64>() {
                    Ok(v) => Some(Tok::Int(v)),
                    Err(_) => {
                        diags.push(Diagnostic::new(
                            pos,
                            format!("integer literal '{s}' too large"),
                        ));
                        None
                    }
                }
            }
        }
    }

    fn operator(&mut self, pos: Pos, diags: &mut Vec<Diagnostic>) -> Option<Tok> {
        let c = self.bump().unwrap();
        let tok = match c {
            '+' => Tok::Plus,
            '-' => Tok::Minus,
            '*' => Tok::Star,
            '/' => Tok::Slash,
            '~' => Tok::Tilde,
            '&' => Tok::Amp,
            ',' => Tok::Comma,
            ';' => Tok::Semi,
            '|' => Tok::Bar,
            '^' => Tok::Caret,
            '=' => Tok::Eq,
            '#' => Tok::Hash,
            '(' => Tok::LParen,
            ')' => Tok::RParen,
            '[' => Tok::LBrack,
            ']' => Tok::RBrack,
            '{' => Tok::LBrace,
            '}' => Tok::RBrace,
            ':' => {
                if self.peek() == Some('=') {
                    self.bump();
                    Tok::Assign
                } else {
                    Tok::Colon
                }
            }
            '.' => {
                if self.peek() == Some('.') {
                    self.bump();
                    Tok::DotDot
                } else {
                    Tok::Dot
                }
            }
            '<' => {
                if self.peek() == Some('=') {
                    self.bump();
                    Tok::Le
                } else {
                    Tok::Lt
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.bump();
                    Tok::Ge
                } else {
                    Tok::Gt
                }
            }
            _ => {
                diags.push(Diagnostic::new(pos, format!("illegal character '{c}'")));
                return None;
            }
        };
        Some(tok)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toks(src: &str) -> Vec<Tok> {
        let mut diags = Vec::new();
        let toks = lex(src, &mut diags);
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        toks.into_iter().map(|t| t.tok).collect()
    }

    #[test]
    fn integers() {
        assert_eq!(
            toks("42 0 0FFH 0H"),
            vec![
                Tok::Int(42),
                Tok::Int(0),
                Tok::Int(255),
                Tok::Int(0),
                Tok::Eof
            ]
        );
    }

    #[test]
    fn hex_literals_are_32_bit_patterns() {
        assert_eq!(
            toks("0FFFFFFFFH 080000000H 07FFFFFFFH"),
            vec![
                Tok::Int(-1),
                Tok::Int(-2147483648),
                Tok::Int(2147483647),
                Tok::Eof
            ]
        );
    }

    #[test]
    fn nested_comment() {
        assert_eq!(
            toks("a (* x (* y *) z *) b"),
            vec![Tok::Ident("a".into()), Tok::Ident("b".into()), Tok::Eof]
        );
    }

    #[test]
    fn keyword_vs_ident() {
        assert_eq!(
            toks("MODULE Module BEGIN begin"),
            vec![
                Tok::Module,
                Tok::Ident("Module".into()),
                Tok::Begin,
                Tok::Ident("begin".into()),
                Tok::Eof
            ]
        );
    }

    #[test]
    fn operators() {
        assert_eq!(
            toks(":= : <= < >= > . .."),
            vec![
                Tok::Assign,
                Tok::Colon,
                Tok::Le,
                Tok::Lt,
                Tok::Ge,
                Tok::Gt,
                Tok::Dot,
                Tok::DotDot,
                Tok::Eof
            ]
        );
    }
}
