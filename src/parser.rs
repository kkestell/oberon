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

    fn identdef(&mut self, what: &str) -> PResult<IdentDef> {
        let (name, pos) = self.expect_ident(what)?;
        Ok(IdentDef { name, pos })
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

        let (consts, types, vars) = self.declarations()?;
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
            types,
            vars,
            procs,
            body,
        })
    }

    // ImportList = IMPORT import {"," import} ";"
    fn import_list(&mut self) -> PResult<Vec<Import>> {
        self.expect(Tok::Import, "'IMPORT'")?;
        let mut imports = Vec::new();
        loop {
            let (name, pos) = self.expect_ident("module name")?;
            imports.push(Import { name, pos });
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

    // TypeDeclaration = identdef "=" StrucType. The right side is the general
    // `type` production rather than StrucType alone, so `Alias = Row;` parses:
    // the Report's own examples declare aliases, and all three reference
    // compilers accept a qualident here.
    fn type_decl(&mut self) -> PResult<TypeDecl> {
        let id = self.identdef("type name")?;
        self.expect(Tok::Eq, "'='")?;
        let ty = self.source_type()?;
        self.expect(Tok::Semi, "';'")?;
        Ok(TypeDecl { id, ty })
    }

    fn var_decl(&mut self) -> PResult<VarDecl> {
        let mut names = vec![self.identdef("variable name")?];
        while *self.peek() == Tok::Comma {
            self.advance();
            names.push(self.identdef("variable name")?);
        }
        self.expect(Tok::Colon, "':'")?;
        let ty = self.source_type()?;
        self.expect(Tok::Semi, "';'")?;
        Ok(VarDecl { names, ty })
    }

    // type = qualident | StrucType
    fn source_type(&mut self) -> PResult<TypeExpr> {
        Ok(TypeExpr::Named(self.qualident("type name")?))
    }

    // qualident = [ident "."] ident. A type name takes no further selectors,
    // so this is deliberately not `designator`.
    fn qualident(&mut self, what: &str) -> PResult<Designator> {
        let (ident, pos) = self.expect_ident(what)?;
        let mut selectors = Vec::new();
        if *self.peek() == Tok::Dot {
            self.advance();
            let (name, fpos) = self.expect_ident(what)?;
            selectors.push(Selector::Field(name, fpos));
        }
        Ok(Designator {
            ident,
            selectors,
            pos,
        })
    }

    // DeclarationSequence allows at most one CONST, one TYPE, and one VAR
    // section, in that order. Parsing them in three phases makes a repeated or
    // out-of-order keyword remain for the enclosing production to diagnose.
    fn declarations(&mut self) -> PResult<(Vec<ConstDecl>, Vec<TypeDecl>, Vec<VarDecl>)> {
        let mut consts = Vec::new();
        let mut types = Vec::new();
        let mut vars = Vec::new();
        if *self.peek() == Tok::Const {
            self.advance();
            while matches!(self.peek(), Tok::Ident(_)) {
                consts.push(self.const_decl()?);
            }
        }
        if *self.peek() == Tok::Type {
            self.advance();
            while matches!(self.peek(), Tok::Ident(_)) {
                types.push(self.type_decl()?);
            }
        }
        if *self.peek() == Tok::Var {
            self.advance();
            while matches!(self.peek(), Tok::Ident(_)) {
                vars.push(self.var_decl()?);
            }
        }
        Ok((consts, types, vars))
    }

    fn proc_declaration(&mut self) -> PResult<ProcDecl> {
        self.expect(Tok::Procedure, "'PROCEDURE'")?;
        let id = self.identdef("procedure name")?;
        let (params, ret) = if *self.peek() == Tok::LParen {
            self.formal_parameters()?
        } else {
            (Vec::new(), None)
        };
        self.expect(Tok::Semi, "';'")?;

        let (consts, types, vars) = self.declarations()?;

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
            types,
            vars,
            body,
            ret_val,
        })
    }

    // FormalParameters = "(" [FPSection {";" FPSection}] ")" [":" qualident]
    fn formal_parameters(&mut self) -> PResult<(Vec<FpSection>, Option<TypeExpr>)> {
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
                let ty = TypeExpr::Named(self.qualident("parameter type name")?);
                sections.push(FpSection { var, names, ty });
                if *self.peek() != Tok::Semi {
                    break;
                }
                self.advance();
            }
        }
        self.expect(Tok::RParen, "')'")?;
        let ret = if *self.peek() == Tok::Colon {
            self.advance();
            Some(TypeExpr::Named(self.qualident("result type name")?))
        } else {
            None
        };
        Ok((sections, ret))
    }

    fn stmt_seq(&mut self) -> PResult<Vec<Stmt>> {
        let mut stmts = Vec::new();
        loop {
            match self.peek() {
                Tok::Ident(_) => stmts.push(self.statement()?),
                Tok::If => stmts.push(self.if_statement()?),
                Tok::While => stmts.push(self.while_statement()?),
                Tok::Repeat => stmts.push(self.repeat_statement()?),
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

    fn statement(&mut self) -> PResult<Stmt> {
        let pos = self.pos();
        let d = self.designator()?;
        match self.peek() {
            Tok::Assign => {
                self.advance();
                let rhs = self.expression()?;
                Ok(Stmt::Assign { lhs: d, rhs })
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

    // designator = qualident {selector}
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
            // Kept rather than discarded: Report 8.2.2 allows unary "+" on a
            // numeric operand only, so "+TRUE" and "+{}" have to be rejected.
            Tok::Plus => {
                let pos = self.pos();
                self.advance();
                Expr::Unary {
                    op: UnOp::Plus,
                    expr: Box::new(self.term()?),
                    pos,
                }
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
                    return Ok(Expr::Apply {
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
            other => Err(self.error(format!("expected expression, found {other:?}"))),
        }
    }
}
