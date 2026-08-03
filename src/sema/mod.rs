use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::ast;
use crate::diag::{Diagnostic, Pos};
use crate::ir;

mod constant;
mod symbols;
mod types;

use symbols::{
    Builtin, ConstValue, Member, Scope, Symbol, builtin_signature, type_list, universe_scope,
};
pub use symbols::{Interface, out_interface};
use types::*;

// The scope stack is always [universe, module, outermost proc, ..., current
// proc], so this index names the module scope and its length names "the
// module scope is innermost".
const MODULE_SCOPE: usize = 1;

// Report 6.1 leaves the largest SET element implementation-defined. This
// compiler picks 31, so a SET is exactly one 32-bit bit vector and bit n
// records membership of element n. Project Oberon makes the same choice.
const SET_MAX: i32 = 31;
const SET_FULL: u32 = u32::MAX;

// Where a variable's storage is, what type it has there, and whether it may be
// written. Report 8.1 builds this by applying selectors to a variable, and
// every writable context — assignment, a VAR actual, INC and its relatives —
// uses the one that comes out.
struct Place {
    addr: ir::Addr,
    ty: Type,
    read_only: bool,
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
            // module, so a module may declare its own ABS and shadow this one
            // rather than colliding with it. ORB.Init builds the same
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
                    literals: self.literals,
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
                        Symbol::Const(value.clone()),
                    ) {
                        self.export(&declaration.id, Member::Const(value));
                    }
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
            if self.declare(
                &declaration.id.name,
                declaration.id.pos,
                Symbol::TypeName(ty.clone()),
            ) {
                self.export(&declaration.id, Member::Type(ty));
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
                        ty: ty.clone(),
                        addr,
                        read_only: false,
                    },
                ) {
                    self.globals.push(ir::Global {
                        symbol: symbol.clone(),
                        ty: ty.storage(),
                    });
                    self.export(
                        id,
                        Member::Var {
                            ty: ty.clone(),
                            symbol,
                        },
                    );
                }
            }
        }
    }

    fn procedure(&mut self, declaration: &ast::ProcDecl, prefix: &str) {
        let (proc_ty, formals, ret) =
            self.resolve_procedure_signature(&declaration.params, declaration.ret.as_ref());
        let symbol = format!("{prefix}.{}", declaration.id.name);
        if let Some(proc_ty) = proc_ty {
            if self.declare(
                &declaration.id.name,
                declaration.id.pos,
                Symbol::Proc {
                    symbol: symbol.clone(),
                    ty: proc_ty.clone(),
                },
            ) {
                self.export(
                    &declaration.id,
                    Member::Proc {
                        symbol: symbol.clone(),
                        ty: proc_ty,
                    },
                );
            }
        }

        self.scopes.push(Scope::new());
        let enclosing = self.current.take();
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
                    read_only: false,
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
                        ty: ty.clone(),
                        addr,
                        read_only: false,
                    },
                ) {
                    self.reserve_slot(&id.name, &ty);
                    // A local can never be exported, but the mark still has
                    // to be diagnosed rather than ignored.
                    self.check_export(id);
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
        let control = match self.assign_target(var) {
            Some(place) if place.ty == Type::Integer => Some(place.addr),
            Some(place) => {
                self.diags.push(Diagnostic::new(
                    var.pos,
                    format!(
                        "control variable '{}' must be INTEGER, found {}",
                        var.ident, place.ty
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
            let current = self.load(addr, ir::Ty::Int);
            // The direction is decided here, from the sign of the folded
            // step, so nothing tests the step at run time.
            let op = if step < 0 {
                ir::BinOp::Ge
            } else {
                ir::BinOp::Le
            };
            let cond = self.bin(op, ir::Ty::Int, current, limit);
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
            let current = self.load(addr.clone(), ir::Ty::Int);
            let next = self.bin(ir::BinOp::Add, ir::Ty::Int, current, ir::Value::Int(step));
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
            Ok(other) => {
                self.diags.push(Diagnostic::new(
                    expr.pos(),
                    format!("FOR step must be INTEGER, found {}", other.ty()),
                ));
                1
            }
            Err(diag) => {
                self.diags.push(diag);
                1
            }
        }
    }

    // Report 9.5. The INTEGER form compares the selector against its labels.
    fn lower_case(&mut self, expr: &ast::Expr, arms: &[ast::CaseArm]) {
        // "First the case expression is evaluated": once, into a temporary
        // that every arm's test then compares against.
        let selector = match self.lower_expr(expr) {
            Some((value, Type::Integer)) => Some(value),
            Some((_, ty)) => {
                self.diags.push(Diagnostic::new(
                    expr.pos(),
                    format!("CASE expression must be INTEGER, found {ty}"),
                ));
                None
            }
            None => None,
        };

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
                    self.bin(
                        ir::BinOp::Eq,
                        ir::Ty::Int,
                        selector.clone(),
                        ir::Value::Int(*low),
                    )
                } else {
                    // Comparisons yield 0 or 1, so a bitwise and is the
                    // conjunction of the two bounds tests.
                    let above = self.bin(
                        ir::BinOp::Ge,
                        ir::Ty::Int,
                        selector.clone(),
                        ir::Value::Int(*low),
                    );
                    let below = self.bin(
                        ir::BinOp::Le,
                        ir::Ty::Int,
                        selector.clone(),
                        ir::Value::Int(*high),
                    );
                    self.bin(ir::BinOp::BitAnd, ir::Ty::Bool, above, below)
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
        self.trap("oberon_case_no_match", expr.pos());
        self.emit(ir::Inst::Label(end));
    }

    // Report 9.5: under an INTEGER selector every label is an integer.
    fn case_label(&mut self, expr: &ast::Expr) -> Option<i32> {
        match self.eval_const(expr) {
            Ok(value) => match value {
                ConstValue::Int(value) => Some(value),
                other => {
                    self.diags.push(Diagnostic::new(
                        expr.pos(),
                        format!("case label must be INTEGER, found {}", other.ty()),
                    ));
                    None
                }
            },
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
            ast::Expr::Set { elements, .. } => self.lower_set(elements),
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
                    // Report 8.2.3: unary "-" on a SET is the complement.
                    // Every one of the 32 bits belongs to the domain, so the
                    // result cannot name an element outside it.
                    (ast::UnOp::Neg, Type::Set) => Some((
                        self.bin(
                            ir::BinOp::BitXor,
                            ir::Ty::Set,
                            arg,
                            ir::Value::Set(SET_FULL),
                        ),
                        Type::Set,
                    )),
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
                ast::BinOp::In => self.lower_membership(lhs, rhs, *pos),
                _ => self.lower_binary(*op, lhs, rhs, *pos),
            },
        }
    }

    // Report 8.2: {} is empty, {m} holds m, and {m .. n} holds m through n
    // and is empty when m > n. Elements are evaluated once each, left to
    // right, and their masks are combined with union.
    fn lower_set(&mut self, elements: &[ast::SetElement]) -> Option<(ir::Value, Type)> {
        let mut result = None;
        let mut ok = true;
        for element in elements {
            let low = self.lower_set_element(&element.low);
            let mask = match &element.high {
                None => low.map(|low| self.set_singleton(low)),
                // The high endpoint is lowered even when the low one failed,
                // so an invalid range reports both of its errors.
                Some(high) => {
                    let high = self.lower_set_element(high);
                    match (low, high) {
                        (Some(low), Some(high)) => Some(self.set_range(low, high)),
                        _ => None,
                    }
                }
            };
            let Some(mask) = mask else {
                ok = false;
                continue;
            };
            result = Some(match result {
                None => mask,
                Some(acc) => self.set_union(acc, mask),
            });
        }
        ok.then(|| (result.unwrap_or(ir::Value::Set(0)), Type::Set))
    }

    fn set_singleton(&mut self, element: ir::Value) -> ir::Value {
        match element {
            ir::Value::Int(n) => ir::Value::Set(1 << n),
            _ => self.bin(ir::BinOp::Shl, ir::Ty::Set, ir::Value::Int(1), element),
        }
    }

    // The bits at or above the low endpoint, intersected with the bits at or
    // below the high one. That is empty exactly when the range is reversed,
    // so the Report's rule for {m .. n} with m > n needs no branch of its
    // own, and no endpoint comparison is generated. The shift counts are
    // `low` and `31 - high`, both already checked to lie in 0 .. 31, so
    // neither can reach the word width.
    fn set_range(&mut self, low: ir::Value, high: ir::Value) -> ir::Value {
        if let (ir::Value::Int(low), ir::Value::Int(high)) = (&low, &high) {
            return ir::Value::Set(set_range_bits(*low, *high));
        }
        let above = self.bin(ir::BinOp::Shl, ir::Ty::Set, ir::Value::Set(SET_FULL), low);
        let drop = self.bin(ir::BinOp::Sub, ir::Ty::Int, ir::Value::Int(SET_MAX), high);
        let below = self.bin(ir::BinOp::Shr, ir::Ty::Set, ir::Value::Set(SET_FULL), drop);
        self.bin(ir::BinOp::BitAnd, ir::Ty::Set, above, below)
    }

    fn set_union(&mut self, lhs: ir::Value, rhs: ir::Value) -> ir::Value {
        match (lhs, rhs) {
            (ir::Value::Set(lhs), ir::Value::Set(rhs)) => ir::Value::Set(lhs | rhs),
            (lhs, rhs) => self.bin(ir::BinOp::BitOr, ir::Ty::Set, lhs, rhs),
        }
    }

    fn lower_set_element(&mut self, expr: &ast::Expr) -> Option<ir::Value> {
        let value = self.lower_int(expr, "set element")?;
        self.check_set_element(expr, value)
    }

    // The one domain rule, shared by constructors, ranges, IN, INCL, and
    // EXCL. A constant is diagnosed here rather than deferred to run time,
    // and folding is what makes `{16 + 16}` as wrong as `{32}`.
    fn check_set_element(&mut self, expr: &ast::Expr, value: ir::Value) -> Option<ir::Value> {
        match self.try_eval_const(expr) {
            Ok(Some(ConstValue::Int(n))) => {
                if !(0..=SET_MAX).contains(&n) {
                    self.diags.push(set_element_range_error(expr.pos(), n));
                    return None;
                }
                return Some(ir::Value::Int(n));
            }
            Ok(Some(_)) => unreachable!("set element was type-checked as INTEGER"),
            Ok(None) => {}
            Err(diag) => {
                self.diags.push(diag);
                return None;
            }
        }
        // QBE reduces a shift count modulo the word width, so an unchecked
        // 32 would quietly behave like 0.
        let low = self.bin(ir::BinOp::Lt, ir::Ty::Int, value.clone(), ir::Value::Int(0));
        let high = self.bin(
            ir::BinOp::Gt,
            ir::Ty::Int,
            value.clone(),
            ir::Value::Int(SET_MAX),
        );
        let bad = self.bin(ir::BinOp::BitOr, ir::Ty::Bool, low, high);
        let trap = self.label("set.bad");
        let ok = self.label("set.ok");
        self.emit(ir::Inst::Br {
            cond: bad,
            then: trap.clone(),
            els: ok.clone(),
        });
        self.emit(ir::Inst::Label(trap));
        self.trap("oberon_set_element_range", expr.pos());
        self.emit(ir::Inst::Label(ok));
        Some(value)
    }

    // Report 8.2: x IN s. The element takes the same domain check as one
    // written in a constructor, because the test lowers to a shift too.
    fn lower_membership(
        &mut self,
        lhs: &ast::Expr,
        rhs: &ast::Expr,
        pos: Pos,
    ) -> Option<(ir::Value, Type)> {
        let element = self.lower_expr(lhs);
        let set = self.lower_expr(rhs);
        let (Some((element, element_ty)), Some((set, set_ty))) = (element, set) else {
            return None;
        };
        if element_ty != Type::Integer || set_ty != Type::Set {
            self.diags.push(Diagnostic::new(
                pos,
                format!("operator 'IN' requires INTEGER and SET, found {element_ty} and {set_ty}"),
            ));
            return None;
        }
        let element = self.check_set_element(lhs, element)?;
        let bit = self.set_singleton(element);
        let masked = self.bin(ir::BinOp::BitAnd, ir::Ty::Set, set, bit);
        Some((
            self.bin(ir::BinOp::Ne, ir::Ty::Set, masked, ir::Value::Set(0)),
            Type::Boolean,
        ))
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
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Slash => {
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
            | BinOp::In
            | BinOp::And
            | BinOp::Or => unreachable!(),
        };

        // Report 8.2: for SET operands "+", "-", "*", and "/" mean union,
        // difference, intersection, and symmetric difference. Difference
        // clears the right-hand bits, so it is an intersection with the
        // complement and never a subtraction.
        if result_ty == Type::Set {
            return Some((
                match op {
                    BinOp::Add => self.bin(ir::BinOp::BitOr, ir::Ty::Set, lhs, rhs_ir),
                    BinOp::Sub => {
                        let keep = self.bin(
                            ir::BinOp::BitXor,
                            ir::Ty::Set,
                            rhs_ir,
                            ir::Value::Set(SET_FULL),
                        );
                        self.bin(ir::BinOp::BitAnd, ir::Ty::Set, lhs, keep)
                    }
                    BinOp::Mul => self.bin(ir::BinOp::BitAnd, ir::Ty::Set, lhs, rhs_ir),
                    BinOp::Slash => self.bin(ir::BinOp::BitXor, ir::Ty::Set, lhs, rhs_ir),
                    _ => unreachable!("only the four set operators yield SET"),
                },
                Type::Set,
            ));
        }

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

    // Report 8.2.4. The relations order INTEGER, and compare SET and BOOLEAN
    // for equality.
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
            Ok((Symbol::Builtin(builtin), [])) => {
                return self.lower_builtin(builtin, actuals, pos);
            }
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
                // A VAR actual will be written, so a read-only designator may
                // not be passed.
                match self.ref_actual(actual, i + 1, var) {
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

    fn lower_builtin(
        &mut self,
        builtin: Builtin,
        actuals: &[ast::Expr],
        pos: Pos,
    ) -> Option<(Option<ir::Value>, Option<Type>)> {
        match builtin {
            Builtin::Inc | Builtin::Dec => return self.lower_inc_dec(builtin, actuals, pos),
            Builtin::Incl | Builtin::Excl => return self.lower_incl_excl(builtin, actuals, pos),
            Builtin::Assert => return self.lower_assert(actuals, pos),
            _ => {}
        }

        let (params, result) = builtin_signature(builtin).expect("function-like builtin");
        if !self.builtin_arity(actuals, params.len(), pos) {
            return None;
        }
        let mut args = Vec::new();
        let mut ok = true;
        for (i, (actual, expected)) in actuals.iter().zip(params).enumerate() {
            // Every argument is lowered even after one of them fails, so a
            // call reports all of its type errors rather than only the first.
            match self.builtin_arg(actual, i + 1, expected) {
                Some(arg) => args.push(arg),
                None => ok = false,
            }
        }
        if !ok {
            return None;
        }

        let value = match builtin {
            Builtin::Abs => self.lower_abs_int(args[0].0.clone(), pos),
            // Truncating remainder, so the test must be against zero and not
            // against one: -3 rem 2 is -1. This agrees with the Report's
            // "x MOD 2 = 1" under the floored MOD the compiler implements.
            Builtin::Odd => {
                let rem = self.bin(
                    ir::BinOp::Rem,
                    ir::Ty::Int,
                    args[0].0.clone(),
                    ir::Value::Int(2),
                );
                self.bin(ir::BinOp::Ne, ir::Ty::Int, rem, ir::Value::Int(0))
            }
            // BOOLEAN is already 0 or 1 in a word and a SET is already its
            // own bit pattern, so the value passes through with only its type
            // changed. cf. Project Oberon, where ORD lowers to nothing.
            Builtin::Ord => args[0].0.clone(),
            Builtin::Lsl | Builtin::Asr | Builtin::Ror => {
                self.lower_shift(builtin, args[0].0.clone(), args[1].0.clone(), &actuals[1])?
            }
            Builtin::Inc | Builtin::Dec | Builtin::Incl | Builtin::Excl | Builtin::Assert => {
                unreachable!("handled above")
            }
        };
        Some((Some(value), Some(result.ty(args[0].1.clone()))))
    }

    // The argument of a predefined operation that changes it: a variable of
    // exactly one type, writable through the same designator path as INC and
    // therefore equally unable to reach an imported variable.
    fn modified_actual(
        &mut self,
        actual: &ast::Expr,
        number: usize,
        expected: Type,
    ) -> Option<ir::Addr> {
        match self.var_actual(actual, number) {
            Some((addr, found)) if found == expected => Some(addr),
            Some((_, found)) => {
                self.diags.push(Diagnostic::new(
                    actual.pos(),
                    format!("argument {number} has type {found}, expected {expected}"),
                ));
                None
            }
            None => None,
        }
    }

    // MIN(INTEGER) has no absolute value. Folding rejects it, so the runtime
    // form must not quietly wrap the way unary minus still does.
    fn lower_abs_int(&mut self, arg: ir::Value, pos: Pos) -> ir::Value {
        let overflows = self.bin(
            ir::BinOp::Eq,
            ir::Ty::Int,
            arg.clone(),
            ir::Value::Int(i32::MIN),
        );
        let bad = self.label("abs.bad");
        let ok = self.label("abs.ok");
        self.emit(ir::Inst::Br {
            cond: overflows,
            then: bad.clone(),
            els: ok.clone(),
        });
        self.emit(ir::Inst::Label(bad));
        self.trap("oberon_abs_overflow", pos);
        self.emit(ir::Inst::Label(ok));

        let negative = self.bin(ir::BinOp::Lt, ir::Ty::Int, arg.clone(), ir::Value::Int(0));
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
            ty: ir::Ty::Int,
            arg: arg.clone(),
        });
        self.emit(ir::Inst::Copy {
            dst: result,
            ty: ir::Ty::Int,
            src: ir::Value::Temp(negated),
        });
        self.emit(ir::Inst::Jmp(end.clone()));
        self.emit(ir::Inst::Label(keep));
        self.emit(ir::Inst::Copy {
            dst: result,
            ty: ir::Ty::Int,
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
        value: ir::Value,
        count: ir::Value,
        count_expr: &ast::Expr,
    ) -> Option<ir::Value> {
        match &count {
            ir::Value::Int(n) => {
                if !(0..=31).contains(n) {
                    self.diags.push(shift_range_error(count_expr.pos(), *n));
                    return None;
                }
            }
            _ => {
                let low = self.bin(ir::BinOp::Lt, ir::Ty::Int, count.clone(), ir::Value::Int(0));
                let high = self.bin(
                    ir::BinOp::Gt,
                    ir::Ty::Int,
                    count.clone(),
                    ir::Value::Int(31),
                );
                let bad = self.bin(ir::BinOp::BitOr, ir::Ty::Bool, low, high);
                let trap = self.label("shift.bad");
                let ok = self.label("shift.ok");
                self.emit(ir::Inst::Br {
                    cond: bad,
                    then: trap.clone(),
                    els: ok.clone(),
                });
                self.emit(ir::Inst::Label(trap));
                self.trap("oberon_shift_range", count_expr.pos());
                self.emit(ir::Inst::Label(ok));
            }
        }
        Some(match builtin {
            Builtin::Lsl => self.bin(ir::BinOp::Shl, ir::Ty::Int, value, count),
            Builtin::Asr => self.bin(ir::BinOp::Sar, ir::Ty::Int, value, count),
            // A logical right shift merged with the bits that fall off the
            // bottom. The mask makes a rotation by zero the identity instead
            // of a shift by the word width.
            Builtin::Ror => {
                let right = self.bin(ir::BinOp::Shr, ir::Ty::Int, value.clone(), count.clone());
                let complement = self.bin(ir::BinOp::Sub, ir::Ty::Int, ir::Value::Int(32), count);
                let left_count = self.bin(
                    ir::BinOp::BitAnd,
                    ir::Ty::Int,
                    complement,
                    ir::Value::Int(31),
                );
                let left = self.bin(ir::BinOp::Shl, ir::Ty::Int, value, left_count);
                self.bin(ir::BinOp::BitOr, ir::Ty::Int, right, left)
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
        // The argument is an integer variable.
        let target = match self.var_actual(&actuals[0], 1) {
            Some((addr, ty @ Type::Integer)) => Some((addr, ty)),
            Some((_, found)) => {
                self.diags.push(Diagnostic::new(
                    actuals[0].pos(),
                    format!("argument 1 has type {found}, expected INTEGER"),
                ));
                None
            }
            None => None,
        };
        let step = match actuals.get(1) {
            Some(actual) => self
                .builtin_arg(actual, 2, &[Type::Integer])
                .map(|(value, _)| value),
            None => Some(ir::Value::Int(1)),
        };
        if let (Some((addr, ty)), Some(step)) = (target, step) {
            let scalar = ty.ir();
            let current = self.load(addr.clone(), scalar);
            let op = if builtin == Builtin::Inc {
                ir::BinOp::Add
            } else {
                ir::BinOp::Sub
            };
            let next = self.bin(op, ir::Ty::Int, current, step);
            self.emit(ir::Inst::Store {
                ty: scalar,
                val: next,
                addr,
            });
        }
        Some((None, None))
    }

    // Report 10.2: INCL(v, x) is v := v + {x} and EXCL(v, x) is v := v - {x}.
    // Both are proper procedures taking a writable SET variable, so they go
    // through the same designator path as INC and DEC and are equally unable
    // to change an imported variable.
    fn lower_incl_excl(
        &mut self,
        builtin: Builtin,
        actuals: &[ast::Expr],
        pos: Pos,
    ) -> Option<(Option<ir::Value>, Option<Type>)> {
        if !self.builtin_arity(actuals, 2, pos) {
            return None;
        }
        let target = self.modified_actual(&actuals[0], 1, Type::Set);
        let element = self
            .builtin_arg(&actuals[1], 2, &[Type::Integer])
            .and_then(|(value, _)| self.check_set_element(&actuals[1], value));
        if let (Some(addr), Some(element)) = (target, element) {
            let bit = self.set_singleton(element);
            let current = self.load(addr.clone(), ir::Ty::Set);
            let next = if builtin == Builtin::Incl {
                self.bin(ir::BinOp::BitOr, ir::Ty::Set, current, bit)
            } else {
                let keep = self.bin(
                    ir::BinOp::BitXor,
                    ir::Ty::Set,
                    bit,
                    ir::Value::Set(SET_FULL),
                );
                self.bin(ir::BinOp::BitAnd, ir::Ty::Set, current, keep)
            };
            self.emit(ir::Inst::Store {
                ty: ir::Ty::Set,
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
        if let Some((cond, _)) = self.builtin_arg(&actuals[0], 1, &[Type::Boolean]) {
            let bad = self.label("assert.bad");
            let ok = self.label("assert.ok");
            self.emit(ir::Inst::Br {
                cond,
                then: ok.clone(),
                els: bad.clone(),
            });
            self.emit(ir::Inst::Label(bad));
            self.trap("oberon_assert_failed", pos);
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

    // The found type comes back with the value because ABS is generic: its
    // result is whichever of the accepted types the argument turned out to be.
    fn builtin_arg(
        &mut self,
        actual: &ast::Expr,
        number: usize,
        expected: &[Type],
    ) -> Option<(ir::Value, Type)> {
        let (value, found) = self.lower_expr(actual)?;
        if expected.contains(&found) {
            Some((value, found))
        } else {
            self.diags.push(Diagnostic::new(
                actual.pos(),
                format!(
                    "argument {number} has type {found}, expected {}",
                    type_list(expected)
                ),
            ));
            None
        }
    }

    // Report 9.2: the selectors of a VAR actual are evaluated when the
    // parameter is substituted, which is exactly once, before the call. That
    // is what resolving the designator to an address here already does, so a
    // selected element reaches a VAR parameter with no extra machinery.
    fn var_actual(&mut self, actual: &ast::Expr, number: usize) -> Option<(ir::Addr, Type)> {
        self.ref_actual(actual, number, true)
            .map(|place| (place.addr, place.ty))
    }

    // The address an actual supplies to a reference formal.
    fn ref_actual(&mut self, actual: &ast::Expr, number: usize, writable: bool) -> Option<Place> {
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
        if writable && place.read_only {
            self.diags.push(Diagnostic::new(
                actual.pos(),
                format!("argument {number} is read-only"),
            ));
            return None;
        }
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
        let Symbol::Var {
            ty,
            addr,
            read_only,
        } = symbol
        else {
            self.diags
                .push(Diagnostic::new(designator.pos, not_a_variable));
            return None;
        };
        let mut place = Place {
            addr,
            ty,
            read_only,
        };
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
        let place = self.place(
            designator,
            format!("cannot assign to '{}'", designator.name()),
        )?;
        if place.read_only {
            self.diags.push(Diagnostic::new(
                designator.pos,
                format!("cannot assign to '{}': it is read-only", designator.name()),
            ));
            return None;
        }
        Some(place)
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
        let (scope_index, symbol) = self
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

fn shift_range_error(pos: Pos, count: i32) -> Diagnostic {
    Diagnostic::new(
        pos,
        format!("shift count {count} is out of range: must be between 0 and 31"),
    )
}

fn set_element_range_error(pos: Pos, element: i32) -> Diagnostic {
    Diagnostic::new(
        pos,
        format!("set element {element} is out of range: must be between 0 and {SET_MAX}"),
    )
}

// The bits of {low .. high}, which is empty when the range is reversed. The
// runtime form in Analyzer::set_range computes the same intersection.
fn set_range_bits(low: i32, high: i32) -> u32 {
    (SET_FULL << low) & (SET_FULL >> (SET_MAX - high))
}

fn label_text(low: i32, high: i32) -> String {
    if low == high {
        format!("case label {low}")
    } else {
        format!("case labels {low}..{high}")
    }
}

fn unary_type_error(pos: Pos, op: ast::UnOp, found: &Type) -> Diagnostic {
    let (name, expected) = match op {
        ast::UnOp::Plus => ("+", "INTEGER"),
        ast::UnOp::Neg => ("-", "INTEGER or SET"),
        ast::UnOp::Not => ("~", "BOOLEAN"),
    };
    Diagnostic::new(
        pos,
        format!("operator '{name}' requires {expected}, found {found}"),
    )
}
