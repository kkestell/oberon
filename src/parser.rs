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
        match self.peek() {
            Tok::Array => self.array_type(),
            Tok::Record => self.unsupported("RECORD types"),
            Tok::Pointer => self.unsupported("POINTER types"),
            Tok::Procedure => self.unsupported("PROCEDURE types"),
            _ => Ok(TypeExpr::Named(self.qualident("type name")?)),
        }
    }

    // ArrayType = ARRAY length {"," length} OF type
    fn array_type(&mut self) -> PResult<TypeExpr> {
        let pos = self.pos();
        self.expect(Tok::Array, "'ARRAY'")?;
        let mut lengths = vec![self.expression()?];
        while *self.peek() == Tok::Comma {
            self.advance();
            lengths.push(self.expression()?);
        }
        self.expect(Tok::Of, "'OF'")?;
        let elem = Box::new(self.source_type()?);
        Ok(TypeExpr::Array { lengths, elem, pos })
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
        let params = if *self.peek() == Tok::LParen {
            self.formal_parameters()?
        } else {
            Vec::new()
        };
        let ret = if *self.peek() == Tok::Colon {
            self.advance();
            Some(TypeExpr::Named(self.qualident("result type name")?))
        } else {
            None
        };
        self.expect(Tok::Semi, "';'")?;

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
                // FormalType = {ARRAY OF} qualident. The open-array prefix is
                // the one form of it that is still unsupported; a fixed array
                // formal is written as a qualident and sema rejects it.
                if *self.peek() == Tok::Array {
                    return self.unsupported("ARRAY OF formal types");
                }
                let ty = TypeExpr::Named(self.qualident("parameter type name")?);
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
            Tok::Str(_) => Ok(self.string()),
            other => Err(self.error(format!("expected case label, found {other:?}"))),
        }
    }

    // The current token is known to be a string; both source forms arrive
    // here as one token carrying bytes.
    fn string(&mut self) -> Expr {
        let pos = self.pos();
        let t = self.advance();
        let Tok::Str(bytes) = t.tok else {
            unreachable!()
        };
        Expr::Str { bytes, pos }
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
                // selector = "[" ExpList "]". The list stays one selector:
                // Report 8.1 makes a[i, j] mean a[i][j], and that expansion
                // belongs to sema along with the types it needs.
                Tok::LBrack => {
                    let bracket = self.pos();
                    self.advance();
                    let mut exprs = vec![self.expression()?];
                    while *self.peek() == Tok::Comma {
                        self.advance();
                        exprs.push(self.expression()?);
                    }
                    self.expect(Tok::RBrack, "']'")?;
                    selectors.push(Selector::Index(exprs, bracket));
                }
                Tok::Caret => return self.unsupported("dereference selectors"),
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
            Tok::In => BinOp::In,
            Tok::Is => return self.unsupported("IS relations"),
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
                Tok::Slash => BinOp::Slash,
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
            Tok::Real(value) => {
                let value = *value;
                let pos = self.pos();
                self.advance();
                Ok(Expr::Real { value, pos })
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
            Tok::LBrace => self.set(),
            Tok::Str(_) => Ok(self.string()),
            Tok::Nil => self.unsupported("other literal factors"),
            other => Err(self.error(format!("expected expression, found {other:?}"))),
        }
    }

    // set = "{" [element {"," element}] "}"
    // element = expression [".." expression]
    fn set(&mut self) -> PResult<Expr> {
        let pos = self.pos();
        self.expect(Tok::LBrace, "'{'")?;
        let mut elements = Vec::new();
        if *self.peek() != Tok::RBrace {
            loop {
                let low = self.expression()?;
                let high = if *self.peek() == Tok::DotDot {
                    self.advance();
                    Some(self.expression()?)
                } else {
                    None
                };
                elements.push(SetElement { low, high });
                if *self.peek() == Tok::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.expect(Tok::RBrace, "'}'")?;
        Ok(Expr::Set { elements, pos })
    }
}

// The type and selector grammar is where this slice adds syntax, so these
// tests check the shape the parser builds rather than going through a whole
// compilation. Everything else stays end to end in tests/corpus.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer;

    fn module(body: &str) -> Module {
        let mut diags = Vec::new();
        let toks = lexer::lex(&format!("MODULE M;\n{body}\nEND M.\n"), &mut diags);
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        parse(toks).expect("module parses")
    }

    fn error(body: &str) -> String {
        let mut diags = Vec::new();
        let toks = lexer::lex(&format!("MODULE M;\n{body}\nEND M.\n"), &mut diags);
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        parse(toks).expect_err("module does not parse").msg
    }

    // The shape of a source type, flattened so a test can state it in one
    // line: "ARRAY 2, 3 OF INTEGER" prints its lengths as written.
    fn shape(ty: &TypeExpr) -> String {
        match ty {
            TypeExpr::Named(d) => d.name(),
            TypeExpr::Array { lengths, elem, .. } => {
                let lengths: Vec<String> = lengths
                    .iter()
                    .map(|length| match length {
                        Expr::Int { value, .. } => value.to_string(),
                        Expr::Name(d) => d.name(),
                        _ => "?".into(),
                    })
                    .collect();
                format!("ARRAY {} OF {}", lengths.join(", "), shape(elem))
            }
        }
    }

    #[test]
    fn exported_type_alias_and_arrays() {
        let m = module("TYPE Row* = ARRAY 8 OF INTEGER; Alias = Row; Grid = ARRAY 2 OF Row;");
        let shapes: Vec<_> = m
            .types
            .iter()
            .map(|t| (t.id.name.as_str(), t.id.export, shape(&t.ty)))
            .collect();
        assert_eq!(
            shapes,
            vec![
                ("Row", true, "ARRAY 8 OF INTEGER".to_string()),
                ("Alias", false, "Row".to_string()),
                ("Grid", false, "ARRAY 2 OF Row".to_string()),
            ]
        );
    }

    #[test]
    fn inline_and_nested_array_variables() {
        let m = module("VAR a: ARRAY 4 OF INTEGER; b: ARRAY 2 OF ARRAY 3 OF REAL;");
        let shapes: Vec<_> = m.vars.iter().map(|v| shape(&v.ty)).collect();
        assert_eq!(
            shapes,
            vec![
                "ARRAY 4 OF INTEGER".to_string(),
                "ARRAY 2 OF ARRAY 3 OF REAL".to_string(),
            ]
        );
    }

    // Report 6.2: the comma list stays one constructor here. Sema is what
    // turns it into nested arrays, so the parser must not flatten it away.
    #[test]
    fn comma_dimensions_stay_one_constructor() {
        let m = module("VAR g: ARRAY 2, 3, N OF INTEGER;");
        assert_eq!(shape(&m.vars[0].ty), "ARRAY 2, 3, N OF INTEGER");
    }

    #[test]
    fn repeated_bracket_selectors() {
        let m = module("VAR x: INTEGER;\nBEGIN x := g[1][i, j].f[0]");
        let Stmt::Assign { rhs, .. } = &m.body[0] else {
            panic!("expected an assignment");
        };
        let Expr::Name(d) = rhs else {
            panic!("expected a designator");
        };
        assert_eq!(d.name(), "g[...][...].f[...]");
        let counts: Vec<usize> = d
            .selectors
            .iter()
            .filter_map(|s| match s {
                Selector::Index(exprs, _) => Some(exprs.len()),
                Selector::Field(..) => None,
            })
            .collect();
        assert_eq!(counts, vec![1, 2, 1]);
    }

    #[test]
    fn malformed_length() {
        assert_eq!(
            error("VAR a: ARRAY OF INTEGER;"),
            "expected expression, found Of"
        );
    }

    #[test]
    fn empty_index_list() {
        assert_eq!(
            error("VAR x: INTEGER;\nBEGIN x := a[]"),
            "expected expression, found RBrack"
        );
    }

    #[test]
    fn missing_of() {
        assert_eq!(
            error("VAR a: ARRAY 4 INTEGER;"),
            "expected 'OF', found Ident(\"INTEGER\")"
        );
    }

    #[test]
    fn missing_closing_bracket() {
        assert_eq!(
            error("VAR x: INTEGER;\nBEGIN x := a[1"),
            "expected ']', found End"
        );
    }

    #[test]
    fn declaration_sections_must_be_ordered() {
        assert_eq!(
            error("VAR a: INTEGER; TYPE T = ARRAY 1 OF INTEGER;"),
            "expected 'END', found Type"
        );
    }

    #[test]
    fn string_factor_and_constant() {
        let m = module("CONST Prompt = \"> \"; Quote = 22X;\nVAR ch: CHAR;\nBEGIN ch := \"A\"");
        let consts: Vec<_> = m
            .consts
            .iter()
            .map(|c| match &c.expr {
                Expr::Str { bytes, .. } => bytes.clone(),
                other => panic!("expected a string, found {other:?}"),
            })
            .collect();
        assert_eq!(consts, vec![b"> ".to_vec(), vec![0x22]]);
        let Stmt::Assign { rhs, .. } = &m.body[0] else {
            panic!("expected an assignment");
        };
        assert!(matches!(rhs, Expr::Str { bytes, .. } if bytes == b"A"));
    }

    // label = integer | string | qualident, and a range may run between two
    // single-character strings.
    #[test]
    fn string_case_labels() {
        let m = module("VAR ch: CHAR;\nBEGIN CASE ch OF \"a\" .. \"f\", 0X: | \"z\": END");
        let Stmt::Case { arms, .. } = &m.body[0] else {
            panic!("expected a case statement");
        };
        assert_eq!(arms.len(), 2);
        let range = &arms[0].labels[0];
        assert!(matches!(&range.low, Expr::Str { bytes, .. } if bytes == b"a"));
        assert!(matches!(range.high.as_ref().unwrap(), Expr::Str { bytes, .. } if bytes == b"f"));
        let single = &arms[0].labels[1];
        assert!(matches!(&single.low, Expr::Str { bytes, .. } if bytes == &[0u8]));
        assert!(single.high.is_none());
    }

    #[test]
    fn structured_types_still_unsupported() {
        assert_eq!(
            error("TYPE R = RECORD END;"),
            "not yet supported: RECORD types"
        );
        assert_eq!(
            error("TYPE P = POINTER TO R;"),
            "not yet supported: POINTER types"
        );
        assert_eq!(
            error("TYPE P = PROCEDURE (n: INTEGER);"),
            "not yet supported: PROCEDURE types"
        );
        assert_eq!(
            error("PROCEDURE P(a: ARRAY OF INTEGER); END P;"),
            "not yet supported: ARRAY OF formal types"
        );
    }
}
