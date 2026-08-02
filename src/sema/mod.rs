use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::ast;
use crate::diag::{Diagnostic, Pos};
use crate::ir;

mod constant;
mod symbols;
mod types;

use symbols::{ConstValue, Scope, Symbol, universe_scope};
pub use symbols::{Interface, out_interface};
use types::*;

// Where a variable's storage is and what type it has there. Report 8.1 builds
// this by applying selectors to a variable, and every writable context —
// assignment and a VAR actual — uses the one that comes out.
struct Place {
    addr: ir::Addr,
    ty: Type,
}

type ResolvedFormal = (bool, String, Pos, Type);
type ResolvedProcedure = (Option<Type>, Vec<ResolvedFormal>, Option<Type>);

// What the source of an assignment or a relation operand turned out to be.
enum Source {
    Value(ir::Value, Type),
}

impl Source {
    // The type a diagnostic reports for this source.
    fn ty(&self) -> Type {
        match self {
            Source::Value(_, ty) => ty.clone(),
        }
    }
}

// `resolved` maps the real name of every module this one imports to that
// module's interface. The driver has already compiled them, so a name missing
// here is a driver bug and not a source error.
pub fn analyze(
    module: &ast::Module,
    resolved: &HashMap<String, Interface>,
) -> Result<(ir::Module, Interface), Vec<Diagnostic>> {
    Analyzer::new(&module.name).module(module, resolved)
}

struct Analyzer {
    module: String,
    scopes: Vec<Scope>,
    diags: Vec<Diagnostic>,
    globals: Vec<ir::Global>,
    literals: Vec<ir::Literal>,
    // The literal holding the module's name, which every runtime check
    // reports alongside its position.
    site_module: String,
    procs: Vec<ir::Proc>,
    interface: Interface,
    current: Option<ProcBuilder>,
}

impl Analyzer {
    fn new(module: &str) -> Self {
        Self {
            module: module.into(),
            // Report 4: the predefined identifiers are not declared in any
            // module, so a module may declare its own INTEGER and shadow this
            // one rather than colliding with it. ORB.Init builds the same
            // universe scope and opens the module scope inside it.
            scopes: vec![universe_scope(), Scope::new()],
            diags: Vec::new(),
            globals: Vec::new(),
            literals: Vec::new(),
            site_module: String::new(),
            procs: Vec::new(),
            interface: Interface::default(),
            current: None,
        }
    }

    fn module(
        mut self,
        module: &ast::Module,
        resolved: &HashMap<String, Interface>,
    ) -> Result<(ir::Module, Interface), Vec<Diagnostic>> {
        let name = self.module.clone().into_bytes();
        let ir::Addr::Global(symbol) = self.literal(&name) else {
            unreachable!("a literal is a global")
        };
        self.site_module = symbol;
        self.imports(&module.imports, resolved);
        self.const_declarations(&module.consts);
        self.type_declarations(&module.types);
        self.global_declarations(&module.vars);

        for proc in &module.procs {
            self.procedure(proc);
        }

        self.current = Some(ProcBuilder::new(format!(".{}.init", self.module), None));
        self.lower_stmts(&module.body);
        self.emit(ir::Inst::Ret(None));
        self.procs.push(
            self.current
                .take()
                .expect("module initializer exists")
                .finish(),
        );

        if self.diags.is_empty() {
            Ok((
                ir::Module {
                    name: self.module,
                    globals: self.globals,
                    literals: self.literals,
                    procs: self.procs,
                },
                self.interface,
            ))
        } else {
            Err(self.diags)
        }
    }

    // Report 11: an import declares the module's name in the client's module
    // scope, and the symbols behind it come from the module's interface.
    fn imports(&mut self, imports: &[ast::Import], resolved: &HashMap<String, Interface>) {
        for import in imports {
            let interface = resolved
                .get(&import.name)
                .expect("the driver resolved every import");
            let members = interface
                .members
                .iter()
                .map(|(name, member)| (name.clone(), member.client_symbol()))
                .collect();
            self.declare(&import.name, import.pos, Symbol::Module(members));
        }
    }

    // Constants are folded in declaration order. The separate type walk is
    // needed because folding short-circuits while unreachable operands still
    // have to be well-typed.
    fn const_declarations(&mut self, declarations: &[ast::ConstDecl]) {
        for declaration in declarations {
            if self.check_const_expr(&declaration.expr).is_none() {
                continue;
            }
            match self.eval_const(&declaration.expr) {
                Ok(value) => {
                    self.declare(
                        &declaration.id.name,
                        declaration.id.pos,
                        Symbol::Const(value),
                    );
                }
                Err(diag) => self.diags.push(diag),
            }
        }
    }

    // Report 4: type declarations are analyzed in textual order, and the right
    // side is resolved before the name is declared. A declaration can
    // therefore use an earlier type or constant but not itself or a later one;
    // the pointer-specific forward reference arrives with pointers.
    fn type_declarations(&mut self, declarations: &[ast::TypeDecl]) {
        for declaration in declarations {
            let Some(ty) = self.resolve_type(&declaration.ty) else {
                continue;
            };
            self.declare(
                &declaration.id.name,
                declaration.id.pos,
                Symbol::TypeName(ty),
            );
        }
    }

    fn global_declarations(&mut self, declarations: &[ast::VarDecl]) {
        for declaration in declarations {
            let Some(ty) = self.resolve_type(&declaration.ty) else {
                continue;
            };
            for id in &declaration.names {
                let symbol = format!("{}.{}", self.module, id.name);
                let addr = ir::Addr::Global(symbol.clone());
                if self.declare(
                    &id.name,
                    id.pos,
                    Symbol::Var {
                        ty: ty.clone(),
                        addr,
                    },
                ) {
                    self.globals.push(ir::Global {
                        symbol,
                        ty: ty.storage(),
                    });
                }
            }
        }
    }

    fn procedure(&mut self, declaration: &ast::ProcDecl) {
        let (proc_ty, formals, ret) =
            self.resolve_procedure_signature(&declaration.params, declaration.ret.as_ref());
        let symbol = format!("{}.{}", self.module, declaration.id.name);
        if let Some(proc_ty) = proc_ty {
            self.declare(
                &declaration.id.name,
                declaration.id.pos,
                Symbol::Proc {
                    symbol: symbol.clone(),
                    ty: proc_ty,
                },
            );
        }

        self.scopes.push(Scope::new());
        self.current = Some(ProcBuilder::new(symbol.clone(), ret.clone()));

        let mut declared_formals = HashSet::new();
        for (var, name, pos, ty) in formals {
            if !declared_formals.insert(name.clone()) {
                continue;
            }
            let by_ref = var;
            let temp = self.builder().temp();
            self.builder().proc.params.push(ir::Param {
                temp,
                pass: if by_ref {
                    ir::ParamPass::Ref
                } else {
                    ir::ParamPass::Value(ty.ir())
                },
            });
            let addr = if by_ref {
                ir::Addr::Temp(temp)
            } else {
                ir::Addr::Slot(name.clone())
            };
            if self.declare(
                &name,
                pos,
                Symbol::Var {
                    ty: ty.clone(),
                    addr: addr.clone(),
                },
            ) && !by_ref
            {
                self.reserve_slot(&name, &ty);
                self.emit(ir::Inst::Store {
                    ty: ty.ir(),
                    val: ir::Value::Temp(temp),
                    addr,
                });
            }
        }

        self.const_declarations(&declaration.consts);
        self.type_declarations(&declaration.types);
        self.local_declarations(&declaration.vars);
        self.lower_stmts(&declaration.body);
        self.lower_return(declaration, ret);

        self.procs
            .push(self.current.take().expect("procedure exists").finish());
        self.scopes.pop().expect("procedure scope exists");
    }

    fn local_declarations(&mut self, declarations: &[ast::VarDecl]) {
        for declaration in declarations {
            let Some(ty) = self.resolve_type(&declaration.ty) else {
                continue;
            };
            for id in &declaration.names {
                let addr = ir::Addr::Slot(id.name.clone());
                if self.declare(
                    &id.name,
                    id.pos,
                    Symbol::Var {
                        ty: ty.clone(),
                        addr,
                    },
                ) {
                    self.reserve_slot(&id.name, &ty);
                }
            }
        }
    }

    // One local's storage.
    fn reserve_slot(&mut self, name: &str, ty: &Type) {
        self.builder()
            .proc
            .slots
            .push((name.to_string(), ty.storage()));
    }

    fn lower_return(&mut self, declaration: &ast::ProcDecl, ret: Option<Type>) {
        match (&declaration.ret, ret, &declaration.ret_val) {
            (Some(_), Some(expected), Some(expr)) => {
                // RETURN asks the same compatibility function as assignment.
                let value = self.lower_expr(expr);
                if let Some((value, found)) = value {
                    let value = match assign_kind(&expected, &found) {
                        Some(AssignKind::Store) => Some(value),
                        _ => {
                            self.diags.push(Diagnostic::new(
                                expr.pos(),
                                format!("RETURN expression has type {found}, expected {expected}"),
                            ));
                            Some(value)
                        }
                    };
                    self.emit(ir::Inst::Ret(value));
                } else {
                    self.emit(ir::Inst::Ret(None));
                }
            }
            (Some(_), Some(_), None) => {
                self.diags.push(Diagnostic::new(
                    declaration.id.pos,
                    format!(
                        "function procedure '{}' must end with RETURN",
                        declaration.id.name
                    ),
                ));
                self.emit(ir::Inst::Ret(None));
            }
            (Some(_), None, Some(expr)) => {
                let _ = self.lower_expr(expr);
                self.emit(ir::Inst::Ret(None));
            }
            (Some(_), None, None) => self.emit(ir::Inst::Ret(None)),
            (None, _, Some(expr)) => {
                let _ = self.lower_expr(expr);
                self.diags.push(Diagnostic::new(
                    expr.pos(),
                    format!(
                        "proper procedure '{}' cannot RETURN a value",
                        declaration.id.name
                    ),
                ));
                self.emit(ir::Inst::Ret(None));
            }
            (None, _, None) => self.emit(ir::Inst::Ret(None)),
        }
    }

    fn lower_stmts(&mut self, stmts: &[ast::Stmt]) {
        for stmt in stmts {
            self.lower_stmt(stmt);
        }
    }

    fn lower_stmt(&mut self, stmt: &ast::Stmt) {
        match stmt {
            ast::Stmt::Assign { lhs, rhs, .. } => self.lower_assign(lhs, rhs),
            ast::Stmt::Call { proc, args, pos } => {
                if let Some((_, Some(_))) = self.lower_call(proc, args, *pos) {
                    self.diags.push(Diagnostic::new(
                        *pos,
                        format!("function '{}' cannot be called as a statement", proc.name()),
                    ));
                }
            }
            ast::Stmt::If {
                cond,
                then,
                elsifs,
                els,
            } => self.lower_if(cond, then, elsifs, els.as_deref()),
            ast::Stmt::While { cond, body, elsifs } => self.lower_while(cond, body, elsifs),
            ast::Stmt::Repeat { body, cond } => self.lower_repeat(body, cond),
        }
    }

    // Report 9.1. A scalar assignment stores one value. assign_kind names the
    // outcomes, and the value parameter and the RETURN expression ask the same
    // function.
    fn lower_assign(&mut self, lhs: &ast::Designator, rhs: &ast::Expr) {
        // The destination designator is resolved first and the source second,
        // each exactly once.
        let target = self.assign_target(lhs);
        let source = self.lower_source(rhs);
        let (Some(target), Some(source)) = (target, source) else {
            return;
        };
        match (assign_kind(&target.ty, &source.ty()), source) {
            (Some(AssignKind::Store), Source::Value(value, _)) => {
                self.emit(ir::Inst::Store {
                    ty: target.ty.ir(),
                    val: value,
                    addr: target.addr,
                });
            }
            (None, source) => {
                self.diags
                    .push(assign_error(rhs.pos(), &target.ty, &source.ty()));
            }
        }
    }

    // One string literal's data object. A literal exists only when a string's
    // bytes have to exist at run time; a string that only ever folds emits
    // nothing. Literals are numbered within the module and never shared
    // between two occurrences of the same text, and the leading dot keeps the
    // symbol outside the space user declarations can name.
    fn literal(&mut self, bytes: &[u8]) -> ir::Addr {
        let symbol = format!(".{}.str{}", self.module, self.literals.len());
        self.literals.push(ir::Literal {
            symbol: symbol.clone(),
            bytes: bytes.to_vec(),
        });
        ir::Addr::Global(symbol)
    }

    // The right-hand side of an assignment or one operand of a relation,
    // evaluated exactly once.
    fn lower_source(&mut self, expr: &ast::Expr) -> Option<Source> {
        let ast::Expr::Name(designator) = expr else {
            let (value, ty) = self.lower_expr(expr)?;
            return Some(Source::Value(value, ty));
        };
        // A constant is not storage, so it is recognized before the designator
        // is resolved as a variable.
        match self.qualident(designator) {
            Ok((Symbol::Const(value), [])) => return Some(Source::Value(value.ir(), value.ty())),
            Ok(_) => {}
            Err(diag) => {
                self.diags.push(diag);
                return None;
            }
        }
        let place = self.place(
            designator,
            format!("'{}' cannot be used as a value", designator.name()),
        )?;
        let value = self.load(place.addr, place.ty.ir());
        Some(Source::Value(value, place.ty))
    }

    // A source where the context needs one scalar value.
    fn source_scalar(&mut self, source: Source) -> Option<(ir::Value, Type)> {
        match source {
            Source::Value(value, ty) => Some((value, ty)),
        }
    }

    fn lower_if(
        &mut self,
        cond: &ast::Expr,
        then: &[ast::Stmt],
        elsifs: &[(ast::Expr, Vec<ast::Stmt>)],
        els: Option<&[ast::Stmt]>,
    ) {
        let end = self.label("if.end");
        self.lower_guarded(cond, then, &end);
        for (cond, body) in elsifs {
            self.lower_guarded(cond, body, &end);
        }
        if let Some(body) = els {
            self.lower_stmts(body);
        }
        self.emit(ir::Inst::Jmp(end.clone()));
        self.emit(ir::Inst::Label(end));
    }

    fn lower_guarded(&mut self, cond: &ast::Expr, body: &[ast::Stmt], end: &str) {
        let value = self.lower_condition(cond);
        if let Some(value) = value {
            let yes = self.label("guard.yes");
            let no = self.label("guard.no");
            self.emit(ir::Inst::Br {
                cond: value,
                then: yes.clone(),
                els: no.clone(),
            });
            self.emit(ir::Inst::Label(yes));
            self.lower_stmts(body);
            self.emit(ir::Inst::Jmp(end.into()));
            self.emit(ir::Inst::Label(no));
        } else {
            self.lower_stmts(body);
        }
    }

    fn lower_while(
        &mut self,
        cond: &ast::Expr,
        body: &[ast::Stmt],
        elsifs: &[(ast::Expr, Vec<ast::Stmt>)],
    ) {
        let test = self.label("while.test");
        let end = self.label("while.end");
        self.emit(ir::Inst::Jmp(test.clone()));
        self.emit(ir::Inst::Label(test.clone()));

        self.lower_loop_guard(cond, body, &test);
        for (cond, body) in elsifs {
            self.lower_loop_guard(cond, body, &test);
        }
        self.emit(ir::Inst::Jmp(end.clone()));
        self.emit(ir::Inst::Label(end));
    }

    fn lower_loop_guard(&mut self, cond: &ast::Expr, body: &[ast::Stmt], test: &str) {
        let value = self.lower_condition(cond);
        if let Some(value) = value {
            let yes = self.label("while.body");
            let no = self.label("while.next");
            self.emit(ir::Inst::Br {
                cond: value,
                then: yes.clone(),
                els: no.clone(),
            });
            self.emit(ir::Inst::Label(yes));
            self.lower_stmts(body);
            self.emit(ir::Inst::Jmp(test.into()));
            self.emit(ir::Inst::Label(no));
        } else {
            self.lower_stmts(body);
        }
    }

    fn lower_repeat(&mut self, body: &[ast::Stmt], cond: &ast::Expr) {
        let body_label = self.label("repeat.body");
        let end = self.label("repeat.end");
        self.emit(ir::Inst::Jmp(body_label.clone()));
        self.emit(ir::Inst::Label(body_label.clone()));
        self.lower_stmts(body);
        if let Some(value) = self.lower_condition(cond) {
            self.emit(ir::Inst::Br {
                cond: value,
                then: end.clone(),
                els: body_label,
            });
        }
        self.emit(ir::Inst::Label(end));
    }

    fn lower_condition(&mut self, expr: &ast::Expr) -> Option<ir::Value> {
        let (value, ty) = self.lower_expr(expr)?;
        if ty == Type::Boolean {
            Some(value)
        } else {
            self.diags.push(Diagnostic::new(
                expr.pos(),
                format!("condition must be BOOLEAN, found {ty}"),
            ));
            None
        }
    }

    fn lower_expr(&mut self, expr: &ast::Expr) -> Option<(ir::Value, Type)> {
        match expr {
            ast::Expr::Int { value, pos } => match i32::try_from(*value) {
                Ok(value) => Some((ir::Value::Int(value), Type::Integer)),
                Err(_) => {
                    self.diags
                        .push(Diagnostic::new(*pos, "integer literal out of range"));
                    None
                }
            },
            ast::Expr::Bool { value, .. } => Some((ir::Value::Bool(*value), Type::Boolean)),
            ast::Expr::Name(_) => {
                let source = self.lower_source(expr)?;
                self.source_scalar(source)
            }
            ast::Expr::Apply { callee, args, pos } => match self.lower_call(callee, args, *pos) {
                Some((Some(value), Some(ty))) => Some((value, ty)),
                Some((None, None)) => {
                    self.diags.push(Diagnostic::new(
                        *pos,
                        format!("'{}' cannot be used as a value", callee.name()),
                    ));
                    None
                }
                Some(_) => unreachable!("call result and return type agree"),
                None => None,
            },
            ast::Expr::Unary { op, expr, pos } => {
                let (arg, found) = self.lower_expr(expr)?;
                match (op, &found) {
                    // Report 8.2.2: unary "+" is the identity on a numeric
                    // operand, so it needs no instruction of its own.
                    (ast::UnOp::Plus, Type::Integer) => Some((arg, found)),
                    // INTEGER negation is the word operation, which wraps at
                    // MIN(INTEGER).
                    (ast::UnOp::Neg, Type::Integer) => {
                        let dst = self.temp();
                        self.emit(ir::Inst::Un {
                            dst,
                            op: ir::UnOp::Neg,
                            ty: found.ir(),
                            arg,
                        });
                        Some((ir::Value::Temp(dst), found))
                    }
                    (ast::UnOp::Not, Type::Boolean) => {
                        let dst = self.temp();
                        self.emit(ir::Inst::Un {
                            dst,
                            op: ir::UnOp::Not,
                            ty: ir::Ty::Bool,
                            arg,
                        });
                        Some((ir::Value::Temp(dst), Type::Boolean))
                    }
                    _ => {
                        self.diags.push(unary_type_error(*pos, *op, &found));
                        None
                    }
                }
            }
            ast::Expr::Binary { op, lhs, rhs, pos } => match op {
                ast::BinOp::And | ast::BinOp::Or => self.lower_logical(*op, lhs, rhs, *pos),
                _ => self.lower_binary(*op, lhs, rhs, *pos),
            },
        }
    }

    fn lower_logical(
        &mut self,
        op: ast::BinOp,
        lhs: &ast::Expr,
        rhs: &ast::Expr,
        pos: Pos,
    ) -> Option<(ir::Value, Type)> {
        let lhs = self.lower_expr(lhs);
        let Some((lhs_value, lhs_ty)) = lhs else {
            let _ = self.lower_expr(rhs);
            return None;
        };

        let rhs_label = self.label("logic.rhs");
        let short_label = self.label("logic.short");
        let end = self.label("logic.end");
        let result = self.temp();
        let (then, els, short) = match op {
            ast::BinOp::And => (rhs_label.clone(), short_label.clone(), false),
            ast::BinOp::Or => (short_label.clone(), rhs_label.clone(), true),
            _ => unreachable!(),
        };
        self.emit(ir::Inst::Br {
            cond: lhs_value,
            then,
            els,
        });
        self.emit(ir::Inst::Label(short_label));
        self.emit(ir::Inst::Copy {
            dst: result,
            ty: ir::Ty::Bool,
            src: ir::Value::Bool(short),
        });
        self.emit(ir::Inst::Jmp(end.clone()));
        self.emit(ir::Inst::Label(rhs_label));
        let rhs = self.lower_expr(rhs);
        if let Some((value, _)) = &rhs {
            self.emit(ir::Inst::Copy {
                dst: result,
                ty: ir::Ty::Bool,
                src: value.clone(),
            });
        }
        self.emit(ir::Inst::Jmp(end.clone()));
        self.emit(ir::Inst::Label(end));

        let (_, rhs_ty) = rhs?;
        check_binary_types(
            pos,
            bin_op_name(op),
            Type::Boolean,
            lhs_ty,
            rhs_ty,
            Type::Boolean,
            &mut self.diags,
        )?;
        Some((ir::Value::Temp(result), Type::Boolean))
    }

    fn lower_binary(
        &mut self,
        op: ast::BinOp,
        lhs: &ast::Expr,
        rhs: &ast::Expr,
        pos: Pos,
    ) -> Option<(ir::Value, Type)> {
        use ast::BinOp;
        if let BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge = op {
            return self.lower_relation(op, lhs, rhs, pos);
        }
        let lhs = self.lower_expr(lhs);
        let rhs_value = self.lower_expr(rhs);
        let (Some((lhs, lhs_ty)), Some((rhs_ir, rhs_ty))) = (lhs, rhs_value) else {
            return None;
        };

        // The operand type selects the machine operation, and it is the
        // result type too: a relation never reaches here.
        let (operand_ty, result_ty) = match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul => {
                let ty = check_arith_types(pos, op, lhs_ty, rhs_ty, &mut self.diags)?;
                (ty.clone(), ty)
            }
            BinOp::Div | BinOp::Mod => {
                check_binary_types(
                    pos,
                    bin_op_name(op),
                    Type::Integer,
                    lhs_ty,
                    rhs_ty,
                    Type::Integer,
                    &mut self.diags,
                )?;
                (Type::Integer, Type::Integer)
            }
            BinOp::Eq
            | BinOp::Ne
            | BinOp::Lt
            | BinOp::Le
            | BinOp::Gt
            | BinOp::Ge
            | BinOp::And
            | BinOp::Or => unreachable!(),
        };

        let value = match op {
            BinOp::Div | BinOp::Mod => {
                if !matches!(rhs, ast::Expr::Int { value, .. } if *value != 0) {
                    self.div_zero_check(rhs_ir.clone(), pos);
                }
                let (rem, adjust) = self.floor_adjust(lhs.clone(), rhs_ir.clone());
                if op == BinOp::Mod {
                    let delta = self.bin(ir::BinOp::Mul, ir::Ty::Int, adjust, rhs_ir);
                    self.bin(ir::BinOp::Add, ir::Ty::Int, rem, delta)
                } else {
                    let quotient = self.bin(ir::BinOp::Div, ir::Ty::Int, lhs, rhs_ir);
                    self.bin(ir::BinOp::Sub, ir::Ty::Int, quotient, adjust)
                }
            }
            _ => self.bin(
                match op {
                    BinOp::Add => ir::BinOp::Add,
                    BinOp::Sub => ir::BinOp::Sub,
                    BinOp::Mul => ir::BinOp::Mul,
                    _ => unreachable!(),
                },
                operand_ty.ir(),
                lhs,
                rhs_ir,
            ),
        };
        Some((value, result_ty))
    }

    // Report 8.2.4. The relations order INTEGER, and compare BOOLEAN for
    // equality.
    fn lower_relation(
        &mut self,
        op: ast::BinOp,
        lhs: &ast::Expr,
        rhs: &ast::Expr,
        pos: Pos,
    ) -> Option<(ir::Value, Type)> {
        use ast::BinOp;
        // Both operands are resolved left to right and each exactly once.
        let lhs_src = self.lower_source(lhs);
        let rhs_src = self.lower_source(rhs);
        let (Some(lhs_src), Some(rhs_src)) = (lhs_src, rhs_src) else {
            return None;
        };

        let lhs_value = self.source_scalar(lhs_src);
        let rhs_value = self.source_scalar(rhs_src);
        let (Some((lhs_value, lhs_ty)), Some((rhs_value, rhs_ty))) = (lhs_value, rhs_value) else {
            return None;
        };
        let operand_ty = match op {
            BinOp::Eq | BinOp::Ne => {
                if lhs_ty == rhs_ty && lhs_ty.scalar().is_some() {
                    lhs_ty
                } else {
                    self.diags.push(Diagnostic::new(
                        pos,
                        format!(
                            "operator '{}' requires operands of the same type, found {lhs_ty} and {rhs_ty}",
                            bin_op_name(op)
                        ),
                    ));
                    return None;
                }
            }
            BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                check_order_types(pos, op, lhs_ty, rhs_ty, &mut self.diags)?
            }
            _ => unreachable!("only a relation reaches lower_relation"),
        };
        let value = self.bin(relation_ir(op), operand_ty.ir(), lhs_value, rhs_value);
        Some((value, Type::Boolean))
    }

    fn div_zero_check(&mut self, divisor: ir::Value, pos: Pos) {
        let zero = self.bin(ir::BinOp::Eq, ir::Ty::Int, divisor, ir::Value::Int(0));
        let trap = self.label("div.zero");
        let ok = self.label("div.ok");
        self.emit(ir::Inst::Br {
            cond: zero,
            then: trap.clone(),
            els: ok.clone(),
        });
        self.emit(ir::Inst::Label(trap));
        self.trap("oberon_div_by_zero", pos);
        self.emit(ir::Inst::Label(ok));
    }

    // Report 8.2.2 requires floored DIV and MOD; QBE's div and rem truncate
    // toward zero. Truncation is off by one step exactly when the remainder
    // is nonzero and its sign differs from the divisor's, so compute that
    // condition as a 0-or-1 value: MOD adds adjust*divisor to the remainder,
    // DIV subtracts adjust from the quotient. Every intermediate stays inside
    // INTEGER; the shorter ((x rem y) + y) rem y does not, and gave wrong
    // answers for divisors near MAX(INTEGER). Constant folding applies the
    // same adjustment and must stay identical.
    fn floor_adjust(&mut self, lhs: ir::Value, rhs: ir::Value) -> (ir::Value, ir::Value) {
        let rem = self.bin(ir::BinOp::Rem, ir::Ty::Int, lhs, rhs.clone());
        let nonzero = self.bin(ir::BinOp::Ne, ir::Ty::Int, rem.clone(), ir::Value::Int(0));
        let rem_neg = self.bin(ir::BinOp::Lt, ir::Ty::Int, rem.clone(), ir::Value::Int(0));
        let rhs_neg = self.bin(ir::BinOp::Lt, ir::Ty::Int, rhs, ir::Value::Int(0));
        let differ = self.bin(ir::BinOp::Ne, ir::Ty::Bool, rem_neg, rhs_neg);
        let adjust = self.bin(ir::BinOp::Mul, ir::Ty::Int, nonzero, differ);
        (rem, adjust)
    }

    fn lower_call(
        &mut self,
        callee: &ast::Designator,
        actuals: &[ast::Expr],
        pos: Pos,
    ) -> Option<(Option<ir::Value>, Option<Type>)> {
        let callable = match self.qualident(callee) {
            Ok((Symbol::Proc { symbol, ty }, [])) => Some((symbol, ty)),
            Ok(_) => {
                self.diags.push(Diagnostic::new(
                    pos,
                    format!("'{}' is not a procedure", callee.name()),
                ));
                return None;
            }
            Err(diag) => {
                self.diags.push(diag);
                None
            }
        };

        let Some((symbol, ty)) = callable else {
            for actual in actuals {
                let _ = self.lower_expr(actual);
            }
            return None;
        };
        let procedure = ty
            .procedure()
            .cloned()
            .expect("a callable carries a procedure type");

        let mut ok = true;
        if actuals.len() != procedure.params.len() {
            self.diags.push(Diagnostic::new(
                pos,
                format!(
                    "wrong number of arguments: expected {}, found {}",
                    procedure.params.len(),
                    actuals.len()
                ),
            ));
            ok = false;
        }

        let mut args = Vec::new();
        for (i, actual) in actuals.iter().enumerate() {
            let Some(formal) = procedure.params.get(i) else {
                let _ = self.lower_expr(actual);
                continue;
            };
            let var = formal.var;
            let expected = formal.ty.clone();
            // A VAR formal takes the address of its actual, so the actual must
            // be a designator of the identical type: Report 10.1's rule.
            if var {
                match self.ref_actual(actual, i + 1) {
                    Some(place) => {
                        if place.ty == expected {
                            args.push(ir::Arg::Ref(place.addr.clone()));
                        } else {
                            self.diags.push(argument_type_error(
                                actual.pos(),
                                i + 1,
                                &expected,
                                &place.ty,
                            ));
                            ok = false;
                        }
                    }
                    None => ok = false,
                }
            } else {
                // The value parameter asks the same compatibility function as
                // assignment.
                match self.lower_expr(actual) {
                    Some((value, found)) => match assign_kind(&expected, &found) {
                        Some(AssignKind::Store) => {
                            args.push(ir::Arg::Val(expected.ir(), value));
                        }
                        _ => {
                            self.diags.push(Diagnostic::new(
                                actual.pos(),
                                format!("argument {} has type {found}, expected {expected}", i + 1),
                            ));
                            ok = false;
                        }
                    },
                    None => ok = false,
                }
            }
        }

        if !ok {
            return None;
        }
        let dst = procedure.ret.as_ref().map(|ty| (self.temp(), ty.ir()));
        self.emit(ir::Inst::Call { dst, symbol, args });
        let ret = procedure.ret.clone();
        Some((dst.map(|(temp, _)| ir::Value::Temp(temp)), ret))
    }

    // The address an actual supplies to a reference formal.
    fn ref_actual(&mut self, actual: &ast::Expr, number: usize) -> Option<Place> {
        let place = match actual {
            ast::Expr::Name(designator) => {
                self.place(designator, format!("argument {number} must be a variable"))
            }
            _ => {
                let _ = self.lower_expr(actual);
                self.diags.push(Diagnostic::new(
                    actual.pos(),
                    format!("argument {number} must be a variable"),
                ));
                return None;
            }
        }?;
        Some(place)
    }

    // The non-emitting type walk used only to check a call in a required
    // constant context. Executable lowering still uses place(), which
    // evaluates and checks every selector.
    fn designator_type(&self, designator: &ast::Designator) -> Result<Type, Diagnostic> {
        let (symbol, rest) = self.qualident(designator)?;
        let ty = match symbol {
            Symbol::Var { ty, .. } => ty,
            Symbol::Proc { ty, .. } if rest.is_empty() => return Ok(ty),
            _ => {
                return Err(Diagnostic::new(
                    designator.pos,
                    format!("'{}' cannot be used as a value", designator.name()),
                ));
            }
        };
        for selector in rest {
            match selector {
                ast::Selector::Field(name, pos) => {
                    return Err(Diagnostic::new(
                        *pos,
                        format!("cannot select '{name}' from {ty}: only a record has fields"),
                    ));
                }
            }
        }
        Ok(ty)
    }

    // The storage a designator denotes, with every selector applied in source
    // order. `not_a_variable` is the caller's own wording for a designator
    // that names something else, because assignment, an argument, and an
    // expression each phrase that differently.
    fn place(&mut self, designator: &ast::Designator, not_a_variable: String) -> Option<Place> {
        let (symbol, rest) = match self.qualident(designator) {
            Ok(found) => found,
            Err(diag) => {
                self.diags.push(diag);
                return None;
            }
        };
        let Symbol::Var { ty, addr } = symbol else {
            self.diags
                .push(Diagnostic::new(designator.pos, not_a_variable));
            return None;
        };
        let mut place = Place { addr, ty };
        for selector in rest {
            match selector {
                // Report 8.1: if r designates a record, r.f denotes the field
                // f of r.
                ast::Selector::Field(name, pos) => {
                    place = self.field(place, name, *pos)?;
                }
            }
        }
        Some(place)
    }

    fn field(&mut self, base: Place, name: &str, pos: Pos) -> Option<Place> {
        self.diags.push(Diagnostic::new(
            pos,
            format!(
                "cannot select '{name}' from {}: only a record has fields",
                base.ty
            ),
        ));
        None
    }

    fn assign_target(&mut self, designator: &ast::Designator) -> Option<Place> {
        self.place(
            designator,
            format!("cannot assign to '{}'", designator.name()),
        )
    }

    fn resolve_type(&mut self, source: &ast::TypeExpr) -> Option<Type> {
        match source {
            ast::TypeExpr::Named(designator) => match self.resolve(designator) {
                Ok(Symbol::TypeName(ty)) => Some(ty),
                Ok(_) => {
                    self.diags.push(Diagnostic::new(
                        designator.pos,
                        format!("'{}' is not a type", designator.name()),
                    ));
                    None
                }
                Err(diag) => {
                    self.diags.push(diag);
                    None
                }
            },
        }
    }

    // Procedure declarations use this source-signature builder. Formal names
    // are checked here and discarded; the resulting type records only mode,
    // type, order, and result.
    fn resolve_procedure_signature(
        &mut self,
        sections: &[ast::FpSection],
        result: Option<&ast::TypeExpr>,
    ) -> ResolvedProcedure {
        let mut formals = Vec::new();
        let mut names = HashSet::new();
        let mut ok = true;
        for section in sections {
            let ty = self.resolve_type(&section.ty);
            if ty.is_none() {
                ok = false;
            }
            for (name, pos) in &section.names {
                if !names.insert(name.clone()) {
                    self.diags.push(Diagnostic::new(
                        *pos,
                        format!("'{name}' is already declared"),
                    ));
                    ok = false;
                }
                if let Some(ty) = &ty {
                    formals.push((section.var, name.clone(), *pos, ty.clone()));
                }
            }
        }

        let ret = match result {
            Some(source) => match self.resolve_type(source) {
                Some(ty) => Some(ty),
                None => {
                    ok = false;
                    None
                }
            },
            None => None,
        };

        let ty = ok.then(|| {
            Type::Procedure(Rc::new(ProcedureType {
                params: formals
                    .iter()
                    .map(|(var, _, _, ty)| ProcedureParam {
                        var: *var,
                        ty: ty.clone(),
                    })
                    .collect(),
                ret: ret.clone(),
            }))
        });
        (ty, formals, ret)
    }

    // A designator that names an object directly, with no selectors left over.
    // Every context that wants a name rather than a variable goes through
    // here: a type, a procedure to call, a constant.
    fn resolve(&self, designator: &ast::Designator) -> Result<Symbol, Diagnostic> {
        let (symbol, rest) = self.qualident(designator)?;
        match rest.first() {
            None => Ok(symbol),
            Some(selector) => Err(selector_error(designator, selector)),
        }
    }

    // qualident = [ident "."] ident, plus the selectors that still have to be
    // applied to what it names. The two are separated because only a variable
    // can carry selectors.
    fn qualident<'a>(
        &self,
        designator: &'a ast::Designator,
    ) -> Result<(Symbol, &'a [ast::Selector]), Diagnostic> {
        let symbol = self
            .scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(&designator.ident))
            .cloned()
            .ok_or_else(|| {
                Diagnostic::new(
                    designator.pos,
                    format!("undeclared identifier '{}'", designator.ident),
                )
            })?;
        // Report 11: the qualifier of an imported object is not a selector on
        // a value, so it is consumed here and only here. Everything after it
        // belongs to the object it named.
        if let Symbol::Module(members) = &symbol
            && let Some(ast::Selector::Field(name, pos)) = designator.selectors.first()
        {
            let member = members.get(name).cloned().ok_or_else(|| {
                Diagnostic::new(
                    *pos,
                    format!(
                        "'{}' is not declared in module '{}'",
                        name, designator.ident
                    ),
                )
            })?;
            return Ok((member, &designator.selectors[1..]));
        }
        Ok((symbol, &designator.selectors))
    }

    // Report 4 forbids duplicate declarations only within one scope. The
    // first declaration wins so later diagnostics still see a stable symbol.
    fn declare(&mut self, name: &str, pos: Pos, symbol: Symbol) -> bool {
        let scope = self.scopes.last_mut().expect("at least the module scope");
        if scope.contains_key(name) {
            self.diags.push(Diagnostic::new(
                pos,
                format!("'{name}' is already declared"),
            ));
            false
        } else {
            scope.insert(name.into(), symbol);
            true
        }
    }

    fn builder(&mut self) -> &mut ProcBuilder {
        self.current.as_mut().expect("lowering inside a procedure")
    }

    fn temp(&mut self) -> usize {
        self.builder().temp()
    }

    fn label(&mut self, kind: &str) -> String {
        self.builder().label(kind)
    }

    fn emit(&mut self, inst: ir::Inst) {
        self.builder().proc.insts.push(inst);
    }

    // `ty` is the operand type, which is what the emitter needs to choose the
    // instruction's class.
    fn bin(&mut self, op: ir::BinOp, ty: ir::Ty, lhs: ir::Value, rhs: ir::Value) -> ir::Value {
        let dst = self.temp();
        self.emit(ir::Inst::Bin {
            dst,
            op,
            ty,
            lhs,
            rhs,
        });
        ir::Value::Temp(dst)
    }

    fn load(&mut self, addr: ir::Addr, ty: ir::Ty) -> ir::Value {
        let dst = self.temp();
        self.emit(ir::Inst::Load { dst, ty, addr });
        ir::Value::Temp(dst)
    }

    // A trap that ends the process: the check that guards it has already
    // branched here, so nothing follows it.
    fn trap(&mut self, symbol: &str, pos: Pos) {
        let site = self.site(pos);
        self.emit(ir::Inst::Trap {
            symbol: symbol.into(),
            site,
        });
    }

    // What a runtime check reports when it fails: this module and `pos`.
    fn site(&self, pos: Pos) -> ir::Site {
        ir::Site {
            module: self.site_module.clone(),
            pos,
        }
    }
}

struct ProcBuilder {
    proc: ir::Proc,
    next_temp: usize,
    next_label: usize,
}

impl ProcBuilder {
    fn new(symbol: String, ret: Option<Type>) -> Self {
        Self {
            proc: ir::Proc {
                symbol,
                params: Vec::new(),
                ret: ret.as_ref().map(Type::ir),
                slots: Vec::new(),
                insts: Vec::new(),
            },
            next_temp: 0,
            next_label: 0,
        }
    }

    fn temp(&mut self) -> usize {
        let temp = self.next_temp;
        self.next_temp += 1;
        temp
    }

    fn label(&mut self, kind: &str) -> String {
        let label = format!(".{kind}{}", self.next_label);
        self.next_label += 1;
        label
    }

    fn finish(self) -> ir::Proc {
        self.proc
    }
}

fn relation_ir(op: ast::BinOp) -> ir::BinOp {
    match op {
        ast::BinOp::Eq => ir::BinOp::Eq,
        ast::BinOp::Ne => ir::BinOp::Ne,
        ast::BinOp::Lt => ir::BinOp::Lt,
        ast::BinOp::Le => ir::BinOp::Le,
        ast::BinOp::Gt => ir::BinOp::Gt,
        ast::BinOp::Ge => ir::BinOp::Ge,
        _ => unreachable!("not a relation"),
    }
}

// A selector on something that cannot carry one. Reaching this means the
// context wanted a name — a type, a procedure, a constant — so the message
// says the selector is out of place rather than guessing at the intent. A
// selector on a variable never comes here: that is a field, and the
// designator walks handle it.
fn selector_error(designator: &ast::Designator, selector: &ast::Selector) -> Diagnostic {
    match selector {
        ast::Selector::Field(name, pos) => Diagnostic::new(
            *pos,
            format!(
                "'{name}' cannot be selected from '{}' here",
                designator.ident
            ),
        ),
    }
}

// Report 9.1 requires the same type on both sides.
fn assign_error(pos: Pos, target: &Type, found: &Type) -> Diagnostic {
    Diagnostic::new(pos, format!("cannot assign {found} to {target}"))
}

fn argument_type_error(pos: Pos, number: usize, expected: &Type, found: &Type) -> Diagnostic {
    Diagnostic::new(
        pos,
        format!("argument {number} has type {found}, expected {expected}"),
    )
}

fn unary_type_error(pos: Pos, op: ast::UnOp, found: &Type) -> Diagnostic {
    let (name, expected) = match op {
        ast::UnOp::Plus => ("+", "INTEGER"),
        ast::UnOp::Neg => ("-", "INTEGER"),
        ast::UnOp::Not => ("~", "BOOLEAN"),
    };
    Diagnostic::new(
        pos,
        format!("operator '{name}' requires {expected}, found {found}"),
    )
}
