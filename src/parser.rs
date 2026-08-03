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
    fn identdef(&mut self, what: &str) -> PResult<IdentDef> {
        let (name, pos) = self.expect_ident(what)?;
        let export = *self.peek() == Tok::Star;
        if export {
            self.advance();
        }
        Ok(IdentDef { name, pos, export })
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

        let (consts, vars) = self.const_var_declarations()?;
        let mut procs = Vec::new();
        while *self.peek() == Tok::Procedure {
            procs.push(self.proc_declaration()?);
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
            procs,
            body,
        })
    }

    // ImportList = IMPORT import {"," import} ";"
    // import = ident [":=" ident]   (alias := module)
    fn import_list(&mut self) -> PResult<Vec<Import>> {
        self.expect(Tok::Import, "'IMPORT'")?;
        let mut imports = Vec::new();
        loop {
            let (first, first_pos) = self.expect_ident("module name")?;
            let import = if *self.peek() == Tok::Assign {
                self.advance();
                let (name, pos) = self.expect_ident("module name")?;
                Import {
                    name,
                    pos,
                    qualifier: first,
                    qualifier_pos: first_pos,
                }
            } else {
                Import {
                    name: first.clone(),
                    pos: first_pos,
                    qualifier: first,
                    qualifier_pos: first_pos,
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
        let id = self.identdef("constant name")?;
        self.expect(Tok::Eq, "'='")?;
        let expr = self.expression()?;
        self.expect(Tok::Semi, "';'")?;
        Ok(ConstDecl { id, expr })
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

    // DeclarationSequence allows at most one CONST and one VAR section, in
    // that order. This accepts them repeated and interleaved; sema diagnoses
    // any redeclarations introduced by that leniency.
    fn const_var_declarations(&mut self) -> PResult<(Vec<ConstDecl>, Vec<VarDecl>)> {
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
                _ => break,
            }
        }
        Ok((consts, vars))
    }

    fn proc_declaration(&mut self) -> PResult<ProcDecl> {
        self.expect(Tok::Procedure, "'PROCEDURE'")?;
        let id = self.identdef("procedure name")?;
        let params = if *self.peek() == Tok::LParen {
            self.formal_parameters()?
        } else {
            Vec::new()
        };
        let ret = if *self.peek() == Tok::Colon {
            self.advance();
            Some(self.designator()?)
        } else {
            None
        };
        self.expect(Tok::Semi, "';'")?;

        let (consts, vars) = self.const_var_declarations()?;
        let mut procs = Vec::new();
        while *self.peek() == Tok::Procedure {
            procs.push(self.proc_declaration()?);
        }

        let body = if *self.peek() == Tok::Begin {
            self.advance();
            self.stmt_seq()?
        } else {
            Vec::new()
        };
        let ret_val = if *self.peek() == Tok::Return {
            self.advance();
            Some(self.expression()?)
        } else {
            None
        };

        self.expect(Tok::End, "'END'")?;
        let (end_name, end_pos) = self.expect_ident("procedure name after END")?;
        if end_name != id.name {
            return Err(Diagnostic::new(
                end_pos,
                format!("procedure is '{}' but END says '{end_name}'", id.name),
            ));
        }
        self.expect(Tok::Semi, "';'")?;

        Ok(ProcDecl {
            id,
            params,
            ret,
            consts,
            vars,
            procs,
            body,
            ret_val,
        })
    }

    // FormalParameters = "(" [FPSection {";" FPSection}] ")"
    fn formal_parameters(&mut self) -> PResult<Vec<FpSection>> {
        self.expect(Tok::LParen, "'('")?;
        let mut sections = Vec::new();
        if *self.peek() != Tok::RParen {
            loop {
                let var = if *self.peek() == Tok::Var {
                    self.advance();
                    true
                } else {
                    false
                };
                let mut names = vec![self.expect_ident("parameter name")?];
                while *self.peek() == Tok::Comma {
                    self.advance();
                    names.push(self.expect_ident("parameter name")?);
                }
                self.expect(Tok::Colon, "':'")?;
                if *self.peek() == Tok::Array {
                    return self.unsupported("ARRAY OF formal types");
                }
                let ty = self.designator()?;
                sections.push(FpSection { var, names, ty });
                if *self.peek() != Tok::Semi {
                    break;
                }
                self.advance();
            }
        }
        self.expect(Tok::RParen, "')'")?;
        Ok(sections)
    }

    fn stmt_seq(&mut self) -> PResult<Vec<Stmt>> {
        let mut stmts = Vec::new();
        loop {
            match self.peek() {
                Tok::Ident(_) => stmts.push(self.statement()?),
                Tok::If => stmts.push(self.if_statement()?),
                Tok::While => stmts.push(self.while_statement()?),
                Tok::Repeat => stmts.push(self.repeat_statement()?),
                Tok::For => stmts.push(self.for_statement()?),
                Tok::Case => stmts.push(self.case_statement()?),
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

    fn if_statement(&mut self) -> PResult<Stmt> {
        self.expect(Tok::If, "'IF'")?;
        let cond = self.expression()?;
        self.expect(Tok::Then, "'THEN'")?;
        let then = self.stmt_seq()?;
        let mut elsifs = Vec::new();
        while *self.peek() == Tok::Elsif {
            self.advance();
            let cond = self.expression()?;
            self.expect(Tok::Then, "'THEN'")?;
            elsifs.push((cond, self.stmt_seq()?));
        }
        let els = if *self.peek() == Tok::Else {
            self.advance();
            Some(self.stmt_seq()?)
        } else {
            None
        };
        self.expect(Tok::End, "'END'")?;
        Ok(Stmt::If {
            cond,
            then,
            elsifs,
            els,
        })
    }

    fn while_statement(&mut self) -> PResult<Stmt> {
        self.expect(Tok::While, "'WHILE'")?;
        let cond = self.expression()?;
        self.expect(Tok::Do, "'DO'")?;
        let body = self.stmt_seq()?;
        let mut elsifs = Vec::new();
        while *self.peek() == Tok::Elsif {
            self.advance();
            let cond = self.expression()?;
            self.expect(Tok::Do, "'DO'")?;
            elsifs.push((cond, self.stmt_seq()?));
        }
        self.expect(Tok::End, "'END'")?;
        Ok(Stmt::While { cond, body, elsifs })
    }

    fn repeat_statement(&mut self) -> PResult<Stmt> {
        self.expect(Tok::Repeat, "'REPEAT'")?;
        let body = self.stmt_seq()?;
        self.expect(Tok::Until, "'UNTIL'")?;
        let cond = self.expression()?;
        Ok(Stmt::Repeat { body, cond })
    }

    // ForStatement = FOR ident ":=" expression TO expression
    //     [BY ConstExpression] DO StatementSequence END.
    fn for_statement(&mut self) -> PResult<Stmt> {
        self.expect(Tok::For, "'FOR'")?;
        let (ident, pos) = self.expect_ident("control variable")?;
        self.expect(Tok::Assign, "':='")?;
        let start = self.expression()?;
        self.expect(Tok::To, "'TO'")?;
        let limit = self.expression()?;
        let step = if *self.peek() == Tok::By {
            self.advance();
            Some(self.expression()?)
        } else {
            None
        };
        self.expect(Tok::Do, "'DO'")?;
        let body = self.stmt_seq()?;
        self.expect(Tok::End, "'END'")?;
        Ok(Stmt::For {
            var: Designator {
                ident,
                selectors: Vec::new(),
                pos,
            },
            start,
            limit,
            step,
            body,
        })
    }

    // CaseStatement = CASE expression OF case {"|" case} END.
    fn case_statement(&mut self) -> PResult<Stmt> {
        self.expect(Tok::Case, "'CASE'")?;
        let expr = self.expression()?;
        self.expect(Tok::Of, "'OF'")?;
        let mut arms = Vec::new();
        loop {
            if let Some(arm) = self.case_arm()? {
                arms.push(arm);
            }
            if *self.peek() == Tok::Bar {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(Tok::End, "'END'")?;
        Ok(Stmt::Case { expr, arms })
    }

    // case = [CaseLabelList ":" StatementSequence].
    // An alternative with no labels is legal and contributes no arm, so
    // `CASE k OF | 1: x := 1 END` parses. That is not the same as a labelled
    // arm whose statement sequence is empty, which does become an arm.
    fn case_arm(&mut self) -> PResult<Option<CaseArm>> {
        if matches!(self.peek(), Tok::Bar | Tok::End) {
            return Ok(None);
        }
        let mut labels = vec![self.label_range()?];
        while *self.peek() == Tok::Comma {
            self.advance();
            labels.push(self.label_range()?);
        }
        self.expect(Tok::Colon, "':'")?;
        let body = self.stmt_seq()?;
        Ok(Some(CaseArm { labels, body }))
    }

    // LabelRange = label [".." label].
    fn label_range(&mut self) -> PResult<LabelRange> {
        let low = self.label()?;
        let high = if *self.peek() == Tok::DotDot {
            self.advance();
            Some(self.label()?)
        } else {
            None
        };
        Ok(LabelRange { low, high })
    }

    // label = integer | string | qualident. Deliberately not `expression`:
    // oberonc accepts `Max - 1` here, but that is an extension to the
    // normative grammar and this compiler does not adopt it.
    fn label(&mut self) -> PResult<Expr> {
        match self.peek() {
            Tok::Int(value) => {
                let value = *value;
                let pos = self.pos();
                self.advance();
                Ok(Expr::Int { value, pos })
            }
            Tok::Ident(_) => Ok(Expr::Name(self.designator()?)),
            Tok::Str(_) | Tok::Char(_) => self.unsupported("string case labels"),
            other => Err(self.error(format!("expected case label, found {other:?}"))),
        }
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
                let args = self.actual_parameters()?;
                Ok(Stmt::Call { proc: d, args, pos })
            }
            _ => Ok(Stmt::Call {
                proc: d,
                args: Vec::new(),
                pos,
            }),
        }
    }

    fn actual_parameters(&mut self) -> PResult<Vec<Expr>> {
        self.expect(Tok::LParen, "'('")?;
        let mut args = Vec::new();
        if *self.peek() != Tok::RParen {
            args.push(self.expression()?);
            while *self.peek() == Tok::Comma {
                self.advance();
                args.push(self.expression()?);
            }
        }
        self.expect(Tok::RParen, "')'")?;
        Ok(args)
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
        let lhs = self.simple_expression()?;
        let op = match self.peek() {
            Tok::Eq => BinOp::Eq,
            Tok::Hash => BinOp::Ne,
            Tok::Lt => BinOp::Lt,
            Tok::Le => BinOp::Le,
            Tok::Gt => BinOp::Gt,
            Tok::Ge => BinOp::Ge,
            Tok::In | Tok::Is => return self.unsupported("IN and IS relations"),
            _ => return Ok(lhs),
        };
        let pos = self.pos();
        self.advance();
        let rhs = self.simple_expression()?;
        Ok(Expr::Binary {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            pos,
        })
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
                Tok::Or => BinOp::Or,
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
                Tok::Amp => BinOp::And,
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
            Tok::True | Tok::False => {
                let value = *self.peek() == Tok::True;
                let pos = self.pos();
                self.advance();
                Ok(Expr::Bool { value, pos })
            }
            Tok::Tilde => {
                let pos = self.pos();
                self.advance();
                Ok(Expr::Unary {
                    op: UnOp::Not,
                    expr: Box::new(self.factor()?),
                    pos,
                })
            }
            Tok::Ident(_) => {
                let d = self.designator()?;
                if *self.peek() == Tok::LParen {
                    let pos = d.pos;
                    return Ok(Expr::Call {
                        callee: d,
                        args: self.actual_parameters()?,
                        pos,
                    });
                }
                Ok(Expr::Name(d))
            }
            Tok::LParen => {
                self.advance();
                let e = self.expression()?;
                self.expect(Tok::RParen, "')'")?;
                Ok(e)
            }
            Tok::Real(_) | Tok::Char(_) | Tok::Str(_) | Tok::Nil | Tok::LBrace => {
                self.unsupported("other literal factors")
            }
            other => Err(self.error(format!("expected expression, found {other:?}"))),
        }
    }
}
