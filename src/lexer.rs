use crate::diag::{Diagnostic, Pos};

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    // Literals
    Ident(String),
    Int(i64),  // i64 so an oversized decimal literal lexes; sema range-checks to i32
    Real(f32), // REAL is IEEE 754 binary32, so the literal rounds once, here
    // Report 3 gives strings two forms, and both are strings: a quoted literal
    // is the bytes of its source text exactly as they appear in the file, and
    // 41X is "a single-character string specified by the ordinal number of the
    // character". The bytes are not decoded or validated, so a UTF-8 source
    // character is one CHAR per byte, and the 0X form can carry the one byte a
    // quoted literal cannot contain.
    Str(Vec<u8>),
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
            } else if c == '"' {
                self.string(pos, diags)
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

    // number = digit {digit} | digit {hexDigit} "H" | digit {hexDigit} "X"
    //        | digit {digit} "." {digit} [ScaleFactor]
    // Hex digits are uppercase only, and hex literals start with a decimal digit.
    fn number(&mut self, pos: Pos, diags: &mut Vec<Diagnostic>) -> Option<Tok> {
        let start = self.i;
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
            Some('X') => {
                self.bump();
                match u32::from_str_radix(&s, 16) {
                    Ok(v) if v <= 0xFF => Some(Tok::Str(vec![v as u8])),
                    _ => {
                        diags.push(Diagnostic::new(
                            pos,
                            format!("character code '{s}X' out of range"),
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
                // digit "." "." ends the integer and leaves DotDot for `1..2`
                if self.peek() == Some('.') && self.peek2() != Some('.') {
                    self.bump();
                    self.real(s, start, pos, diags)
                } else {
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
    }

    // Fraction digits and scale factor are both optional; normalize into a
    // form f32::from_str is guaranteed to accept.
    fn real(
        &mut self,
        int_part: String,
        start: usize,
        pos: Pos,
        diags: &mut Vec<Diagnostic>,
    ) -> Option<Tok> {
        let mut text = int_part;
        text.push('.');
        let mut any = false;
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            text.push(self.bump().unwrap());
            any = true;
        }
        if !any {
            text.push('0');
        }
        if self.peek() == Some('E') {
            self.bump();
            text.push('e');
            if matches!(self.peek(), Some('+') | Some('-')) {
                text.push(self.bump().unwrap());
            }
            let mut any = false;
            while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                text.push(self.bump().unwrap());
                any = true;
            }
            if !any {
                diags.push(Diagnostic::new(pos, "malformed scale factor"));
                return None;
            }
        }
        // Straight to binary32, so the value the AST carries is the one the
        // generated code uses: parsing to binary64 first would round twice.
        // A magnitude too large to represent becomes an infinity, which the
        // source has no way to mean; one too small rounds to zero, which is
        // the ordinary IEEE result and not an error. cf. Project Oberon's
        // ORS.Mod, which rejects an exponent above its range and returns zero
        // for one below it.
        let value: f32 = text.parse().expect("constructed a valid float literal");
        if !value.is_finite() {
            let raw: String = self.chars[start..self.i].iter().collect();
            diags.push(Diagnostic::new(
                pos,
                format!("real literal '{raw}' is too large"),
            ));
            return None;
        }
        Some(Tok::Real(value))
    }

    fn string(&mut self, pos: Pos, diags: &mut Vec<Diagnostic>) -> Option<Tok> {
        self.bump(); // opening quote; no escape sequences in Oberon
        let mut s = String::new();
        loop {
            match self.peek() {
                Some('"') => {
                    self.bump();
                    return Some(Tok::Str(s.into_bytes()));
                }
                Some('\n') | None => {
                    diags.push(Diagnostic::new(pos, "unterminated string"));
                    return None;
                }
                Some(c) => {
                    s.push(c);
                    self.bump();
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
    fn reals_and_scale_factors() {
        assert_eq!(
            toks("1.5 2.0E3 1.E-2 37.4E5 1.0E+3"),
            vec![
                Tok::Real(1.5),
                Tok::Real(2000.0),
                Tok::Real(0.01),
                Tok::Real(3740000.0),
                Tok::Real(1000.0),
                Tok::Eof
            ]
        );
    }

    // The whole token rounds once, so the largest accepted literal is the one
    // whose binary32 value is still finite.
    #[test]
    fn largest_real_is_accepted() {
        assert_eq!(toks("3.4E38"), vec![Tok::Real(3.4e38), Tok::Eof]);
    }

    #[test]
    fn overflowing_real_diagnoses() {
        let mut diags = Vec::new();
        lex("3.5E38", &mut diags);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].msg, "real literal '3.5E38' is too large");
    }

    // Underflow is the ordinary IEEE result, not an error.
    #[test]
    fn underflowing_real_is_zero() {
        assert_eq!(toks("1.0E-60"), vec![Tok::Real(0.0), Tok::Eof]);
    }

    #[test]
    fn missing_exponent_digit_diagnoses() {
        let mut diags = Vec::new();
        lex("1.0E", &mut diags);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].msg, "malformed scale factor");
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

    // Both source forms produce the one string token: 41X is a one-character
    // string, 0X is a one-character string holding the null character, and a
    // byte above 127 is an ordinary character with no meaning attached.
    #[test]
    fn ordinal_strings() {
        assert_eq!(
            toks("41X 0X 0FFX"),
            vec![
                Tok::Str(vec![0x41]),
                Tok::Str(vec![0]),
                Tok::Str(vec![0xFF]),
                Tok::Eof
            ]
        );
    }

    #[test]
    fn string_literal() {
        assert_eq!(
            toks("\"hello\" \"\""),
            vec![Tok::Str(b"hello".to_vec()), Tok::Str(Vec::new()), Tok::Eof]
        );
    }

    // A quoted literal is the bytes of its source text, so a multi-byte UTF-8
    // character is one CHAR per byte, not one CHAR per character.
    #[test]
    fn string_literal_keeps_source_bytes() {
        assert_eq!(toks("\"é\""), vec![Tok::Str(vec![0xC3, 0xA9]), Tok::Eof]);
    }

    #[test]
    fn ordinal_string_out_of_range_diagnoses() {
        let mut diags = Vec::new();
        lex("100X", &mut diags);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].msg, "character code '100X' out of range");
    }

    #[test]
    fn integer_range_disambiguation() {
        assert_eq!(
            toks("1..2"),
            vec![Tok::Int(1), Tok::DotDot, Tok::Int(2), Tok::Eof]
        );
        assert_eq!(toks("1.5"), vec![Tok::Real(1.5), Tok::Eof]);
    }

    #[test]
    fn nested_comment() {
        assert_eq!(
            toks("a (* x (* y *) z *) b"),
            vec![Tok::Ident("a".into()), Tok::Ident("b".into()), Tok::Eof]
        );
    }

    #[test]
    fn unterminated_string_diagnoses() {
        let mut diags = Vec::new();
        lex("\"abc", &mut diags);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].msg.contains("unterminated string"));
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
