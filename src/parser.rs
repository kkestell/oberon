use crate::ast::*;
use crate::diag::{Diagnostic, Pos};
use crate::lexer::{Tok, Token};

// Fail-fast on the first parse error. Multi-diagnostic collection happens in
// sema, which walks a valid AST; the parser has no such luxury.
pub fn parse(toks: Vec<Token>) -> Result<Module, Diagnostic> {
    Parser { toks, i: 0 }.module()
}

type PResult<T> = Result<T, Diagnostic>;

struct Parser {
    toks: Vec<Token>,
    i: usize,
}

impl Parser {
    fn peek(&self) -> &Tok {
        &self.toks[self.i].tok
    }

    fn pos(&self) -> Pos {
        self.toks[self.i].pos
    }

    fn advance(&mut self) -> Token {
        let t = self.toks[self.i].clone();
        if t.tok != Tok::Eof {
            self.i += 1;
        }
        t
    }

    fn error(&self, msg: impl Into<String>) -> Diagnostic {
        Diagnostic::new(self.pos(), msg)
    }

    // For constructs that are valid Oberon-07 but not implemented yet. A
    // `todo!()` here would panic on correct source and report a position
    // inside the parser instead of one inside the user's module.
    fn unsupported<T>(&self, what: &str) -> PResult<T> {
        Err(self.error(format!("not yet supported: {what}")))
    }

    fn expect(&mut self, tok: Tok, what: &str) -> PResult<Token> {
        if *self.peek() == tok {
            Ok(self.advance())
        } else {
            Err(self.error(format!("expected {what}, found {:?}", self.peek())))
        }
    }

    fn expect_ident(&mut self, what: &str) -> PResult<(String, Pos)> {
        match self.peek() {
            Tok::Ident(_) => {
                let t = self.advance();
                let Tok::Ident(name) = t.tok else {
                    unreachable!()
                };
                Ok((name, t.pos))
            }
            other => Err(self.error(format!("expected {what}, found {other:?}"))),
        }
    }

    // identdef = ident ["*"]
    fn identdef(&mut self, what: &str) -> PResult<(String, Pos)> {
        let id = self.expect_ident(what)?;
        if *self.peek() == Tok::Star {
            return self.unsupported("export marks");
        }
        Ok(id)
    }

    fn module(&mut self) -> PResult<Module> {
        self.expect(Tok::Module, "'MODULE'")?;
        let (name, pos) = self.expect_ident("module name")?;
        self.expect(Tok::Semi, "';'")?;

        let imports = if *self.peek() == Tok::Import {
            self.import_list()?
        } else {
            Vec::new()
        };

        // DeclarationSequence allows at most one CONST and one VAR section, in
        // that order. This loop accepts them repeated and interleaved; sema
        // catches the redeclarations that leniency could otherwise let through.
        let mut consts = Vec::new();
        let mut vars = Vec::new();
        loop {
            match self.peek() {
                Tok::Const => {
                    self.advance();
                    while matches!(self.peek(), Tok::Ident(_)) {
                        consts.push(self.const_decl()?);
                    }
                }
                Tok::Var => {
                    self.advance();
                    while matches!(self.peek(), Tok::Ident(_)) {
                        vars.push(self.var_decl()?);
                    }
                }
                Tok::Type => return self.unsupported("TYPE declarations"),
                Tok::Procedure => return self.unsupported("PROCEDURE declarations"),
                _ => break,
            }
        }

        let body = if *self.peek() == Tok::Begin {
            self.advance();
            self.stmt_seq()?
        } else {
            Vec::new()
        };

        self.expect(Tok::End, "'END'")?;
        let (end_name, end_pos) = self.expect_ident("module name after END")?;
        if end_name != name {
            return Err(Diagnostic::new(
                end_pos,
                format!("module is '{name}' but END says '{end_name}'"),
            ));
        }
        self.expect(Tok::Dot, "'.'")?;

        Ok(Module {
            name,
            pos,
            imports,
            consts,
            vars,
            body,
        })
    }

    // ImportList = IMPORT import {"," import} ";"
    // import = ident [":=" ident]   (alias := module)
    fn import_list(&mut self) -> PResult<Vec<Import>> {
        self.expect(Tok::Import, "'IMPORT'")?;
        let mut imports = Vec::new();
        loop {
            let (first, pos) = self.expect_ident("module name")?;
            let import = if *self.peek() == Tok::Assign {
                self.advance();
                let (name, _) = self.expect_ident("module name")?;
                Import {
                    name,
                    alias: Some(first),
                    pos,
                }
            } else {
                Import {
                    name: first,
                    alias: None,
                    pos,
                }
            };
            imports.push(import);
            if *self.peek() == Tok::Comma {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(Tok::Semi, "';'")?;
        Ok(imports)
    }

    fn const_decl(&mut self) -> PResult<ConstDecl> {
        let (name, pos) = self.identdef("constant name")?;
        self.expect(Tok::Eq, "'='")?;
        let expr = self.expression()?;
        self.expect(Tok::Semi, "';'")?;
        Ok(ConstDecl { name, pos, expr })
    }

    fn var_decl(&mut self) -> PResult<VarDecl> {
        let mut names = vec![self.identdef("variable name")?];
        while *self.peek() == Tok::Comma {
            self.advance();
            names.push(self.identdef("variable name")?);
        }
        self.expect(Tok::Colon, "':'")?;
        let ty = self.designator()?; // TODO: structured types
        self.expect(Tok::Semi, "';'")?;
        Ok(VarDecl { names, ty })
    }

    fn stmt_seq(&mut self) -> PResult<Vec<Stmt>> {
        let mut stmts = Vec::new();
        loop {
            match self.peek() {
                Tok::Ident(_) => stmts.push(self.statement()?),
                Tok::If | Tok::While | Tok::Repeat | Tok::For | Tok::Case => {
                    return self.unsupported("structured statements");
                }
                _ => {} // empty statement is legal
            }
            if *self.peek() == Tok::Semi {
                self.advance();
            } else {
                break;
            }
        }
        Ok(stmts)
    }

    fn statement(&mut self) -> PResult<Stmt> {
        let pos = self.pos();
        let d = self.designator()?;
        match self.peek() {
            Tok::Assign => {
                self.advance();
                let rhs = self.expression()?;
                Ok(Stmt::Assign { lhs: d, rhs, pos })
            }
            Tok::LParen => {
                self.advance();
                let mut args = Vec::new();
                if *self.peek() != Tok::RParen {
                    args.push(self.expression()?);
                    while *self.peek() == Tok::Comma {
                        self.advance();
                        args.push(self.expression()?);
                    }
                }
                self.expect(Tok::RParen, "')'")?;
                Ok(Stmt::Call { proc: d, args, pos })
            }
            _ => Ok(Stmt::Call {
                proc: d,
                args: Vec::new(),
                pos,
            }),
        }
    }

    fn designator(&mut self) -> PResult<Designator> {
        let (ident, pos) = self.expect_ident("identifier")?;
        let mut selectors = Vec::new();
        loop {
            match self.peek() {
                Tok::Dot => {
                    self.advance();
                    let (name, fpos) = self.expect_ident("field name")?;
                    selectors.push(Selector::Field(name, fpos));
                }
                Tok::LBrack | Tok::Caret => {
                    return self.unsupported("index and dereference selectors");
                }
                _ => break,
            }
        }
        Ok(Designator {
            ident,
            selectors,
            pos,
        })
    }

    // expression = SimpleExpression [relation SimpleExpression]
    fn expression(&mut self) -> PResult<Expr> {
        let e = self.simple_expression()?;
        match self.peek() {
            Tok::Eq | Tok::Hash | Tok::Lt | Tok::Le | Tok::Gt | Tok::Ge | Tok::In | Tok::Is => {
                self.unsupported("relations")
            }
            _ => Ok(e),
        }
    }

    // SimpleExpression = ["+" | "-"] term {AddOperator term}
    fn simple_expression(&mut self) -> PResult<Expr> {
        let mut e = match self.peek() {
            Tok::Plus => {
                self.advance(); // unary + is the identity
                self.term()?
            }
            Tok::Minus => {
                let pos = self.pos();
                self.advance();
                match self.term()? {
                    // -2147483648 has no positive counterpart, so fold the sign
                    // into the literal before sema range-checks it. Only a bare
                    // literal folds: -a*b still means -(a*b).
                    Expr::Int { value, .. } => Expr::Int { value: -value, pos },
                    term => Expr::Unary {
                        op: UnOp::Neg,
                        expr: Box::new(term),
                        pos,
                    },
                }
            }
            _ => self.term()?,
        };
        loop {
            let op = match self.peek() {
                Tok::Plus => BinOp::Add,
                Tok::Minus => BinOp::Sub,
                Tok::Or => return self.unsupported("OR"),
                _ => break,
            };
            let pos = self.pos();
            self.advance();
            let rhs = self.term()?;
            e = Expr::Binary {
                op,
                lhs: Box::new(e),
                rhs: Box::new(rhs),
                pos,
            };
        }
        Ok(e)
    }

    // term = factor {MulOperator factor}
    fn term(&mut self) -> PResult<Expr> {
        let mut e = self.factor()?;
        loop {
            let op = match self.peek() {
                Tok::Star => BinOp::Mul,
                Tok::Div => BinOp::Div,
                Tok::Mod => BinOp::Mod,
                Tok::Slash => return self.unsupported("real division"),
                Tok::Amp => return self.unsupported("'&'"),
                _ => break,
            };
            let pos = self.pos();
            self.advance();
            let rhs = self.factor()?;
            e = Expr::Binary {
                op,
                lhs: Box::new(e),
                rhs: Box::new(rhs),
                pos,
            };
        }
        Ok(e)
    }

    fn factor(&mut self) -> PResult<Expr> {
        match self.peek() {
            Tok::Int(value) => {
                let value = *value;
                let pos = self.pos();
                self.advance();
                Ok(Expr::Int { value, pos })
            }
            Tok::Ident(_) => {
                let d = self.designator()?;
                if *self.peek() == Tok::LParen {
                    return self.unsupported("function calls in expressions");
                }
                Ok(Expr::Name(d))
            }
            Tok::LParen => {
                self.advance();
                let e = self.expression()?;
                self.expect(Tok::RParen, "')'")?;
                Ok(e)
            }
            Tok::Real(_)
            | Tok::Char(_)
            | Tok::Str(_)
            | Tok::Nil
            | Tok::True
            | Tok::False
            | Tok::LBrace
            | Tok::Tilde => self.unsupported("non-INTEGER factors"),
            other => Err(self.error(format!("expected expression, found {other:?}"))),
        }
    }
}
