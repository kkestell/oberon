use std::collections::HashMap;
use std::fmt;

use crate::ast;
use crate::diag::{Diagnostic, Pos};
use crate::ir;

type Scope = HashMap<String, Symbol>;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Type {
    Integer,
    Boolean,
}

impl Type {
    fn ir(self) -> ir::Ty {
        match self {
            Type::Integer => ir::Ty::Int,
            Type::Boolean => ir::Ty::Bool,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Integer => write!(f, "INTEGER"),
            Type::Boolean => write!(f, "BOOLEAN"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ConstValue {
    Int(i32),
    Bool(bool),
}

impl ConstValue {
    fn ty(self) -> Type {
        match self {
            ConstValue::Int(_) => Type::Integer,
            ConstValue::Bool(_) => Type::Boolean,
        }
    }

    fn ir(self) -> ir::Value {
        match self {
            ConstValue::Int(v) => ir::Value::Int(v),
            ConstValue::Bool(v) => ir::Value::Bool(v),
        }
    }
}

#[derive(Debug, Clone)]
enum Symbol {
    Const(ConstValue),
    Var {
        ty: Type,
        addr: ir::Addr,
    },
    Proc {
        symbol: String,
        params: Vec<(bool, Type)>,
        ret: Option<Type>,
    },
    Module(HashMap<String, Symbol>),
    TypeName(Type),
}

pub fn analyze(module: &ast::Module) -> Result<ir::Program, Vec<Diagnostic>> {
    Analyzer::new(&module.name).module(module)
}

struct Analyzer {
    module: String,
    scopes: Vec<Scope>,
    diags: Vec<Diagnostic>,
    globals: Vec<ir::Global>,
    procs: Vec<ir::Proc>,
    current: Option<ProcBuilder>,
}

impl Analyzer {
    fn new(module: &str) -> Self {
        let mut module_scope = Scope::new();
        module_scope.insert("INTEGER".into(), Symbol::TypeName(Type::Integer));
        module_scope.insert("BOOLEAN".into(), Symbol::TypeName(Type::Boolean));
        Self {
            module: module.into(),
            scopes: vec![module_scope],
            diags: Vec::new(),
            globals: Vec::new(),
            procs: Vec::new(),
            current: None,
        }
    }

    fn module(mut self, module: &ast::Module) -> Result<ir::Program, Vec<Diagnostic>> {
        self.imports(&module.imports);
        self.const_declarations(&module.consts);
        self.global_declarations(&module.vars);

        let prefix = self.module.clone();
        for proc in &module.procs {
            self.procedure(proc, &prefix);
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
            Ok(ir::Program {
                module: self.module,
                globals: self.globals,
                procs: self.procs,
            })
        } else {
            Err(self.diags)
        }
    }

    fn imports(&mut self, imports: &[ast::Import]) {
        for import in imports {
            if import.name == "Out" {
                let visible = import.alias.clone().unwrap_or_else(|| import.name.clone());
                self.declare(&visible, import.pos, Symbol::Module(out_scope()));
            } else {
                self.diags.push(Diagnostic::new(
                    import.pos,
                    format!("unknown module '{}'", import.name),
                ));
            }
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
                    self.declare(&declaration.name, declaration.pos, Symbol::Const(value));
                }
                Err(diag) => self.diags.push(diag),
            }
        }
    }

    fn global_declarations(&mut self, declarations: &[ast::VarDecl]) {
        for declaration in declarations {
            let Some(ty) = self.resolve_type(&declaration.ty) else {
                continue;
            };
            for (name, pos) in &declaration.names {
                let symbol = format!("{}.{}", self.module, name);
                let addr = ir::Addr::Global(symbol.clone());
                if self.declare(name, *pos, Symbol::Var { ty, addr }) {
                    self.globals.push(ir::Global {
                        symbol,
                        ty: ty.ir(),
                    });
                }
            }
        }
    }

    fn procedure(&mut self, declaration: &ast::ProcDecl, prefix: &str) {
        let mut formals = Vec::new();
        let mut params_ok = true;
        for section in &declaration.params {
            let Some(ty) = self.resolve_type(&section.ty) else {
                params_ok = false;
                continue;
            };
            for (name, pos) in &section.names {
                formals.push((section.var, name.clone(), *pos, ty));
            }
        }

        let (ret, ret_ok) = match &declaration.ret {
            Some(designator) => match self.resolve_type(designator) {
                Some(ty) => (Some(ty), true),
                None => (None, false),
            },
            None => (None, true),
        };
        let symbol = format!("{prefix}.{}", declaration.name);
        if params_ok && ret_ok {
            self.declare(
                &declaration.name,
                declaration.pos,
                Symbol::Proc {
                    symbol: symbol.clone(),
                    params: formals.iter().map(|(var, _, _, ty)| (*var, *ty)).collect(),
                    ret,
                },
            );
        }

        self.scopes.push(Scope::new());
        let enclosing = self.current.take();
        self.current = Some(ProcBuilder::new(symbol.clone(), ret));

        for (var, name, pos, ty) in formals {
            let temp = self.builder().temp();
            self.builder().proc.params.push(ir::Param {
                temp,
                pass: if var {
                    ir::ParamPass::Ref
                } else {
                    ir::ParamPass::Value(ty.ir())
                },
            });
            let addr = if var {
                ir::Addr::Temp(temp)
            } else {
                ir::Addr::Slot(name.clone())
            };
            if self.declare(
                &name,
                pos,
                Symbol::Var {
                    ty,
                    addr: addr.clone(),
                },
            ) && !var
            {
                self.builder().proc.slots.push((name, ty.ir()));
                self.emit(ir::Inst::Store {
                    ty: ty.ir(),
                    val: ir::Value::Temp(temp),
                    addr,
                });
            }
        }

        self.const_declarations(&declaration.consts);
        self.local_declarations(&declaration.vars);
        for proc in &declaration.procs {
            self.procedure(proc, &symbol);
        }
        self.lower_stmts(&declaration.body);
        self.lower_return(declaration, ret);

        self.procs
            .push(self.current.take().expect("procedure exists").finish());
        self.current = enclosing;
        self.scopes.pop().expect("procedure scope exists");
    }

    fn local_declarations(&mut self, declarations: &[ast::VarDecl]) {
        for declaration in declarations {
            let Some(ty) = self.resolve_type(&declaration.ty) else {
                continue;
            };
            for (name, pos) in &declaration.names {
                let addr = ir::Addr::Slot(name.clone());
                if self.declare(name, *pos, Symbol::Var { ty, addr }) {
                    self.builder().proc.slots.push((name.clone(), ty.ir()));
                }
            }
        }
    }

    fn lower_return(&mut self, declaration: &ast::ProcDecl, ret: Option<Type>) {
        match (&declaration.ret, ret, &declaration.ret_val) {
            (Some(_), Some(expected), Some(expr)) => {
                let value = self.lower_expr(expr);
                if let Some((value, found)) = value {
                    if found != expected {
                        self.diags.push(Diagnostic::new(
                            expr.pos(),
                            format!("RETURN expression has type {found}, expected {expected}"),
                        ));
                    }
                    self.emit(ir::Inst::Ret(Some(value)));
                } else {
                    self.emit(ir::Inst::Ret(None));
                }
            }
            (Some(_), Some(_), None) => {
                self.diags.push(Diagnostic::new(
                    declaration.pos,
                    format!(
                        "function procedure '{}' must end with RETURN",
                        declaration.name
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
                        declaration.name
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
            ast::Stmt::Assign { lhs, rhs, .. } => {
                let lhs = self.addr_of(lhs);
                let lowered_rhs = self.lower_expr(rhs);
                if let (Some((addr, lhs_ty)), Some((value, rhs_ty))) = (lhs, lowered_rhs) {
                    if lhs_ty == rhs_ty {
                        self.emit(ir::Inst::Store {
                            ty: lhs_ty.ir(),
                            val: value,
                            addr,
                        });
                    } else {
                        self.diags.push(Diagnostic::new(
                            rhs.pos(),
                            format!("cannot assign {rhs_ty} to {lhs_ty}"),
                        ));
                    }
                }
            }
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
            ast::Expr::Name(designator) => match self.resolve(designator) {
                Ok(Symbol::Const(value)) => Some((value.ir(), value.ty())),
                Ok(Symbol::Var { ty, addr }) => {
                    let dst = self.temp();
                    self.emit(ir::Inst::Load {
                        dst,
                        ty: ty.ir(),
                        addr,
                    });
                    Some((ir::Value::Temp(dst), ty))
                }
                Ok(_) => {
                    self.diags.push(Diagnostic::new(
                        designator.pos,
                        format!("'{}' cannot be used as a value", designator.name()),
                    ));
                    None
                }
                Err(diag) => {
                    self.diags.push(diag);
                    None
                }
            },
            ast::Expr::Call { callee, args, pos } => match self.lower_call(callee, args, *pos) {
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
                let expected = match op {
                    ast::UnOp::Neg => Type::Integer,
                    ast::UnOp::Not => Type::Boolean,
                };
                if found != expected {
                    self.diags.push(unary_type_error(
                        *pos,
                        match op {
                            ast::UnOp::Neg => "-",
                            ast::UnOp::Not => "~",
                        },
                        expected,
                        found,
                    ));
                    return None;
                }
                let dst = self.temp();
                self.emit(ir::Inst::Un {
                    dst,
                    op: match op {
                        ast::UnOp::Neg => ir::UnOp::Neg,
                        ast::UnOp::Not => ir::UnOp::Not,
                    },
                    arg,
                });
                Some((ir::Value::Temp(dst), expected))
            }
            ast::Expr::Binary { op, lhs, rhs, pos } => {
                if matches!(op, ast::BinOp::And | ast::BinOp::Or) {
                    self.lower_logical(*op, lhs, rhs, *pos)
                } else {
                    self.lower_binary(*op, lhs, rhs, *pos)
                }
            }
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
            src: ir::Value::Bool(short),
        });
        self.emit(ir::Inst::Jmp(end.clone()));
        self.emit(ir::Inst::Label(rhs_label));
        let rhs = self.lower_expr(rhs);
        if let Some((value, _)) = &rhs {
            self.emit(ir::Inst::Copy {
                dst: result,
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
        let lhs = self.lower_expr(lhs);
        let rhs_value = self.lower_expr(rhs);
        let (Some((lhs, lhs_ty)), Some((rhs_ir, rhs_ty))) = (lhs, rhs_value) else {
            return None;
        };

        use ast::BinOp;
        let result_ty = match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => check_binary_types(
                pos,
                bin_op_name(op),
                Type::Integer,
                lhs_ty,
                rhs_ty,
                Type::Integer,
                &mut self.diags,
            )?,
            BinOp::Eq | BinOp::Ne => {
                if lhs_ty != rhs_ty {
                    self.diags.push(Diagnostic::new(
                        pos,
                        format!(
                            "operator '{}' requires operands of the same type, found {lhs_ty} and {rhs_ty}",
                            bin_op_name(op)
                        ),
                    ));
                    return None;
                }
                Type::Boolean
            }
            BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => check_binary_types(
                pos,
                bin_op_name(op),
                Type::Integer,
                lhs_ty,
                rhs_ty,
                Type::Boolean,
                &mut self.diags,
            )?,
            BinOp::And | BinOp::Or => unreachable!(),
        };

        let value = match op {
            BinOp::Div | BinOp::Mod => {
                if !matches!(rhs, ast::Expr::Int { value, .. } if *value != 0) {
                    self.div_zero_check(rhs_ir.clone());
                }
                let (rem, adjust) = self.floor_adjust(lhs.clone(), rhs_ir.clone());
                if op == BinOp::Mod {
                    let delta = self.bin(ir::BinOp::Mul, adjust, rhs_ir);
                    self.bin(ir::BinOp::Add, rem, delta)
                } else {
                    let quotient = self.bin(ir::BinOp::Div, lhs, rhs_ir);
                    self.bin(ir::BinOp::Sub, quotient, adjust)
                }
            }
            _ => self.bin(
                match op {
                    BinOp::Add => ir::BinOp::Add,
                    BinOp::Sub => ir::BinOp::Sub,
                    BinOp::Mul => ir::BinOp::Mul,
                    BinOp::Eq => ir::BinOp::Eq,
                    BinOp::Ne => ir::BinOp::Ne,
                    BinOp::Lt => ir::BinOp::Lt,
                    BinOp::Le => ir::BinOp::Le,
                    BinOp::Gt => ir::BinOp::Gt,
                    BinOp::Ge => ir::BinOp::Ge,
                    BinOp::Div | BinOp::Mod | BinOp::And | BinOp::Or => unreachable!(),
                },
                lhs,
                rhs_ir,
            ),
        };
        Some((value, result_ty))
    }

    fn div_zero_check(&mut self, divisor: ir::Value) {
        let zero = self.bin(ir::BinOp::Eq, divisor, ir::Value::Int(0));
        let trap = self.label("div.zero");
        let ok = self.label("div.ok");
        self.emit(ir::Inst::Br {
            cond: zero,
            then: trap.clone(),
            els: ok.clone(),
        });
        self.emit(ir::Inst::Label(trap));
        self.emit(ir::Inst::Call {
            dst: None,
            symbol: "oberon_div_by_zero".into(),
            args: Vec::new(),
        });
        self.emit(ir::Inst::Halt);
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
        let rem = self.bin(ir::BinOp::Rem, lhs, rhs.clone());
        let nonzero = self.bin(ir::BinOp::Ne, rem.clone(), ir::Value::Int(0));
        let rem_neg = self.bin(ir::BinOp::Lt, rem.clone(), ir::Value::Int(0));
        let rhs_neg = self.bin(ir::BinOp::Lt, rhs, ir::Value::Int(0));
        let differ = self.bin(ir::BinOp::Ne, rem_neg, rhs_neg);
        let adjust = self.bin(ir::BinOp::Mul, nonzero, differ);
        (rem, adjust)
    }

    fn lower_call(
        &mut self,
        callee: &ast::Designator,
        actuals: &[ast::Expr],
        pos: Pos,
    ) -> Option<(Option<ir::Value>, Option<Type>)> {
        let proc = match self.resolve(callee) {
            Ok(Symbol::Proc {
                symbol,
                params,
                ret,
            }) => Some((symbol, params, ret)),
            Ok(_) => {
                self.diags.push(Diagnostic::new(
                    pos,
                    format!("'{}' is not a procedure", callee.name()),
                ));
                None
            }
            Err(diag) => {
                self.diags.push(diag);
                None
            }
        };

        let Some((symbol, params, ret)) = proc else {
            for actual in actuals {
                let _ = self.lower_expr(actual);
            }
            return None;
        };

        let mut ok = true;
        if actuals.len() != params.len() {
            self.diags.push(Diagnostic::new(
                pos,
                format!(
                    "wrong number of arguments: expected {}, found {}",
                    params.len(),
                    actuals.len()
                ),
            ));
            ok = false;
        }

        let mut args = Vec::new();
        for (i, actual) in actuals.iter().enumerate() {
            let Some((var, expected)) = params.get(i).copied() else {
                let _ = self.lower_expr(actual);
                continue;
            };
            if var {
                match self.var_actual(actual, i + 1) {
                    Some((addr, found)) => {
                        if found == expected {
                            args.push(ir::Arg::Ref(addr));
                        } else {
                            self.diags.push(Diagnostic::new(
                                actual.pos(),
                                format!("argument {} has type {found}, expected {expected}", i + 1),
                            ));
                            ok = false;
                        }
                    }
                    None => ok = false,
                }
            } else {
                match self.lower_expr(actual) {
                    Some((value, found)) => {
                        if found == expected {
                            args.push(ir::Arg::Val(expected.ir(), value));
                        } else {
                            self.diags.push(Diagnostic::new(
                                actual.pos(),
                                format!("argument {} has type {found}, expected {expected}", i + 1),
                            ));
                            ok = false;
                        }
                    }
                    None => ok = false,
                }
            }
        }

        if !ok {
            return None;
        }
        let dst = ret.map(|ty| (self.temp(), ty.ir()));
        self.emit(ir::Inst::Call { dst, symbol, args });
        Some((dst.map(|(temp, _)| ir::Value::Temp(temp)), ret))
    }

    fn var_actual(&mut self, actual: &ast::Expr, number: usize) -> Option<(ir::Addr, Type)> {
        if let ast::Expr::Name(designator) = actual {
            match self.resolve(designator) {
                Ok(Symbol::Var { ty, addr }) => Some((addr, ty)),
                Ok(_) => {
                    self.diags.push(Diagnostic::new(
                        actual.pos(),
                        format!("argument {number} must be a variable"),
                    ));
                    None
                }
                Err(diag) => {
                    self.diags.push(diag);
                    None
                }
            }
        } else {
            let _ = self.lower_expr(actual);
            self.diags.push(Diagnostic::new(
                actual.pos(),
                format!("argument {number} must be a variable"),
            ));
            None
        }
    }

    fn addr_of(&mut self, designator: &ast::Designator) -> Option<(ir::Addr, Type)> {
        match self.resolve(designator) {
            Ok(Symbol::Var { ty, addr }) => Some((addr, ty)),
            Ok(_) => {
                self.diags.push(Diagnostic::new(
                    designator.pos,
                    format!("cannot assign to '{}'", designator.name()),
                ));
                None
            }
            Err(diag) => {
                self.diags.push(diag);
                None
            }
        }
    }

    fn check_const_expr(&mut self, expr: &ast::Expr) -> Option<Type> {
        match expr {
            ast::Expr::Int { value, pos } => match i32::try_from(*value) {
                Ok(_) => Some(Type::Integer),
                Err(_) => {
                    self.diags
                        .push(Diagnostic::new(*pos, "integer literal out of range"));
                    None
                }
            },
            ast::Expr::Bool { .. } => Some(Type::Boolean),
            ast::Expr::Name(designator) => match self.resolve(designator) {
                Ok(Symbol::Const(value)) => Some(value.ty()),
                Ok(Symbol::Var { ty, .. }) => Some(ty),
                Ok(_) => {
                    self.diags.push(Diagnostic::new(
                        designator.pos,
                        format!("'{}' cannot be used as a value", designator.name()),
                    ));
                    None
                }
                Err(diag) => {
                    self.diags.push(diag);
                    None
                }
            },
            ast::Expr::Call { callee, args, pos } => self.check_const_call(callee, args, *pos),
            ast::Expr::Unary { op, expr, pos } => {
                let found = self.check_const_expr(expr)?;
                let expected = match op {
                    ast::UnOp::Neg => Type::Integer,
                    ast::UnOp::Not => Type::Boolean,
                };
                if found == expected {
                    Some(expected)
                } else {
                    self.diags.push(unary_type_error(
                        *pos,
                        match op {
                            ast::UnOp::Neg => "-",
                            ast::UnOp::Not => "~",
                        },
                        expected,
                        found,
                    ));
                    None
                }
            }
            ast::Expr::Binary { op, lhs, rhs, pos } => {
                let lhs = self.check_const_expr(lhs);
                let rhs = self.check_const_expr(rhs);
                let (Some(lhs), Some(rhs)) = (lhs, rhs) else {
                    return None;
                };
                use ast::BinOp;
                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                        check_binary_types(
                            *pos,
                            bin_op_name(*op),
                            Type::Integer,
                            lhs,
                            rhs,
                            Type::Integer,
                            &mut self.diags,
                        )
                    }
                    BinOp::And | BinOp::Or => check_binary_types(
                        *pos,
                        bin_op_name(*op),
                        Type::Boolean,
                        lhs,
                        rhs,
                        Type::Boolean,
                        &mut self.diags,
                    ),
                    BinOp::Eq | BinOp::Ne => {
                        if lhs == rhs {
                            Some(Type::Boolean)
                        } else {
                            self.diags.push(Diagnostic::new(
                                *pos,
                                format!(
                                    "operator '{}' requires operands of the same type, found {lhs} and {rhs}",
                                    bin_op_name(*op)
                                ),
                            ));
                            None
                        }
                    }
                    BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => check_binary_types(
                        *pos,
                        bin_op_name(*op),
                        Type::Integer,
                        lhs,
                        rhs,
                        Type::Boolean,
                        &mut self.diags,
                    ),
                }
            }
        }
    }

    fn check_const_call(
        &mut self,
        callee: &ast::Designator,
        actuals: &[ast::Expr],
        pos: Pos,
    ) -> Option<Type> {
        let proc = match self.resolve(callee) {
            Ok(Symbol::Proc { params, ret, .. }) => Some((params, ret)),
            Ok(_) => {
                self.diags.push(Diagnostic::new(
                    pos,
                    format!("'{}' is not a procedure", callee.name()),
                ));
                None
            }
            Err(diag) => {
                self.diags.push(diag);
                None
            }
        };
        let Some((params, ret)) = proc else {
            for actual in actuals {
                let _ = self.check_const_expr(actual);
            }
            return None;
        };
        if actuals.len() != params.len() {
            self.diags.push(Diagnostic::new(
                pos,
                format!(
                    "wrong number of arguments: expected {}, found {}",
                    params.len(),
                    actuals.len()
                ),
            ));
        }
        for (i, actual) in actuals.iter().enumerate() {
            let found = self.check_const_expr(actual);
            if let (Some(found), Some((_, expected))) = (found, params.get(i))
                && found != *expected
            {
                self.diags.push(Diagnostic::new(
                    actual.pos(),
                    format!("argument {} has type {found}, expected {expected}", i + 1),
                ));
            }
        }
        match ret {
            Some(ty) => Some(ty),
            None => {
                self.diags.push(Diagnostic::new(
                    pos,
                    format!("'{}' cannot be used as a value", callee.name()),
                ));
                None
            }
        }
    }

    fn eval_const(&self, expr: &ast::Expr) -> Result<ConstValue, Diagnostic> {
        match expr {
            ast::Expr::Int { value, pos } => i32::try_from(*value)
                .map(ConstValue::Int)
                .map_err(|_| Diagnostic::new(*pos, "integer literal out of range")),
            ast::Expr::Bool { value, .. } => Ok(ConstValue::Bool(*value)),
            ast::Expr::Name(designator) => match self.resolve(designator)? {
                Symbol::Const(value) => Ok(value),
                _ => Err(Diagnostic::new(
                    designator.pos,
                    format!("'{}' is not a constant", designator.name()),
                )),
            },
            ast::Expr::Call { pos, .. } => Err(Diagnostic::new(
                *pos,
                "constant expression contains a procedure call",
            )),
            ast::Expr::Unary { op, expr, pos } => {
                let value = self.eval_const(expr)?;
                match (op, value) {
                    (ast::UnOp::Neg, ConstValue::Int(value)) => value
                        .checked_neg()
                        .map(ConstValue::Int)
                        .ok_or_else(|| Diagnostic::new(*pos, "constant expression overflows")),
                    (ast::UnOp::Not, ConstValue::Bool(value)) => Ok(ConstValue::Bool(!value)),
                    _ => unreachable!("constant expression was type-checked"),
                }
            }
            ast::Expr::Binary {
                op: ast::BinOp::And,
                lhs,
                rhs,
                ..
            } => match self.eval_const(lhs)? {
                ConstValue::Bool(false) => Ok(ConstValue::Bool(false)),
                ConstValue::Bool(true) => self.eval_const(rhs),
                ConstValue::Int(_) => unreachable!("constant expression was type-checked"),
            },
            ast::Expr::Binary {
                op: ast::BinOp::Or,
                lhs,
                rhs,
                ..
            } => match self.eval_const(lhs)? {
                ConstValue::Bool(true) => Ok(ConstValue::Bool(true)),
                ConstValue::Bool(false) => self.eval_const(rhs),
                ConstValue::Int(_) => unreachable!("constant expression was type-checked"),
            },
            ast::Expr::Binary { op, lhs, rhs, pos } => {
                let lhs = self.eval_const(lhs)?;
                let rhs = self.eval_const(rhs)?;
                eval_const_binary(*op, lhs, rhs, *pos)
            }
        }
    }

    fn resolve_type(&mut self, designator: &ast::Designator) -> Option<Type> {
        match self.resolve(designator) {
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
        }
    }

    fn resolve(&self, designator: &ast::Designator) -> Result<Symbol, Diagnostic> {
        let (scope_index, mut symbol) = self
            .scopes
            .iter()
            .enumerate()
            .rev()
            .find_map(|(index, scope)| {
                scope
                    .get(&designator.ident)
                    .cloned()
                    .map(|symbol| (index, symbol))
            })
            .ok_or_else(|| {
                Diagnostic::new(
                    designator.pos,
                    format!("undeclared identifier '{}'", designator.ident),
                )
            })?;
        // Report 10: a procedure body sees its formals, its own locals, and
        // the module's objects. An enclosing procedure's variables are absent
        // from that list, so Oberon-07 needs no static link. Constants and
        // procedures stay visible at every level, following ORG.MakeItem and
        // OJB.thisObj; OBNC hides intermediate constants too, and we don't.
        //
        // "Enclosing" is read off the stack position rather than a level
        // stored on the symbol, which holds only because a procedure body is
        // the one thing that pushes a scope: the stack is always
        // [module, outermost proc, ..., current proc]. A slice that pushes a
        // scope for anything else must record the level on the symbol
        // instead, or this test quietly starts letting those variables in.
        if scope_index != 0
            && scope_index + 1 != self.scopes.len()
            && matches!(symbol, Symbol::Var { .. })
        {
            return Err(Diagnostic::new(
                designator.pos,
                format!(
                    "'{}' is not accessible: a nested procedure cannot use the variables or parameters of an enclosing procedure",
                    designator.ident
                ),
            ));
        }
        for selector in &designator.selectors {
            match (symbol, selector) {
                (Symbol::Module(members), ast::Selector::Field(name, pos)) => {
                    symbol = members.get(name).cloned().ok_or_else(|| {
                        Diagnostic::new(
                            *pos,
                            format!(
                                "'{}' is not declared in module '{}'",
                                name, designator.ident
                            ),
                        )
                    })?;
                }
                (_, ast::Selector::Field(name, pos)) => {
                    return Err(Diagnostic::new(
                        *pos,
                        format!(
                            "cannot select '{name}' from '{}': record field selection is not yet supported",
                            designator.ident
                        ),
                    ));
                }
            }
        }
        Ok(symbol)
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

    fn bin(&mut self, op: ir::BinOp, lhs: ir::Value, rhs: ir::Value) -> ir::Value {
        let dst = self.temp();
        self.emit(ir::Inst::Bin { dst, op, lhs, rhs });
        ir::Value::Temp(dst)
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
                ret: ret.map(Type::ir),
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

fn out_scope() -> HashMap<String, Symbol> {
    HashMap::from([
        (
            "Int".into(),
            Symbol::Proc {
                symbol: "oberon_out_int".into(),
                params: vec![(false, Type::Integer), (false, Type::Integer)],
                ret: None,
            },
        ),
        (
            "Ln".into(),
            Symbol::Proc {
                symbol: "oberon_out_ln".into(),
                params: Vec::new(),
                ret: None,
            },
        ),
    ])
}

fn eval_const_binary(
    op: ast::BinOp,
    lhs: ConstValue,
    rhs: ConstValue,
    pos: Pos,
) -> Result<ConstValue, Diagnostic> {
    use ast::BinOp;
    match op {
        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
            let (ConstValue::Int(lhs), ConstValue::Int(rhs)) = (lhs, rhs) else {
                unreachable!("constant expression was type-checked");
            };
            if rhs == 0 && matches!(op, BinOp::Div | BinOp::Mod) {
                return Err(Diagnostic::new(pos, "constant DIV or MOD by zero"));
            }
            let value = match op {
                BinOp::Add => lhs.checked_add(rhs),
                BinOp::Sub => lhs.checked_sub(rhs),
                BinOp::Mul => lhs.checked_mul(rhs),
                // Report 8.2.2 requires floored DIV and MOD. Keep this
                // adjustment identical to the sequence floor_adjust emits.
                BinOp::Div | BinOp::Mod => {
                    let remainder = lhs.checked_rem(rhs);
                    let adjust = matches!(remainder, Some(r) if r != 0 && (r < 0) != (rhs < 0));
                    if op == BinOp::Mod {
                        if adjust {
                            remainder.and_then(|r| r.checked_add(rhs))
                        } else {
                            remainder
                        }
                    } else {
                        let quotient = lhs.checked_div(rhs);
                        if adjust {
                            quotient.and_then(|q| q.checked_sub(1))
                        } else {
                            quotient
                        }
                    }
                }
                _ => unreachable!(),
            };
            value
                .map(ConstValue::Int)
                .ok_or_else(|| Diagnostic::new(pos, "constant expression overflows"))
        }
        BinOp::Eq | BinOp::Ne => {
            assert_eq!(lhs.ty(), rhs.ty(), "constant expression was type-checked");
            let equal = lhs == rhs;
            Ok(ConstValue::Bool(if op == BinOp::Eq {
                equal
            } else {
                !equal
            }))
        }
        BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
            let (ConstValue::Int(lhs), ConstValue::Int(rhs)) = (lhs, rhs) else {
                unreachable!("constant expression was type-checked");
            };
            Ok(ConstValue::Bool(match op {
                BinOp::Lt => lhs < rhs,
                BinOp::Le => lhs <= rhs,
                BinOp::Gt => lhs > rhs,
                BinOp::Ge => lhs >= rhs,
                _ => unreachable!(),
            }))
        }
        BinOp::And | BinOp::Or => unreachable!("short-circuit operators handled separately"),
    }
}

fn check_binary_types(
    pos: Pos,
    op: &str,
    expected: Type,
    lhs: Type,
    rhs: Type,
    result: Type,
    diags: &mut Vec<Diagnostic>,
) -> Option<Type> {
    if lhs == expected && rhs == expected {
        Some(result)
    } else {
        diags.push(Diagnostic::new(
            pos,
            format!("operator '{op}' requires {expected} and {expected}, found {lhs} and {rhs}"),
        ));
        None
    }
}

fn unary_type_error(pos: Pos, op: &str, expected: Type, found: Type) -> Diagnostic {
    Diagnostic::new(
        pos,
        format!("operator '{op}' requires {expected}, found {found}"),
    )
}

fn bin_op_name(op: ast::BinOp) -> &'static str {
    match op {
        ast::BinOp::Add => "+",
        ast::BinOp::Sub => "-",
        ast::BinOp::Mul => "*",
        ast::BinOp::Div => "DIV",
        ast::BinOp::Mod => "MOD",
        ast::BinOp::Eq => "=",
        ast::BinOp::Ne => "#",
        ast::BinOp::Lt => "<",
        ast::BinOp::Le => "<=",
        ast::BinOp::Gt => ">",
        ast::BinOp::Ge => ">=",
        ast::BinOp::And => "&",
        ast::BinOp::Or => "OR",
    }
}
