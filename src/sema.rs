use std::collections::HashMap;
use std::fmt;

use crate::ast;
use crate::diag::{Diagnostic, Pos};
use crate::ir;

type Scope = HashMap<String, Symbol>;

// The scope stack is always [universe, module, outermost proc, ..., current
// proc], so this index names the module scope and its length names "the
// module scope is innermost".
const MODULE_SCOPE: usize = 1;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
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
pub enum ConstValue {
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
        // Set on an imported variable, and while a FOR statement's body is
        // being lowered so the body cannot move its own control variable.
        read_only: bool,
    },
    Proc {
        symbol: String,
        params: Vec<(bool, Type)>,
        ret: Option<Type>,
    },
    Module(HashMap<String, Symbol>),
    TypeName(Type),
    Builtin(Builtin),
}

// The public face of an analyzed module. Report 4 says a client sees only the
// marked declarations, so an unmarked one is absent here rather than present
// and hidden: no lookup path can forget to check a visibility flag that does
// not exist. The interface owns its data and is cloned into each client.
//
// This is a build result held in memory, not a symbol file. Slice 10 adds
// named types to it when TYPE declarations arrive.
#[derive(Debug, Clone, Default)]
pub struct Interface {
    pub members: HashMap<String, Member>,
}

#[derive(Debug, Clone)]
pub enum Member {
    Const(ConstValue),
    Var {
        ty: Type,
        symbol: String,
    },
    Proc {
        symbol: String,
        params: Vec<(bool, Type)>,
        ret: Option<Type>,
    },
}

impl Member {
    // Report 9.1 and 11: an imported variable is read-only in the client. The
    // declaring module keeps the writable symbol it built for itself.
    fn client_symbol(&self) -> Symbol {
        match self {
            Member::Const(value) => Symbol::Const(*value),
            Member::Var { ty, symbol } => Symbol::Var {
                ty: *ty,
                addr: ir::Addr::Global(symbol.clone()),
                read_only: true,
            },
            Member::Proc {
                symbol,
                params,
                ret,
            } => Symbol::Proc {
                symbol: symbol.clone(),
                params: params.clone(),
                ret: *ret,
            },
        }
    }
}

// Report 10.2. Only the operations whose argument types exist are here; LEN,
// FLOOR, FLT, CHR, INCL, EXCL, NEW, PACK, and UNPK arrive with their types.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Builtin {
    Abs,
    Odd,
    Lsl,
    Asr,
    Ror,
    Ord,
    Inc,
    Dec,
    Assert,
}

fn universe_scope() -> Scope {
    let mut scope = Scope::new();
    scope.insert("INTEGER".into(), Symbol::TypeName(Type::Integer));
    scope.insert("BOOLEAN".into(), Symbol::TypeName(Type::Boolean));
    for (name, builtin) in [
        ("ABS", Builtin::Abs),
        ("ODD", Builtin::Odd),
        ("LSL", Builtin::Lsl),
        ("ASR", Builtin::Asr),
        ("ROR", Builtin::Ror),
        ("ORD", Builtin::Ord),
        ("INC", Builtin::Inc),
        ("DEC", Builtin::Dec),
        ("ASSERT", Builtin::Assert),
    ] {
        scope.insert(name.into(), Symbol::Builtin(builtin));
    }
    scope
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
    procs: Vec<ir::Proc>,
    interface: Interface,
    current: Option<ProcBuilder>,
}

impl Analyzer {
    fn new(module: &str) -> Self {
        Self {
            module: module.into(),
            // Report 4: the predefined identifiers are not declared in any
            // module, so a module may declare its own ABS and shadow this one
            // rather than colliding with it. ORB.Init builds the same
            // universe scope and opens the module scope inside it.
            scopes: vec![universe_scope(), Scope::new()],
            diags: Vec::new(),
            globals: Vec::new(),
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
        self.imports(&module.imports, resolved);
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
            Ok((
                ir::Module {
                    name: self.module,
                    globals: self.globals,
                    procs: self.procs,
                },
                self.interface,
            ))
        } else {
            Err(self.diags)
        }
    }

    // Report 11: an import declares the qualifier in the client's module
    // scope. Two imports of one module under different qualifiers are legal
    // and share its objects, because the symbols come from the interface and
    // never from the qualifier.
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
            self.declare(
                &import.qualifier,
                import.qualifier_pos,
                Symbol::Module(members),
            );
        }
    }

    // Report 4 permits an export mark only on a declaration in a module's
    // scope. cf. ORP.CheckExport, which consumes the mark at every identdef
    // and reports it when the declaration level is not zero.
    fn check_export(&mut self, id: &ast::IdentDef) -> bool {
        if !id.export {
            return false;
        }
        if self.scopes.len() != MODULE_SCOPE + 1 {
            self.diags.push(Diagnostic::new(
                id.pos,
                format!(
                    "'{}' cannot be exported: only a declaration in the module's scope can be marked",
                    id.name
                ),
            ));
            return false;
        }
        true
    }

    fn export(&mut self, id: &ast::IdentDef, member: Member) {
        if self.check_export(id) {
            self.interface.members.insert(id.name.clone(), member);
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
                    if self.declare(
                        &declaration.id.name,
                        declaration.id.pos,
                        Symbol::Const(value),
                    ) {
                        self.export(&declaration.id, Member::Const(value));
                    }
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
            // One declaration may mix marked and unmarked names, so the mark
            // is read per name rather than per declaration.
            for id in &declaration.names {
                let symbol = format!("{}.{}", self.module, id.name);
                let addr = ir::Addr::Global(symbol.clone());
                if self.declare(
                    &id.name,
                    id.pos,
                    Symbol::Var {
                        ty,
                        addr,
                        read_only: false,
                    },
                ) {
                    self.globals.push(ir::Global {
                        symbol: symbol.clone(),
                        ty: ty.ir(),
                    });
                    self.export(id, Member::Var { ty, symbol });
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
        let symbol = format!("{prefix}.{}", declaration.id.name);
        if params_ok && ret_ok {
            let params: Vec<_> = formals.iter().map(|(var, _, _, ty)| (*var, *ty)).collect();
            if self.declare(
                &declaration.id.name,
                declaration.id.pos,
                Symbol::Proc {
                    symbol: symbol.clone(),
                    params: params.clone(),
                    ret,
                },
            ) {
                self.export(
                    &declaration.id,
                    Member::Proc {
                        symbol: symbol.clone(),
                        params,
                        ret,
                    },
                );
            }
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
                    read_only: false,
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
            for id in &declaration.names {
                let addr = ir::Addr::Slot(id.name.clone());
                if self.declare(
                    &id.name,
                    id.pos,
                    Symbol::Var {
                        ty,
                        addr,
                        read_only: false,
                    },
                ) {
                    self.builder().proc.slots.push((id.name.clone(), ty.ir()));
                    // A local can never be exported, but the mark still has
                    // to be diagnosed rather than ignored.
                    self.check_export(id);
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
            ast::Stmt::For {
                var,
                start,
                limit,
                step,
                body,
            } => self.lower_for(var, start, limit, step.as_ref(), body),
            ast::Stmt::Case { expr, arms } => self.lower_case(expr, arms),
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

    // Report 9.8 defines the statement by rewriting it as a WHILE whose
    // condition contains the limit expression, so the limit is recomputed
    // before every test, including the final failing one. ORP.StatSequence
    // and OJP.StatSequence both record the loop top before parsing the limit,
    // and OBNC puts it in a C for-condition, so all three do the same.
    fn lower_for(
        &mut self,
        var: &ast::Designator,
        start: &ast::Expr,
        limit: &ast::Expr,
        step: Option<&ast::Expr>,
        body: &[ast::Stmt],
    ) {
        let control = match self.addr_of(var) {
            Some((addr, Type::Integer)) => Some(addr),
            Some((_, ty)) => {
                self.diags.push(Diagnostic::new(
                    var.pos,
                    format!(
                        "control variable '{}' must be INTEGER, found {ty}",
                        var.ident
                    ),
                ));
                None
            }
            None => None,
        };
        let step = self.for_step(step);

        let start = self.lower_int(start, "FOR start value");
        if let (Some(addr), Some(value)) = (control.clone(), start) {
            self.emit(ir::Inst::Store {
                ty: ir::Ty::Int,
                val: value,
                addr,
            });
        }

        let test = self.label("for.test");
        let end = self.label("for.end");
        self.emit(ir::Inst::Jmp(test.clone()));
        self.emit(ir::Inst::Label(test.clone()));

        let limit = self.lower_int(limit, "FOR limit");
        if let (Some(addr), Some(limit)) = (control.clone(), limit) {
            let current = self.load(addr, Type::Integer);
            // The direction is decided here, from the sign of the folded
            // step, so nothing tests the step at run time.
            let op = if step < 0 {
                ir::BinOp::Ge
            } else {
                ir::BinOp::Le
            };
            let cond = self.bin(op, current, limit);
            let yes = self.label("for.body");
            self.emit(ir::Inst::Br {
                cond,
                then: yes.clone(),
                els: end.clone(),
            });
            self.emit(ir::Inst::Label(yes));
        }

        let previous = self.set_read_only(&var.ident, true);
        self.lower_stmts(body);
        self.set_read_only(&var.ident, previous);

        if let Some(addr) = control {
            let current = self.load(addr.clone(), Type::Integer);
            let next = self.bin(ir::BinOp::Add, current, ir::Value::Int(step));
            self.emit(ir::Inst::Store {
                ty: ir::Ty::Int,
                val: next,
                addr,
            });
        }
        self.emit(ir::Inst::Jmp(test));
        self.emit(ir::Inst::Label(end));
    }

    // BY takes a ConstExpression, and a step of zero cannot terminate:
    // ORP reports "zero increment" and this compiler does the same rather
    // than emitting an infinite loop. One is the recovery value so the body
    // still gets lowered and diagnosed.
    fn for_step(&mut self, step: Option<&ast::Expr>) -> i32 {
        let Some(expr) = step else { return 1 };
        if self.check_const_expr(expr).is_none() {
            return 1;
        }
        match self.eval_const(expr) {
            Ok(ConstValue::Int(0)) => {
                self.diags
                    .push(Diagnostic::new(expr.pos(), "FOR step must not be zero"));
                1
            }
            Ok(ConstValue::Int(value)) => value,
            Ok(ConstValue::Bool(_)) => {
                self.diags.push(Diagnostic::new(
                    expr.pos(),
                    "FOR step must be INTEGER, found BOOLEAN",
                ));
                1
            }
            Err(diag) => {
                self.diags.push(diag);
                1
            }
        }
    }

    // Report 9.5. Only the INTEGER form: CHAR labels arrive with CHAR, and
    // the record and pointer form of the statement arrives with pointers.
    fn lower_case(&mut self, expr: &ast::Expr, arms: &[ast::CaseArm]) {
        // "First the case expression is evaluated": once, into a temporary
        // that every arm's test then compares against.
        let selector = self.lower_int(expr, "CASE expression");

        // One label table for the whole statement, as in oberonc: labels must
        // be distinct across alternatives, not just within one.
        let mut covered: Vec<(i32, i32)> = Vec::new();
        let mut arm_ranges = Vec::new();
        for arm in arms {
            let mut ranges = Vec::new();
            for range in &arm.labels {
                let low = self.case_label(&range.low);
                let high = match &range.high {
                    Some(expr) => self.case_label(expr),
                    None => low,
                };
                let (Some(low), Some(high)) = (low, high) else {
                    continue;
                };
                if low > high {
                    self.diags.push(Diagnostic::new(
                        range.low.pos(),
                        format!("case label range {low}..{high} is reversed"),
                    ));
                    continue;
                }
                if covered.iter().any(|(a, b)| low <= *b && *a <= high) {
                    self.diags.push(Diagnostic::new(
                        range.low.pos(),
                        format!(
                            "{} already covered by an earlier alternative",
                            label_text(low, high)
                        ),
                    ));
                    continue;
                }
                covered.push((low, high));
                ranges.push((low, high));
            }
            arm_ranges.push(ranges);
        }

        let Some(selector) = selector else {
            for arm in arms {
                self.lower_stmts(&arm.body);
            }
            return;
        };

        let end = self.label("case.end");
        for (arm, ranges) in arms.iter().zip(&arm_ranges) {
            if ranges.is_empty() {
                self.lower_stmts(&arm.body);
                continue;
            }
            let body = self.label("case.arm");
            let next = self.label("case.next");
            for (low, high) in ranges {
                let cond = if low == high {
                    self.bin(ir::BinOp::Eq, selector.clone(), ir::Value::Int(*low))
                } else {
                    // Comparisons yield 0 or 1, so a bitwise and is the
                    // conjunction of the two bounds tests.
                    let above = self.bin(ir::BinOp::Ge, selector.clone(), ir::Value::Int(*low));
                    let below = self.bin(ir::BinOp::Le, selector.clone(), ir::Value::Int(*high));
                    self.bin(ir::BinOp::BitAnd, above, below)
                };
                let miss = self.label("case.test");
                self.emit(ir::Inst::Br {
                    cond,
                    then: body.clone(),
                    els: miss.clone(),
                });
                self.emit(ir::Inst::Label(miss));
            }
            self.emit(ir::Inst::Jmp(next.clone()));
            self.emit(ir::Inst::Label(body));
            self.lower_stmts(&arm.body);
            self.emit(ir::Inst::Jmp(end.clone()));
            self.emit(ir::Inst::Label(next));
        }
        // Oberon-07 has no ELSE in a case statement, so a selector matching
        // no label has to mean something. oberonc traps and OBNC raises; both
        // beat quietly doing nothing when a label has a typo in it.
        self.trap("oberon_case_no_match");
        self.emit(ir::Inst::Label(end));
    }

    fn case_label(&mut self, expr: &ast::Expr) -> Option<i32> {
        match self.eval_const(expr) {
            Ok(ConstValue::Int(value)) => Some(value),
            Ok(ConstValue::Bool(_)) => {
                self.diags.push(Diagnostic::new(
                    expr.pos(),
                    "case label must be INTEGER, found BOOLEAN",
                ));
                None
            }
            Err(diag) => {
                self.diags.push(diag);
                None
            }
        }
    }

    fn lower_int(&mut self, expr: &ast::Expr, what: &str) -> Option<ir::Value> {
        let (value, ty) = self.lower_expr(expr)?;
        if ty == Type::Integer {
            Some(value)
        } else {
            self.diags.push(Diagnostic::new(
                expr.pos(),
                format!("{what} must be INTEGER, found {ty}"),
            ));
            None
        }
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
                Ok(Symbol::Var { ty, addr, .. }) => Some((self.load(addr, ty), ty)),
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
        self.trap("oberon_div_by_zero");
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
            // Before the user-procedure path: the predefined operations are
            // generic or variable in arity, and none of them is a call in the
            // emitted code.
            Ok(Symbol::Builtin(builtin)) => return self.lower_builtin(builtin, actuals, pos),
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

    fn lower_builtin(
        &mut self,
        builtin: Builtin,
        actuals: &[ast::Expr],
        pos: Pos,
    ) -> Option<(Option<ir::Value>, Option<Type>)> {
        match builtin {
            Builtin::Inc | Builtin::Dec => return self.lower_inc_dec(builtin, actuals, pos),
            Builtin::Assert => return self.lower_assert(actuals, pos),
            _ => {}
        }

        let (params, ret) = builtin_signature(builtin).expect("function-like builtin");
        if !self.builtin_arity(actuals, params.len(), pos) {
            return None;
        }
        let mut args = Vec::new();
        let mut ok = true;
        for (i, (actual, expected)) in actuals.iter().zip(params).enumerate() {
            // Every argument is lowered even after one of them fails, so a
            // call reports all of its type errors rather than only the first.
            match self.builtin_arg(actual, i + 1, *expected) {
                Some(value) => args.push(value),
                None => ok = false,
            }
        }
        if !ok {
            return None;
        }

        let value = match builtin {
            Builtin::Abs => self.lower_abs(args[0].clone()),
            // Truncating remainder, so the test must be against zero and not
            // against one: -3 rem 2 is -1. This agrees with the Report's
            // "x MOD 2 = 1" under the floored MOD the compiler implements.
            Builtin::Odd => {
                let rem = self.bin(ir::BinOp::Rem, args[0].clone(), ir::Value::Int(2));
                self.bin(ir::BinOp::Ne, rem, ir::Value::Int(0))
            }
            // BOOLEAN is already 0 or 1 in a word, so the value passes
            // through with only its type changed.
            Builtin::Ord => args[0].clone(),
            Builtin::Lsl | Builtin::Asr | Builtin::Ror => {
                self.lower_shift(builtin, &args, &actuals[1])?
            }
            Builtin::Inc | Builtin::Dec | Builtin::Assert => unreachable!("handled above"),
        };
        Some((Some(value), Some(ret)))
    }

    // MIN(INTEGER) has no absolute value. Folding rejects it, so the runtime
    // form must not quietly wrap the way unary minus still does.
    fn lower_abs(&mut self, arg: ir::Value) -> ir::Value {
        let overflows = self.bin(ir::BinOp::Eq, arg.clone(), ir::Value::Int(i32::MIN));
        let bad = self.label("abs.bad");
        let ok = self.label("abs.ok");
        self.emit(ir::Inst::Br {
            cond: overflows,
            then: bad.clone(),
            els: ok.clone(),
        });
        self.emit(ir::Inst::Label(bad));
        self.trap("oberon_abs_overflow");
        self.emit(ir::Inst::Label(ok));

        let negative = self.bin(ir::BinOp::Lt, arg.clone(), ir::Value::Int(0));
        let negate = self.label("abs.neg");
        let keep = self.label("abs.pos");
        let end = self.label("abs.end");
        let result = self.temp();
        self.emit(ir::Inst::Br {
            cond: negative,
            then: negate.clone(),
            els: keep.clone(),
        });
        self.emit(ir::Inst::Label(negate));
        let negated = self.temp();
        self.emit(ir::Inst::Un {
            dst: negated,
            op: ir::UnOp::Neg,
            arg: arg.clone(),
        });
        self.emit(ir::Inst::Copy {
            dst: result,
            src: ir::Value::Temp(negated),
        });
        self.emit(ir::Inst::Jmp(end.clone()));
        self.emit(ir::Inst::Label(keep));
        self.emit(ir::Inst::Copy {
            dst: result,
            src: arg,
        });
        self.emit(ir::Inst::Jmp(end.clone()));
        self.emit(ir::Inst::Label(end));
        ir::Value::Temp(result)
    }

    // The Report does not constrain the count, and QBE reduces it modulo the
    // result width, which would silently turn LSL(x, 32) into x. Diagnose a
    // count the lowering already knows and check the rest at run time, which
    // is the rule Slice 8 commits to for dynamic array indices.
    fn lower_shift(
        &mut self,
        builtin: Builtin,
        args: &[ir::Value],
        count_expr: &ast::Expr,
    ) -> Option<ir::Value> {
        let (value, count) = (args[0].clone(), args[1].clone());
        match &count {
            ir::Value::Int(n) => {
                if !(0..=31).contains(n) {
                    self.diags.push(shift_range_error(count_expr.pos(), *n));
                    return None;
                }
            }
            _ => {
                let low = self.bin(ir::BinOp::Lt, count.clone(), ir::Value::Int(0));
                let high = self.bin(ir::BinOp::Gt, count.clone(), ir::Value::Int(31));
                let bad = self.bin(ir::BinOp::BitOr, low, high);
                let trap = self.label("shift.bad");
                let ok = self.label("shift.ok");
                self.emit(ir::Inst::Br {
                    cond: bad,
                    then: trap.clone(),
                    els: ok.clone(),
                });
                self.emit(ir::Inst::Label(trap));
                self.trap("oberon_shift_range");
                self.emit(ir::Inst::Label(ok));
            }
        }
        Some(match builtin {
            Builtin::Lsl => self.bin(ir::BinOp::Shl, value, count),
            Builtin::Asr => self.bin(ir::BinOp::Sar, value, count),
            // A logical right shift merged with the bits that fall off the
            // bottom. The mask makes a rotation by zero the identity instead
            // of a shift by the word width.
            Builtin::Ror => {
                let right = self.bin(ir::BinOp::Shr, value.clone(), count.clone());
                let complement = self.bin(ir::BinOp::Sub, ir::Value::Int(32), count);
                let left_count = self.bin(ir::BinOp::BitAnd, complement, ir::Value::Int(31));
                let left = self.bin(ir::BinOp::Shl, value, left_count);
                self.bin(ir::BinOp::BitOr, right, left)
            }
            _ => unreachable!("not a shift"),
        })
    }

    fn lower_inc_dec(
        &mut self,
        builtin: Builtin,
        actuals: &[ast::Expr],
        pos: Pos,
    ) -> Option<(Option<ir::Value>, Option<Type>)> {
        if actuals.is_empty() || actuals.len() > 2 {
            self.diags.push(Diagnostic::new(
                pos,
                format!(
                    "wrong number of arguments: expected 1 or 2, found {}",
                    actuals.len()
                ),
            ));
            for actual in actuals {
                let _ = self.lower_expr(actual);
            }
            return None;
        }
        let target = match self.var_actual(&actuals[0], 1) {
            Some((addr, Type::Integer)) => Some(addr),
            Some((_, ty)) => {
                self.diags.push(Diagnostic::new(
                    actuals[0].pos(),
                    format!("argument 1 has type {ty}, expected INTEGER"),
                ));
                None
            }
            None => None,
        };
        let step = match actuals.get(1) {
            Some(actual) => self.builtin_arg(actual, 2, Type::Integer),
            None => Some(ir::Value::Int(1)),
        };
        if let (Some(addr), Some(step)) = (target, step) {
            let current = self.load(addr.clone(), Type::Integer);
            let op = if builtin == Builtin::Inc {
                ir::BinOp::Add
            } else {
                ir::BinOp::Sub
            };
            let next = self.bin(op, current, step);
            self.emit(ir::Inst::Store {
                ty: ir::Ty::Int,
                val: next,
                addr,
            });
        }
        Some((None, None))
    }

    fn lower_assert(
        &mut self,
        actuals: &[ast::Expr],
        pos: Pos,
    ) -> Option<(Option<ir::Value>, Option<Type>)> {
        if !self.builtin_arity(actuals, 1, pos) {
            return None;
        }
        // No constant special case: ASSERT(FALSE) emits the test like any
        // other condition and QBE folds it.
        if let Some(cond) = self.builtin_arg(&actuals[0], 1, Type::Boolean) {
            let bad = self.label("assert.bad");
            let ok = self.label("assert.ok");
            self.emit(ir::Inst::Br {
                cond,
                then: ok.clone(),
                els: bad.clone(),
            });
            self.emit(ir::Inst::Label(bad));
            self.trap("oberon_assert_failed");
            self.emit(ir::Inst::Label(ok));
        }
        Some((None, None))
    }

    fn builtin_arity(&mut self, actuals: &[ast::Expr], count: usize, pos: Pos) -> bool {
        if actuals.len() == count {
            return true;
        }
        self.diags.push(Diagnostic::new(
            pos,
            format!(
                "wrong number of arguments: expected {count}, found {}",
                actuals.len()
            ),
        ));
        for actual in actuals {
            let _ = self.lower_expr(actual);
        }
        false
    }

    fn builtin_arg(
        &mut self,
        actual: &ast::Expr,
        number: usize,
        expected: Type,
    ) -> Option<ir::Value> {
        let (value, found) = self.lower_expr(actual)?;
        if found == expected {
            Some(value)
        } else {
            self.diags.push(Diagnostic::new(
                actual.pos(),
                format!("argument {number} has type {found}, expected {expected}"),
            ));
            None
        }
    }

    fn var_actual(&mut self, actual: &ast::Expr, number: usize) -> Option<(ir::Addr, Type)> {
        if let ast::Expr::Name(designator) = actual {
            match self.resolve(designator) {
                Ok(Symbol::Var {
                    ty,
                    addr,
                    read_only,
                }) => {
                    if read_only {
                        self.diags.push(Diagnostic::new(
                            actual.pos(),
                            format!("argument {number} is read-only"),
                        ));
                        return None;
                    }
                    Some((addr, ty))
                }
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
            Ok(Symbol::Var {
                ty,
                addr,
                read_only,
            }) => {
                if read_only {
                    self.diags.push(Diagnostic::new(
                        designator.pos,
                        format!("cannot assign to '{}': it is read-only", designator.name()),
                    ));
                    return None;
                }
                Some((addr, ty))
            }
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
            Ok(Symbol::Builtin(builtin)) => {
                return self.check_const_builtin(builtin, callee, actuals, pos);
            }
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

    fn check_const_builtin(
        &mut self,
        builtin: Builtin,
        callee: &ast::Designator,
        actuals: &[ast::Expr],
        pos: Pos,
    ) -> Option<Type> {
        let Some((params, ret)) = builtin_signature(builtin) else {
            for actual in actuals {
                let _ = self.check_const_expr(actual);
            }
            self.diags.push(Diagnostic::new(
                pos,
                format!("'{}' cannot be used as a value", callee.name()),
            ));
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
            for actual in actuals {
                let _ = self.check_const_expr(actual);
            }
            // Unlike a user procedure, stop here: eval_const would otherwise
            // fold the same call and report the same arity twice.
            return None;
        }
        let mut ok = true;
        for (i, (actual, expected)) in actuals.iter().zip(params).enumerate() {
            let Some(found) = self.check_const_expr(actual) else {
                ok = false;
                continue;
            };
            if found != *expected {
                self.diags.push(Diagnostic::new(
                    actual.pos(),
                    format!("argument {} has type {found}, expected {expected}", i + 1),
                ));
                ok = false;
            }
        }
        ok.then_some(ret)
    }

    fn eval_const_builtin(
        &self,
        builtin: Builtin,
        callee: &ast::Designator,
        actuals: &[ast::Expr],
        pos: Pos,
    ) -> Result<ConstValue, Diagnostic> {
        if builtin_signature(builtin).is_none() {
            return Err(Diagnostic::new(
                pos,
                format!("'{}' cannot be used as a value", callee.name()),
            ));
        }
        let args = actuals
            .iter()
            .map(|actual| self.eval_const(actual))
            .collect::<Result<Vec<_>, _>>()?;
        match (builtin, args.as_slice()) {
            (Builtin::Abs, [ConstValue::Int(value)]) => value
                .checked_abs()
                .map(ConstValue::Int)
                .ok_or_else(|| Diagnostic::new(pos, "constant expression overflows")),
            (Builtin::Odd, [ConstValue::Int(value)]) => Ok(ConstValue::Bool(value % 2 != 0)),
            (Builtin::Ord, [ConstValue::Bool(value)]) => Ok(ConstValue::Int(i32::from(*value))),
            (
                Builtin::Lsl | Builtin::Asr | Builtin::Ror,
                [ConstValue::Int(x), ConstValue::Int(n)],
            ) => {
                let count = u32::try_from(*n)
                    .ok()
                    .filter(|n| *n < 32)
                    .ok_or_else(|| shift_range_error(actuals[1].pos(), *n))?;
                Ok(ConstValue::Int(match builtin {
                    // A logical shift: bits past the top are discarded, so
                    // LSL(1, 31) is MIN(INTEGER) and not an overflow, even
                    // though the Report's gloss reads "x * 2^n".
                    Builtin::Lsl => ((*x as u32) << count) as i32,
                    Builtin::Asr => x >> count,
                    Builtin::Ror => (*x as u32).rotate_right(count) as i32,
                    _ => unreachable!("not a shift"),
                }))
            }
            _ => unreachable!("builtin call was type-checked"),
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
            ast::Expr::Call { callee, args, pos } => match self.resolve(callee)? {
                Symbol::Builtin(builtin) => self.eval_const_builtin(builtin, callee, args, *pos),
                _ => Err(Diagnostic::new(
                    *pos,
                    "constant expression contains a procedure call",
                )),
            },
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
        // the one thing that pushes a scope; see MODULE_SCOPE. A slice that
        // pushes a scope for anything else must record the level on the
        // symbol instead, or this test quietly starts letting those variables
        // in. The universe scope holds no variables, so index 0 never reaches
        // the check.
        if scope_index != MODULE_SCOPE
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

    fn load(&mut self, addr: ir::Addr, ty: Type) -> ir::Value {
        let dst = self.temp();
        self.emit(ir::Inst::Load {
            dst,
            ty: ty.ir(),
            addr,
        });
        ir::Value::Temp(dst)
    }

    // Sets the flag on the innermost binding of `name` and returns what it
    // was, so a FOR statement can restore it rather than clearing it: a
    // nested FOR over the same variable must leave the outer one read-only.
    fn set_read_only(&mut self, name: &str, value: bool) -> bool {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(symbol) = scope.get_mut(name) {
                return match symbol {
                    Symbol::Var { read_only, .. } => std::mem::replace(read_only, value),
                    _ => false,
                };
            }
        }
        false
    }

    // A trap that ends the process: the check that guards it has already
    // branched here, so nothing follows the halt.
    fn trap(&mut self, symbol: &str) {
        self.emit(ir::Inst::Call {
            dst: None,
            symbol: symbol.into(),
            args: Vec::new(),
        });
        self.emit(ir::Inst::Halt);
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

// The temporary native Out: an interface with no Oberon source behind it,
// whose procedures are the C runtime's. The driver installs it only when no
// source module of that name is found. Slice 17 replaces it with lib/Out.Mod,
// which ordinary source lookup will then select.
pub fn out_interface() -> Interface {
    Interface {
        members: HashMap::from([
            (
                "Int".into(),
                Member::Proc {
                    symbol: "oberon_out_int".into(),
                    params: vec![(false, Type::Integer), (false, Type::Integer)],
                    ret: None,
                },
            ),
            (
                "Ln".into(),
                Member::Proc {
                    symbol: "oberon_out_ln".into(),
                    params: Vec::new(),
                    ret: None,
                },
            ),
        ]),
    }
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

// Argument types and result type of the function-like predefined operations.
// INC, DEC, and ASSERT are proper procedures and have no entry: a None here
// is what makes them "cannot be used as a value" in an expression.
fn builtin_signature(builtin: Builtin) -> Option<(&'static [Type], Type)> {
    match builtin {
        Builtin::Abs => Some((&[Type::Integer], Type::Integer)),
        Builtin::Odd => Some((&[Type::Integer], Type::Boolean)),
        Builtin::Lsl | Builtin::Asr | Builtin::Ror => {
            Some((&[Type::Integer, Type::Integer], Type::Integer))
        }
        Builtin::Ord => Some((&[Type::Boolean], Type::Integer)),
        Builtin::Inc | Builtin::Dec | Builtin::Assert => None,
    }
}

fn shift_range_error(pos: Pos, count: i32) -> Diagnostic {
    Diagnostic::new(
        pos,
        format!("shift count {count} is out of range: must be between 0 and 31"),
    )
}

fn label_text(low: i32, high: i32) -> String {
    if low == high {
        format!("case label {low}")
    } else {
        format!("case labels {low}..{high}")
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
