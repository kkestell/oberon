use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::ast;
use crate::diag::{Diagnostic, Pos};
use crate::ir;

mod constant;
mod symbols;
mod types;

use constant::{floor_const, relation_holds, str_const_cmp};
use symbols::{
    Builtin, ConstValue, Member, RecordDynamic, Scope, Symbol, builtin_signature, type_list,
    universe_scope,
};
pub use symbols::{Interface, runtime_interface};
use types::*;

// The scope stack is always [universe, module, outermost proc, ..., current
// proc], so this index names the module scope and its length names "the
// module scope is innermost".
const MODULE_SCOPE: usize = 1;

// Report 6.1 leaves the largest SET element implementation-defined. This
// compiler picks 63, so a SET is exactly one 64-bit bit vector and bit n
// records membership of element n, matching the INTEGER width as Project
// Oberon's 31 matches its 32-bit word.
const SET_MAX: i64 = 63;
const SET_FULL: u64 = u64::MAX;

// Report 10.2: FLOOR yields the largest INTEGER not greater than its
// argument, so an argument only has a result while one exists. Both endpoints
// are exact binary64 values, and the upper one is excluded because it is
// MAX(INTEGER) + 1. The runtime check in oberon_floor uses the same pair.
const FLOOR_MIN: f64 = -9223372036854775808.0;
const FLOOR_LIMIT: f64 = 9223372036854775808.0;

// Where a variable's storage is, what type it has there, and whether it may be
// written. Report 8.1 builds this by applying selectors to a variable, and
// every writable context — assignment, a VAR actual, INC and its relatives —
// uses the one that comes out.
struct Place {
    addr: ir::Addr,
    ty: Type,
    // One length for every array constructor reachable by indexing from this
    // place. Open dimensions hold incoming temporaries; fixed dimensions hold
    // immediates. Indexing consumes the first entry.
    shape: Vec<ir::Value>,
    read_only: bool,
    dynamic: Option<RecordDynamic>,
}

type ResolvedFormal = (bool, String, Pos, Type);
type ResolvedProcedure = (Option<Type>, Vec<ResolvedFormal>, Option<Type>);

// What the source of an assignment or a relation operand turned out to be.
// An array or a record keeps its address because it has no scalar value to
// load, and a string keeps its bytes because what it becomes — a CHAR, a copy
// into a character array, one side of a comparison — depends on the context.
enum Source {
    Value(ir::Value, Type),
    Structured(Place),
    Str(Rc<Vec<u8>>),
}

#[derive(Clone, Copy, PartialEq)]
enum ApplicationKind {
    Call,
    Guard,
    Invalid,
}

impl Source {
    // The type a diagnostic reports for this source.
    fn ty(&self) -> Type {
        match self {
            Source::Value(_, ty) => ty.clone(),
            Source::Structured(place) => place.ty.clone(),
            Source::Str(bytes) => Type::String(bytes.len()),
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
    descriptors: Vec<ir::Descriptor>,
    // The aligned bytes the module's globals have reserved so far, so the
    // declaration that would take the data object past the target limit is the
    // one that reports it.
    globals_size: i64,
    literals: Vec<ir::Literal>,
    // The literal holding the module's name, which every runtime check
    // reports alongside its position.
    site_module: String,
    procs: Vec<ir::Proc>,
    interface: Interface,
    current: Option<ProcBuilder>,
    pending_pointers: Vec<Rc<PointerType>>,
    allow_pointer_forward: bool,
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
            descriptors: Vec::new(),
            globals_size: 0,
            literals: Vec::new(),
            site_module: String::new(),
            procs: Vec::new(),
            interface: Interface::default(),
            current: None,
            pending_pointers: Vec::new(),
            allow_pointer_forward: false,
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
                    descriptors: self.descriptors,
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
        let pending_start = self.pending_pointers.len();
        let previous = std::mem::replace(&mut self.allow_pointer_forward, true);
        for declaration in declarations {
            let Some(ty) = self.resolve_type_named(&declaration.ty, Some(&declaration.id.name))
            else {
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
        self.allow_pointer_forward = previous;
        self.resolve_pending_pointers(pending_start);
    }

    fn resolve_pending_pointers(&mut self, start: usize) {
        let pending: Vec<_> = self.pending_pointers.drain(start..).collect();
        for pointer in pending {
            let (name, pos) = match &*pointer.base.borrow() {
                PointerBase::Pending { name, pos } => (name.clone(), *pos),
                _ => continue,
            };
            let resolved = self
                .scopes
                .last()
                .and_then(|scope| scope.get(&name))
                .cloned();
            match resolved {
                Some(Symbol::TypeName(Type::Record(record))) => {
                    *pointer.base.borrow_mut() = PointerBase::Resolved(record);
                }
                Some(Symbol::TypeName(ty)) => {
                    self.diags.push(Diagnostic::new(
                        pos,
                        format!("pointer base must be a record type, found {ty}"),
                    ));
                    *pointer.base.borrow_mut() = PointerBase::Invalid;
                }
                Some(_) => {
                    self.diags
                        .push(Diagnostic::new(pos, format!("'{name}' is not a type")));
                    *pointer.base.borrow_mut() = PointerBase::Invalid;
                }
                None => {
                    self.diags.push(Diagnostic::new(
                        pos,
                        format!("undeclared identifier '{name}'"),
                    ));
                    *pointer.base.borrow_mut() = PointerBase::Invalid;
                }
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
                        shape: fixed_shape(&ty),
                        read_only: false,
                        dynamic: ty
                            .record()
                            .map(|record| RecordDynamic::Static(record.descriptor.clone())),
                    },
                ) {
                    match reserve(self.globals_size, &ty) {
                        Some(total) => self.globals_size = total,
                        None => self.diags.push(Diagnostic::new(
                            id.pos,
                            "module storage exceeds target object-size limit",
                        )),
                    }
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
            let eligible = self.scopes.len() == MODULE_SCOPE + 1;
            if self.declare(
                &declaration.id.name,
                declaration.id.pos,
                Symbol::Proc {
                    symbol: symbol.clone(),
                    ty: proc_ty.clone(),
                    eligible,
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
            // A structured parameter is a reference whichever kind it is:
            // Report 10.1 confines "the formal is a local variable holding
            // the value" to basic types, and 9.1 forbids assigning to a
            // structured value parameter or its elements. That pairing
            // licenses passing the address and copying nothing, and all
            // three reference compilers do exactly that — Project Oberon
            // reclassifies the parameter as a read-only reference, OBNC
            // emits a const pointer, oberonc passes the JVM reference. The
            // consequence is that aliasing is observable, which
            // ParamAlias.Mod pins.
            let by_ref = var || ty.structured();
            let temp = self.builder().temp();
            self.builder().proc.params.push(ir::Param {
                temp,
                pass: if by_ref {
                    ir::ParamPass::Ref
                } else {
                    ir::ParamPass::Value(ty.ir())
                },
            });
            let mut open_lengths = Vec::new();
            let dynamic = if var {
                ty.record().map(|_| {
                    let descriptor = self.builder().temp();
                    self.builder().proc.params.push(ir::Param {
                        temp: descriptor,
                        pass: ir::ParamPass::Value(ir::Ty::Pointer),
                    });
                    RecordDynamic::Incoming(ir::Value::Temp(descriptor))
                })
            } else {
                None
            };
            for _ in 0..ty.open_rank() {
                let length = self.builder().temp();
                self.builder().proc.params.push(ir::Param {
                    temp: length,
                    pass: ir::ParamPass::Value(ir::Ty::Int),
                });
                open_lengths.push(ir::Value::Temp(length));
            }
            let addr = if by_ref {
                ir::Addr::Temp(temp)
            } else {
                ir::Addr::Slot(name.clone())
            };
            let shape = shape_with_open_lengths(&ty, &open_lengths);
            if self.declare(
                &name,
                pos,
                Symbol::Var {
                    ty: ty.clone(),
                    addr: addr.clone(),
                    shape,
                    // Read-only in its entirety, all the way down: the place
                    // walk carries the flag through every field and element.
                    read_only: !var && ty.structured(),
                    dynamic,
                },
            ) && !by_ref
            {
                self.reserve_slot(&name, &ty, pos);
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
                        shape: fixed_shape(&ty),
                        read_only: false,
                        dynamic: ty
                            .record()
                            .map(|record| RecordDynamic::Static(record.descriptor.clone())),
                    },
                ) {
                    self.reserve_slot(&id.name, &ty, id.pos);
                    // A local can never be exported, but the mark still has
                    // to be diagnosed rather than ignored.
                    self.check_export(id);
                }
            }
        }
    }

    // One local's storage, plus its share of the running frame total. Several
    // individually valid locals must not add up to an activation record the
    // target cannot address, so the declaration that crosses the limit is the
    // one that reports it.
    fn reserve_slot(&mut self, name: &str, ty: &Type, pos: Pos) {
        match reserve(self.builder().frame, ty) {
            Some(total) => self.builder().frame = total,
            None => self.diags.push(Diagnostic::new(
                pos,
                "procedure storage exceeds target object-size limit",
            )),
        }
        self.builder()
            .proc
            .slots
            .push((name.to_string(), ty.storage()));
    }

    fn lower_return(&mut self, declaration: &ast::ProcDecl, ret: Option<Type>) {
        match (&declaration.ret, ret, &declaration.ret_val) {
            (Some(_), Some(expected), Some(expr)) => {
                // RETURN asks the same compatibility function as assignment.
                // A single-character string already lowered to a CHAR, so
                // only the checked BYTE result appears here.
                let value = self.lower_expr(expr);
                if let Some((value, found)) = value {
                    let value = match assign_kind(&expected, &found) {
                        Some(AssignKind::Store) => Some(value),
                        Some(AssignKind::ByteRange) => {
                            self.check_byte_domain(Some(expr), value, ByteDomain::Store, expr.pos())
                        }
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
            ast::Stmt::Assign { lhs, rhs, pos } => self.lower_assign(lhs, rhs, *pos),
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

    // Report 9.1. A scalar assignment stores one value, identical array types
    // copy the whole representation, and the exceptions connect strings with
    // CHAR and with character arrays. assign_kind names the outcomes, and the
    // value parameter and the RETURN expression ask the same function.
    fn lower_assign(&mut self, lhs: &ast::Designator, rhs: &ast::Expr, pos: Pos) {
        // The destination designator is resolved first and the source second,
        // each exactly once, so both sides' index expressions run in source
        // order and a selected row is copied from where it was when the
        // statement started.
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
            (Some(AssignKind::ByteRange), Source::Value(value, _)) => {
                if let Some(value) =
                    self.check_byte_domain(Some(rhs), value, ByteDomain::Store, rhs.pos())
                {
                    self.emit(ir::Inst::Store {
                        ty: ir::Ty::Byte,
                        val: value,
                        addr: target.addr,
                    });
                }
            }
            (Some(AssignKind::CharFromString), Source::Str(bytes)) => {
                self.emit(ir::Inst::Store {
                    ty: ir::Ty::Byte,
                    val: ir::Value::Byte(bytes[0]),
                    addr: target.addr,
                });
            }
            // The fixed and open destinations differ only in where the fit is
            // decided, which copy_string reads off the destination type.
            (Some(AssignKind::StringCopy | AssignKind::OpenStringCopy), Source::Str(bytes)) => {
                self.copy_string(&bytes, target, rhs.pos());
            }
            (
                Some(AssignKind::WholeCopy | AssignKind::RecordPrefixCopy),
                Source::Structured(source),
            ) => {
                // A zero-length array or an empty record still resolved both
                // designators and ran both sides' checks; only the byte count
                // is zero. A record's count includes its padding, which no
                // program can observe either way.
                self.emit(ir::Inst::CopyBytes {
                    dst: target.addr,
                    src: source.addr,
                    size: target.ty.size(),
                });
            }
            (Some(AssignKind::OpenPrefixCopy), Source::Structured(source)) => {
                let source_len = source.shape[0].clone();
                let destination_len = target.shape[0].clone();
                let site = self.site(pos);
                self.emit(ir::Inst::CheckArrayCopy {
                    source_len: source_len.clone(),
                    destination_len,
                    site,
                });
                let Type::Array(array) = &target.ty else {
                    unreachable!("an open prefix copies into a fixed array")
                };
                let temp = self.temp();
                self.emit(ir::Inst::CopyElements {
                    temp,
                    dst: target.addr,
                    src: source.addr,
                    count: source_len,
                    stride: array.elem.size(),
                });
            }
            (Some(_), _) => unreachable!("assign_kind agrees with the source's shape"),
            (None, source) => {
                self.diags
                    .push(assign_error(rhs.pos(), &target.ty, &source.ty()));
            }
        }
    }

    // Report 9.1: a string may be assigned to any array of characters
    // provided the number of characters is less than the length of the array,
    // and a null character is appended. The rule is checked here, so a string
    // that leaves no room for the terminator never becomes a runtime failure.
    // The copy moves exactly the characters and one null and leaves the rest
    // of the destination untouched; cf. Project Oberon, whose word-at-a-time
    // copy can write up to three bytes past the terminator.
    fn copy_string(&mut self, bytes: &[u8], target: Place, pos: Pos) {
        assert!(
            target.ty.char_array(),
            "assign_kind chose a character array"
        );
        let count = i64::try_from(bytes.len() + 1).expect("a string fits the source file");
        if let Type::Array(array) = &target.ty {
            if count > array.len {
                self.diags.push(Diagnostic::new(
                    pos,
                    format!(
                        "a {} and its null terminator do not fit in {}",
                        Type::String(bytes.len()),
                        target.ty
                    ),
                ));
                return;
            }
        } else {
            let site = self.site(pos);
            self.emit(ir::Inst::CheckArrayCopy {
                source_len: ir::Value::Int(count),
                destination_len: target.shape[0].clone(),
                site,
            });
        }
        let src = self.literal(bytes);
        self.emit(ir::Inst::CopyBytes {
            dst: target.addr,
            src,
            size: count,
        });
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
    // evaluated exactly once. An array designator has no scalar value, so it
    // stays an address here rather than being rejected, and a string stays
    // its bytes: whole-array assignment, string assignment, and the character
    // array relations are the contexts that want them.
    fn lower_source(&mut self, expr: &ast::Expr) -> Option<Source> {
        if let ast::Expr::Str { bytes, .. } = expr {
            return Some(Source::Str(Rc::new(bytes.clone())));
        }
        if let ast::Expr::Apply { callee, args, pos } = expr
            && self.application_kind(callee) == ApplicationKind::Guard
        {
            let place = self.terminal_guard_place(
                callee,
                args,
                *pos,
                format!("'{}' cannot be used as a value", callee.name()),
            )?;
            return Some(match place.ty.scalar() {
                Some(ty) => Source::Value(self.load(place.addr, ty), place.ty),
                None => Source::Structured(place),
            });
        }
        let ast::Expr::Name(designator) = expr else {
            let (value, ty) = self.lower_expr(expr)?;
            return Some(Source::Value(value, ty));
        };
        // A constant is not storage, so it is recognized before the designator
        // is resolved as a variable.
        match self.qualident(designator) {
            Ok((Symbol::Const(ConstValue::Str(bytes)), [])) => return Some(Source::Str(bytes)),
            Ok((Symbol::Const(value), [])) => return Some(Source::Value(value.ir(), value.ty())),
            Ok((
                Symbol::Proc {
                    symbol,
                    ty,
                    eligible: true,
                },
                [],
            )) => {
                return Some(Source::Value(ir::Value::Symbol(symbol), ty));
            }
            Ok((
                Symbol::Proc {
                    eligible: false, ..
                },
                [],
            )) => {
                self.diags.push(Diagnostic::new(
                    designator.pos,
                    format!(
                        "procedure '{}' cannot be used as a value: only a globally declared procedure is eligible",
                        designator.name()
                    ),
                ));
                return None;
            }
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
        Some(match place.ty.scalar() {
            Some(ty) => {
                let value = self.load(place.addr, ty);
                // Report 6.1: BYTE is compatible with INTEGER. Reading a BYTE
                // yields an INTEGER — the load already zero-extended it — so
                // everything downstream is ordinary INTEGER behaviour.
                let read_ty = if place.ty == Type::Byte {
                    Type::Integer
                } else {
                    place.ty
                };
                Source::Value(value, read_ty)
            }
            None => Source::Structured(place),
        })
    }

    // A source where the context needs one scalar value. `expr` is the source
    // text the value came from, for the diagnostics.
    fn source_scalar(&mut self, expr: &ast::Expr, source: Source) -> Option<(ir::Value, Type)> {
        match source {
            Source::Value(value, ty) => Some((value, ty)),
            // Report 8: an expression operates on values, and an array or
            // record designator has none. That is the whole rejection of
            // records in relations, in arithmetic, in conditions, in a CASE
            // selector, and in every other value context.
            Source::Structured(place) => {
                let ast::Expr::Name(designator) = expr else {
                    unreachable!("only a designator resolves to structured storage");
                };
                self.diags.push(Diagnostic::new(
                    designator.pos,
                    format!(
                        "'{}' has type {} and cannot be used as a value",
                        designator.name(),
                        place.ty
                    ),
                ));
                None
            }
            Source::Str(bytes) => self.char_from_string(&bytes, expr.pos()),
        }
    }

    // Report 9.1 lets a single-character string stand for a CHAR value, and
    // this is the one place that rule is written. The empty string has no
    // characters, so it is not a single-character string and has no CHAR
    // value; cf. Project Oberon, which agrees, and OBNC, which converts any
    // string of length one or less.
    fn char_from_string(&mut self, bytes: &[u8], pos: Pos) -> Option<(ir::Value, Type)> {
        if bytes.len() == 1 {
            Some((ir::Value::Byte(bytes[0]), Type::Char))
        } else {
            self.diags.push(Diagnostic::new(
                pos,
                format!(
                    "a {} cannot be used as a value: only a single-character string is a CHAR",
                    Type::String(bytes.len())
                ),
            ));
            None
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
    fn for_step(&mut self, step: Option<&ast::Expr>) -> i64 {
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

    // Report 9.5. The INTEGER and CHAR forms reduce labels to ordinals, while
    // the record and pointer forms select the dynamic-type path first.
    fn lower_case(&mut self, expr: &ast::Expr, arms: &[ast::CaseArm]) {
        if let ast::Expr::Name(designator) = expr
            && let Ok((Symbol::Var { ty, .. }, _)) = self.qualident(designator)
            && matches!(ty, Type::Pointer(_) | Type::Record(_))
        {
            self.lower_type_case(designator, arms);
            return;
        }
        // "First the case expression is evaluated": once, into a temporary
        // that every arm's test then compares against.
        let (selector, char_labels) = match self.lower_expr(expr) {
            Some((value, Type::Integer)) => (Some(value), false),
            Some((value, Type::Char)) => (Some(value), true),
            Some((_, Type::Pointer(_))) => {
                self.diags.push(Diagnostic::new(
                    expr.pos(),
                    "type CASE requires a qualified variable without selectors",
                ));
                for arm in arms {
                    self.lower_stmts(&arm.body);
                }
                return;
            }
            Some((_, ty)) => {
                self.diags.push(Diagnostic::new(
                    expr.pos(),
                    format!("CASE expression must be INTEGER or CHAR, found {ty}"),
                ));
                (None, false)
            }
            None => (None, false),
        };

        // One label table for the whole statement, as in oberonc: labels must
        // be distinct across alternatives, not just within one.
        let mut covered: Vec<(i64, i64)> = Vec::new();
        let mut arm_ranges = Vec::new();
        for arm in arms {
            let mut ranges = Vec::new();
            for range in &arm.labels {
                let low = self.case_label(&range.low, char_labels);
                let high = match &range.high {
                    Some(expr) => self.case_label(expr, char_labels),
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

    fn lower_type_case(&mut self, designator: &ast::Designator, arms: &[ast::CaseArm]) {
        let (symbol, rest) = match self.qualident(designator) {
            Ok(found) => found,
            Err(diag) => {
                self.diags.push(diag);
                return;
            }
        };
        let Symbol::Var { ty: declared, .. } = symbol else {
            unreachable!("the caller selected a variable")
        };
        if !rest.is_empty() {
            self.diags.push(Diagnostic::new(
                designator.pos,
                "type CASE requires a qualified variable without selectors",
            ));
            for arm in arms {
                self.lower_stmts(&arm.body);
            }
            return;
        }
        let place = match self.place(designator, "type CASE requires a variable".into()) {
            Some(place) => place,
            None => return,
        };
        if matches!(declared, Type::Record(_))
            && !matches!(place.dynamic, Some(RecordDynamic::Incoming(_)))
        {
            self.diags.push(Diagnostic::new(
                designator.pos,
                "record type CASE requires a record VAR parameter",
            ));
            for arm in arms {
                self.lower_stmts(&arm.body);
            }
            return;
        }

        let actual = match &declared {
            Type::Pointer(_) => self.load(place.addr.clone(), ir::Ty::Pointer),
            Type::Record(_) => self.materialize_descriptor(&place),
            _ => unreachable!("the caller selected a dynamic type"),
        };

        let mut labels = Vec::new();
        let mut seen: Vec<Type> = Vec::new();
        for arm in arms {
            let target = if arm.labels.len() != 1 {
                self.diags.push(Diagnostic::new(
                    arm.labels
                        .first()
                        .map_or(designator.pos, |label| label.low.pos()),
                    "a type CASE arm requires exactly one type label",
                ));
                None
            } else {
                let label = &arm.labels[0];
                if label.high.is_some() {
                    self.diags.push(Diagnostic::new(
                        label.low.pos(),
                        "a type CASE label cannot be a range",
                    ));
                    None
                } else if let ast::Expr::Name(name) = &label.low {
                    let target = self.named_type(name);
                    match target {
                        Some(target)
                            if (matches!(declared, Type::Pointer(_))
                                && matches!(target, Type::Pointer(_))
                                || matches!(declared, Type::Record(_))
                                    && matches!(target, Type::Record(_)))
                                && match (&target, &declared) {
                                    (Type::Pointer(_), Type::Pointer(_)) => {
                                        pointer_extends(&target, &declared)
                                    }
                                    (Type::Record(target), Type::Record(declared)) => {
                                        record_extends(target, declared)
                                    }
                                    _ => false,
                                } =>
                        {
                            if seen.contains(&target) {
                                self.diags.push(Diagnostic::new(
                                    name.pos,
                                    format!("type CASE label {target} is duplicated"),
                                ));
                                None
                            } else {
                                seen.push(target.clone());
                                Some(target)
                            }
                        }
                        Some(target) => {
                            self.type_test_mismatch(label.low.pos(), name.pos, &declared, &target);
                            None
                        }
                        None => None,
                    }
                } else {
                    self.diags.push(Diagnostic::new(
                        label.low.pos(),
                        "a type CASE label must be a qualified type name",
                    ));
                    None
                }
            };
            labels.push(target);
        }

        let end = self.label("typecase.end");
        for (arm, target) in arms.iter().zip(labels) {
            let Some(target) = target else {
                self.lower_stmts(&arm.body);
                continue;
            };
            let descriptor = match &target {
                Type::Pointer(pointer) => pointer.record().map(|record| record.descriptor.clone()),
                Type::Record(record) => Some(record.descriptor.clone()),
                _ => None,
            };
            let Some(descriptor) = descriptor else {
                continue;
            };
            let test = self.temp();
            match declared {
                Type::Pointer(_) => self.emit(ir::Inst::TypeTestPointer {
                    dst: test,
                    pointer: actual.clone(),
                    target: descriptor,
                }),
                Type::Record(_) => self.emit(ir::Inst::TypeTestDescriptor {
                    dst: test,
                    descriptor: actual.clone(),
                    target: descriptor,
                }),
                _ => unreachable!(),
            }
            let body = self.label("typecase.arm");
            let next = self.label("typecase.next");
            self.emit(ir::Inst::Br {
                cond: ir::Value::Temp(test),
                then: body.clone(),
                els: next.clone(),
            });
            self.emit(ir::Inst::Label(body));
            let old = self.narrow_variable(designator, target);
            self.lower_stmts(&arm.body);
            if let Some(old) = old {
                self.narrow_variable(designator, old);
            }
            self.emit(ir::Inst::Jmp(end.clone()));
            self.emit(ir::Inst::Label(next));
        }
        self.emit(ir::Inst::Jmp(end.clone()));
        self.emit(ir::Inst::Label(end));
    }

    fn narrow_variable(&mut self, designator: &ast::Designator, ty: Type) -> Option<Type> {
        if let Some(ast::Selector::Field(member, _)) = designator.selectors.first() {
            for scope in self.scopes.iter_mut().rev() {
                if let Some(Symbol::Module(members)) = scope.get_mut(&designator.ident)
                    && let Some(Symbol::Var { ty: current, .. }) = members.get_mut(member)
                {
                    return Some(std::mem::replace(current, ty));
                }
            }
        } else {
            for scope in self.scopes.iter_mut().rev() {
                if let Some(Symbol::Var { ty: current, .. }) = scope.get_mut(&designator.ident) {
                    return Some(std::mem::replace(current, ty));
                }
            }
        }
        None
    }

    // Report 9.5: under an INTEGER selector every label is an integer, and
    // under a CHAR selector every label and range endpoint is a
    // single-character string or a CHAR constant. Both reduce to ordinals.
    fn case_label(&mut self, expr: &ast::Expr, char_labels: bool) -> Option<i64> {
        match self.eval_const(expr) {
            Ok(value) => match (char_labels, value) {
                (false, ConstValue::Int(value)) => Some(value),
                (true, ConstValue::Char(c)) => Some(i64::from(c)),
                (true, ConstValue::Str(bytes)) if bytes.len() == 1 => Some(i64::from(bytes[0])),
                (false, other) => {
                    self.diags.push(Diagnostic::new(
                        expr.pos(),
                        format!("case label must be INTEGER, found {}", other.ty()),
                    ));
                    None
                }
                (true, other) => {
                    self.diags.push(Diagnostic::new(
                        expr.pos(),
                        format!("case label must be CHAR, found {}", other.ty()),
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
            ast::Expr::Int { value, pos } => match i64::try_from(*value) {
                Ok(value) => Some((ir::Value::Int(value), Type::Integer)),
                Err(_) => {
                    self.diags
                        .push(Diagnostic::new(*pos, "integer literal out of range"));
                    None
                }
            },
            ast::Expr::Real { value, .. } => Some((ir::Value::Real(*value), Type::Real)),
            ast::Expr::Bool { value, .. } => Some((ir::Value::Bool(*value), Type::Boolean)),
            ast::Expr::Nil { .. } => Some((ir::Value::Pointer(0), Type::Nil)),
            ast::Expr::Set { elements, .. } => self.lower_set(elements),
            // Report 8: an expression operates on values, and an array
            // designator has none. Whole-array assignment is the one place
            // that wants the array itself, and it uses lower_source directly.
            // A string here follows the one single-character rule: Report 9.1
            // lets a single-character string stand for a CHAR, so the
            // comparison `ch >= "A"`, the argument `p("A")`, the assignment
            // `ch := 0X`, and the result `RETURN "A"` all lower through this
            // arm with no further cases.
            ast::Expr::Str { .. } | ast::Expr::Name(_) => {
                let source = self.lower_source(expr)?;
                self.source_scalar(expr, source)
            }
            ast::Expr::Apply { callee, args, pos } => {
                if self.application_kind(callee) == ApplicationKind::Guard {
                    let place = self.terminal_guard_place(
                        callee,
                        args,
                        *pos,
                        format!("'{}' cannot be used as a value", callee.name()),
                    )?;
                    let Some(ty) = place.ty.scalar() else {
                        self.diags.push(Diagnostic::new(
                            *pos,
                            format!(
                                "'{}' has type {} and cannot be used as a value",
                                callee.name(),
                                place.ty
                            ),
                        ));
                        return None;
                    };
                    return Some((self.load(place.addr, ty), place.ty));
                }
                match self.lower_call(callee, args, *pos) {
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
                }
            }
            ast::Expr::TypeTest { expr, ty, pos } => self.lower_type_test(expr, ty, *pos),
            ast::Expr::Unary { op, expr, pos } => {
                let (arg, found) = self.lower_expr(expr)?;
                match (op, &found) {
                    // Report 8.2.2: unary "+" is the identity on a numeric
                    // operand, so it needs no instruction of its own.
                    (ast::UnOp::Plus, Type::Integer | Type::Real) => Some((arg, found)),
                    // REAL negation flips the binary64 sign; INTEGER negation
                    // is the long operation, which still wraps at
                    // MIN(INTEGER) as it did before this slice.
                    (ast::UnOp::Neg, Type::Integer | Type::Real) => {
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
                    // Every one of the 64 bits belongs to the domain, so the
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
    // `low` and `63 - high`, both already checked to lie in 0 .. 63, so
    // neither can reach the register width.
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
    // and folding is what makes `{16 + 16}` as wrong as `{64}`.
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
        // QBE reduces a shift count modulo the register width, so an unchecked
        // 64 would quietly behave like 0.
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

    // The 0-through-255 domain shared by a BYTE store and a CHR argument,
    // with the shape every dynamic check here has: a value the compiler can
    // fold is a source diagnostic and emits no runtime check, and any other
    // value is compared against both bounds before it is used. `expr` is
    // absent when the value is computed and never foldable, as in INC on a
    // BYTE.
    fn check_byte_domain(
        &mut self,
        expr: Option<&ast::Expr>,
        value: ir::Value,
        domain: ByteDomain,
        pos: Pos,
    ) -> Option<ir::Value> {
        if let Some(expr) = expr {
            match self.try_eval_const(expr) {
                Ok(Some(ConstValue::Int(n))) => {
                    return if (0..=255).contains(&n) {
                        Some(value)
                    } else {
                        self.diags.push(Diagnostic::new(
                            expr.pos(),
                            format!(
                                "{} {n} is out of range: must be between 0 and 255",
                                domain.describe()
                            ),
                        ));
                        None
                    };
                }
                Ok(Some(_)) => unreachable!("the value was type-checked as INTEGER"),
                Ok(None) => {}
                Err(diag) => {
                    self.diags.push(diag);
                    return None;
                }
            }
        }
        let low = self.bin(ir::BinOp::Lt, ir::Ty::Int, value.clone(), ir::Value::Int(0));
        let high = self.bin(
            ir::BinOp::Gt,
            ir::Ty::Int,
            value.clone(),
            ir::Value::Int(255),
        );
        let bad = self.bin(ir::BinOp::BitOr, ir::Ty::Bool, low, high);
        let trap = self.label(&format!("{}.bad", domain.label()));
        let ok = self.label(&format!("{}.ok", domain.label()));
        self.emit(ir::Inst::Br {
            cond: bad,
            then: trap.clone(),
            els: ok.clone(),
        });
        self.emit(ir::Inst::Label(trap));
        self.trap(domain.trap(), pos);
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
                    // Only REAL reaches this: "/" on two INTEGERs is not an
                    // Oberon operation, and the SET form returned above.
                    // IEEE division by zero yields an infinity or a NaN and
                    // is not a trap, unlike DIV and MOD.
                    BinOp::Slash => ir::BinOp::Div,
                    _ => unreachable!(),
                },
                operand_ty.ir(),
                lhs,
                rhs_ir,
            ),
        };
        Some((value, result_ty))
    }

    // Report 8.2.4. The relations order INTEGER, REAL, and CHAR, compare SET
    // and BOOLEAN for equality, and extend to strings and character arrays:
    // two operands may be compared when each is a character array or a
    // string and at least one is a character array. Two strings fold
    // instead, and a CHAR against a single-character string is an ordinary
    // CHAR comparison, so neither reaches the runtime call.
    fn lower_relation(
        &mut self,
        op: ast::BinOp,
        lhs: &ast::Expr,
        rhs: &ast::Expr,
        pos: Pos,
    ) -> Option<(ir::Value, Type)> {
        use ast::BinOp;
        // Both operands are resolved left to right and each exactly once, so
        // an index expression in either runs once and is bounds-checked
        // before the comparison.
        let lhs_src = self.lower_source(lhs);
        let rhs_src = self.lower_source(rhs);
        let (Some(lhs_src), Some(rhs_src)) = (lhs_src, rhs_src) else {
            return None;
        };

        if let (Source::Str(a), Source::Str(b)) = (&lhs_src, &rhs_src) {
            let holds = relation_holds(op, str_const_cmp(a, b));
            return Some((ir::Value::Bool(holds), Type::Boolean));
        }

        let stringy = |source: &Source| match source {
            Source::Str(_) => true,
            Source::Structured(place) => place.ty.char_array(),
            Source::Value(..) => false,
        };
        let is_array = |source: &Source| matches!(source, Source::Structured(_));
        if stringy(&lhs_src) && stringy(&rhs_src) && (is_array(&lhs_src) || is_array(&rhs_src)) {
            let (lhs_addr, lhs_len) = self.cmp_operand(lhs_src);
            let (rhs_addr, rhs_len) = self.cmp_operand(rhs_src);
            let result = self.call_runtime(
                "oberon_str_cmp",
                vec![
                    ir::Arg::Ref(lhs_addr),
                    ir::Arg::Val(ir::Ty::Int, lhs_len),
                    ir::Arg::Ref(rhs_addr),
                    ir::Arg::Val(ir::Ty::Int, rhs_len),
                ],
                ir::Ty::Int,
            );
            // One runtime helper serves all six relations; only the
            // comparison against the returned value differs.
            let value = self.bin(relation_ir(op), ir::Ty::Int, result, ir::Value::Int(0));
            return Some((value, Type::Boolean));
        }

        let lhs_value = self.source_scalar(lhs, lhs_src);
        let rhs_value = self.source_scalar(rhs, rhs_src);
        let (Some((lhs_value, lhs_ty)), Some((rhs_value, rhs_ty))) = (lhs_value, rhs_value) else {
            return None;
        };
        let operand_ty = match op {
            BinOp::Eq | BinOp::Ne => {
                if lhs_ty == rhs_ty && lhs_ty.scalar().is_some() {
                    lhs_ty
                } else if pointer_value_compatible(&lhs_ty, &rhs_ty) {
                    // NIL has no type of its own, so the comparison takes its
                    // class from whichever side is a pointer. Two NILs keep
                    // the pseudo-type, which answers with the same class.
                    if lhs_ty.pointer().is_some() {
                        lhs_ty
                    } else {
                        rhs_ty
                    }
                } else if procedure_value_compatible(&lhs_ty, &rhs_ty) {
                    if lhs_ty.procedure().is_some() {
                        lhs_ty
                    } else if rhs_ty.procedure().is_some() {
                        rhs_ty
                    } else {
                        lhs_ty
                    }
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
        // A CHAR operand is zero-extended in its register, so the signed
        // comparison the emitter picks for it gives the unsigned ordering.
        let value = self.bin(relation_ir(op), operand_ty.ir(), lhs_value, rhs_value);
        Some((value, Type::Boolean))
    }

    // One side of a runtime character comparison: an address and the length
    // that bounds the walk. A character array is bounded by its declared
    // length, and a string by its character count plus one for the
    // terminator its data object carries.
    fn cmp_operand(&mut self, source: Source) -> (ir::Addr, ir::Value) {
        match source {
            Source::Structured(place) => {
                assert!(place.ty.char_array(), "the operand is a character array");
                (place.addr, place.shape[0].clone())
            }
            Source::Str(bytes) => {
                let len = i64::try_from(bytes.len() + 1).expect("a literal fits the source file");
                (self.literal(&bytes), ir::Value::Int(len))
            }
            Source::Value(..) => unreachable!("the operand was checked to be characters"),
        }
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
        // Resolve and, for an indirect call, load the target before lowering
        // any actual. That captures selectors and the value exactly once.
        let callable = match self.qualident(callee) {
            Ok((Symbol::Builtin(builtin), [])) => {
                return self.lower_builtin(builtin, actuals, pos);
            }
            Ok((Symbol::Proc { symbol, ty, .. }, [])) => {
                Some((ir::CallTarget::Direct(symbol), ty, None))
            }
            Ok((Symbol::Var { .. }, _)) => {
                let place = self.place(callee, format!("'{}' is not a procedure", callee.name()));
                match place {
                    Some(place) if place.ty.procedure().is_some() => {
                        let value = self.load(place.addr, ir::Ty::Procedure);
                        Some((
                            ir::CallTarget::Indirect(value.clone()),
                            place.ty,
                            Some(value),
                        ))
                    }
                    Some(place) => {
                        self.diags.push(Diagnostic::new(
                            pos,
                            format!(
                                "'{}' has type {} and is not a procedure",
                                callee.name(),
                                place.ty
                            ),
                        ));
                        return None;
                    }
                    None => None,
                }
            }
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

        let Some((target, ty, indirect)) = callable else {
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
            // A structured formal — value or VAR — takes the address of its
            // actual, so the actual must be a designator of the identical
            // type. For VAR that is Report 10.1's rule; for a structured
            // value it is what remains of assignment compatibility once the
            // actual must be addressable. The one casualty is a string actual
            // for a fixed character-array formal, which Project Oberon and
            // oberonc also reject: under reference semantics no assignment to
            // the formal happens, so Report 9.1's string exception has
            // nothing to attach to. Strings meet parameters through Slice
            // 14's open arrays of CHAR. The rejection covers a string of any
            // length, a single character included, and is checked before the
            // expression is lowered so it reports as a string where a
            // variable is required rather than as a CHAR mismatch.
            if var || expected.structured() {
                if expected.structured()
                    && let Some(bytes) = self.string_expr(actual)
                {
                    if !var && open_string_formal(&expected) {
                        let len = i64::try_from(bytes.len() + 1)
                            .expect("a string literal fits the source file");
                        args.push(ir::Arg::Ref(self.literal(&bytes)));
                        args.push(ir::Arg::Val(ir::Ty::Int, ir::Value::Int(len)));
                    } else {
                        self.diags.push(Diagnostic::new(
                            actual.pos(),
                            format!(
                                "argument {} is a string where a variable is required",
                                i + 1
                            ),
                        ));
                        ok = false;
                    }
                    continue;
                }
                // A structured value actual is a read, so a read-only
                // designator — an imported variable, another structured value
                // parameter — may be passed on. A VAR actual will be written
                // and may not be.
                match self.ref_actual(actual, i + 1, var) {
                    Some(place) => {
                        let compatible = if expected.open_rank() > 0 {
                            open_actual_compatible(&expected, &place.ty)
                        } else if let (Some(expected), Some(actual)) =
                            (expected.record(), place.ty.record())
                        {
                            record_extends(actual, expected)
                        } else {
                            place.ty == expected
                        };
                        if compatible {
                            args.push(ir::Arg::Ref(place.addr.clone()));
                            if var && expected.record().is_some() {
                                let descriptor = self.materialize_descriptor(&place);
                                args.push(ir::Arg::Val(ir::Ty::Pointer, descriptor));
                            }
                            for length in place.shape.iter().take(expected.open_rank()) {
                                args.push(ir::Arg::Val(ir::Ty::Int, length.clone()));
                            }
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
                // assignment. A single-character string already lowered to a
                // CHAR, so only the checked BYTE store appears here.
                match self.lower_expr(actual) {
                    Some((value, found)) => match assign_kind(&expected, &found) {
                        Some(AssignKind::Store) => {
                            args.push(ir::Arg::Val(expected.ir(), value));
                        }
                        Some(AssignKind::ByteRange) => {
                            match self.check_byte_domain(
                                Some(actual),
                                value,
                                ByteDomain::Store,
                                actual.pos(),
                            ) {
                                Some(value) => args.push(ir::Arg::Val(ir::Ty::Byte, value)),
                                None => ok = false,
                            }
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
        if let Some(procedure) = indirect {
            let site = self.site(pos);
            self.emit(ir::Inst::CheckProcedure { procedure, site });
        }
        let dst = procedure.ret.as_ref().map(|ty| (self.temp(), ty.ir()));
        self.emit(ir::Inst::Call { dst, target, args });
        // Report 6.1 again: a function whose result type is BYTE produces an
        // INTEGER value when it is called.
        let ret = procedure
            .ret
            .clone()
            .map(|ty| if ty == Type::Byte { Type::Integer } else { ty });
        Some((dst.map(|(temp, _)| ir::Value::Temp(temp)), ret))
    }

    fn lower_builtin(
        &mut self,
        builtin: Builtin,
        actuals: &[ast::Expr],
        pos: Pos,
    ) -> Option<(Option<ir::Value>, Option<Type>)> {
        match builtin {
            Builtin::Len => return self.lower_len(actuals, pos),
            Builtin::Inc | Builtin::Dec => return self.lower_inc_dec(builtin, actuals, pos),
            Builtin::Incl | Builtin::Excl => return self.lower_incl_excl(builtin, actuals, pos),
            Builtin::Pack => return self.lower_pack(actuals, pos),
            Builtin::Unpk => return self.lower_unpk(actuals, pos),
            Builtin::Assert => return self.lower_assert(actuals, pos),
            Builtin::New => return self.lower_new(actuals, pos),
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
            Builtin::Abs => match args[0].1 {
                Type::Integer => self.lower_abs_int(args[0].0.clone(), pos),
                Type::Real => self.call_runtime(
                    "oberon_abs_real",
                    vec![ir::Arg::Val(ir::Ty::Real, args[0].0.clone())],
                    ir::Ty::Real,
                ),
                _ => unreachable!("ABS was type-checked as INTEGER or REAL"),
            },
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
            // The rounded argument has to be inside the INTEGER range for a
            // result to exist, and neither QBE nor C defines a conversion
            // that is out of range, so the wrapper checks the domain itself.
            Builtin::Floor => {
                self.check_floor_argument(&actuals[0])?;
                let mut floor_args = vec![ir::Arg::Val(ir::Ty::Real, args[0].0.clone())];
                floor_args.extend(self.site(pos).args());
                self.call_runtime("oberon_floor", floor_args, ir::Ty::Int)
            }
            // The one conversion between machine classes. It rounds to the
            // nearest binary64 value, so an INTEGER above 2^53 in magnitude of the range
            // does not survive the trip.
            Builtin::Flt => {
                let dst = self.temp();
                self.emit(ir::Inst::IntToReal {
                    dst,
                    arg: args[0].0.clone(),
                });
                ir::Value::Temp(dst)
            }
            // BOOLEAN is already 0 or 1 in a register, a SET is already its own
            // bit pattern, and a CHAR is already its ordinal, so the value
            // passes through with only its type changed. cf. Project Oberon,
            // where ORD lowers to nothing.
            Builtin::Ord => args[0].0.clone(),
            // CHR emits no conversion either: the checked value is the
            // result. Its domain is the CHAR ordinals, which are the same
            // 0 through 255 as BYTE, but the check and the message are its
            // own, matching how each dynamic check has its own line.
            Builtin::Chr => {
                self.check_byte_domain(Some(&actuals[0]), args[0].0.clone(), ByteDomain::Chr, pos)?
            }
            Builtin::Lsl | Builtin::Asr | Builtin::Ror => {
                self.lower_shift(builtin, args[0].0.clone(), args[1].0.clone(), &actuals[1])?
            }
            Builtin::Len
            | Builtin::Inc
            | Builtin::Dec
            | Builtin::Incl
            | Builtin::Excl
            | Builtin::Pack
            | Builtin::Unpk
            | Builtin::Assert
            | Builtin::New => unreachable!("handled above"),
        };
        Some((Some(value), Some(result.ty(args[0].1.clone()))))
    }

    // Report 10.2: LEN(v) is the length of the array v. For a fixed array that
    // length is a property of the type, so the result is an immediate — but
    // the designator is still resolved, so `LEN(a[f()])` calls f once and
    // checks its result before returning the inner length. cf. Project
    // Oberon's ORP.StandFunc, which lowers the designator and then reads the
    // length out of its type.
    fn lower_len(
        &mut self,
        actuals: &[ast::Expr],
        pos: Pos,
    ) -> Option<(Option<ir::Value>, Option<Type>)> {
        // The arguments are deliberately not lowered when the arity is wrong:
        // an array actual has no value, and reporting that on top of the arity
        // would be two complaints about one mistake.
        if actuals.len() != 1 {
            self.diags.push(Diagnostic::new(
                pos,
                format!(
                    "wrong number of arguments: expected 1, found {}",
                    actuals.len()
                ),
            ));
            return None;
        }
        let ast::Expr::Name(designator) = &actuals[0] else {
            let _ = self.lower_expr(&actuals[0]);
            self.diags.push(Diagnostic::new(
                actuals[0].pos(),
                "argument 1 must be an array variable",
            ));
            return None;
        };
        let place = self.place(designator, "argument 1 must be an array variable".into())?;
        if !place.ty.is_array() {
            self.diags.push(Diagnostic::new(
                actuals[0].pos(),
                format!("argument 1 has type {}, expected an array", place.ty),
            ));
            return None;
        }
        Some((Some(place.shape[0].clone()), Some(Type::Integer)))
    }

    // Report 10.2: PACK(x, n) is x := x * 2^n on a writable REAL variable.
    // The address is resolved before the exponent is lowered, so the two
    // arguments are evaluated left to right and each exactly once.
    fn lower_pack(
        &mut self,
        actuals: &[ast::Expr],
        pos: Pos,
    ) -> Option<(Option<ir::Value>, Option<Type>)> {
        if !self.builtin_arity(actuals, 2, pos) {
            return None;
        }
        let target = self.modified_actual(&actuals[0], 1, Type::Real);
        let exponent = self
            .builtin_arg(&actuals[1], 2, &[Type::Integer])
            .map(|(value, _)| value);
        if let (Some(addr), Some(exponent)) = (target, exponent) {
            self.emit(ir::Inst::Call {
                dst: None,
                target: ir::CallTarget::Direct("oberon_pack".into()),
                args: vec![ir::Arg::Ref(addr), ir::Arg::Val(ir::Ty::Int, exponent)],
            });
        }
        Some((None, None))
    }

    // Report 10.2: UNPK(x, n) splits x into a fraction and an exponent, so
    // both arguments are writable variables and both reach the runtime as
    // addresses.
    fn lower_unpk(
        &mut self,
        actuals: &[ast::Expr],
        pos: Pos,
    ) -> Option<(Option<ir::Value>, Option<Type>)> {
        if !self.builtin_arity(actuals, 2, pos) {
            return None;
        }
        let fraction = self.modified_actual(&actuals[0], 1, Type::Real);
        let exponent = self.modified_actual(&actuals[1], 2, Type::Integer);
        if let (Some(fraction), Some(exponent)) = (fraction, exponent) {
            self.emit(ir::Inst::Call {
                dst: None,
                target: ir::CallTarget::Direct("oberon_unpk".into()),
                args: [ir::Arg::Ref(fraction), ir::Arg::Ref(exponent)]
                    .into_iter()
                    .chain(self.site(pos).args())
                    .collect(),
            });
        }
        Some((None, None))
    }

    fn lower_new(
        &mut self,
        actuals: &[ast::Expr],
        pos: Pos,
    ) -> Option<(Option<ir::Value>, Option<Type>)> {
        if !self.builtin_arity(actuals, 1, pos) {
            return None;
        }
        let target = self.var_actual(&actuals[0], 1);
        match target {
            Some((addr, Type::Pointer(pointer))) => {
                if let Some(record) = pointer.record() {
                    let dst = self.temp();
                    self.emit(ir::Inst::Alloc {
                        dst,
                        size: record.size,
                        scanned: record.contains_pointers,
                        descriptor: record.descriptor.clone(),
                    });
                    self.emit(ir::Inst::Store {
                        ty: ir::Ty::Pointer,
                        val: ir::Value::Temp(dst),
                        addr,
                    });
                }
            }
            Some((_, found)) => self.diags.push(Diagnostic::new(
                actuals[0].pos(),
                format!("argument 1 has type {found}, expected a pointer"),
            )),
            None => {}
        }
        Some((None, None))
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

    // A predefined operation the C runtime implements. The call names the
    // exact value type of every argument and of the result, so QBE emits the
    // native calling classes.
    fn call_runtime(&mut self, symbol: &str, args: Vec<ir::Arg>, ret: ir::Ty) -> ir::Value {
        let dst = self.temp();
        self.emit(ir::Inst::Call {
            dst: Some((dst, ret)),
            target: ir::CallTarget::Direct(symbol.into()),
            args,
        });
        ir::Value::Temp(dst)
    }

    // A known FLOOR argument outside the INTEGER result domain is a source
    // error even in an executable expression. A dynamic argument keeps the
    // runtime check. Valid constants also keep the ordinary runtime lowering,
    // because this compiler has no optimization pass.
    fn check_floor_argument(&mut self, actual: &ast::Expr) -> Option<()> {
        match self.try_eval_const(actual) {
            Ok(Some(ConstValue::Real(value))) => match floor_const(value, actual.pos()) {
                Ok(_) => Some(()),
                Err(diag) => {
                    self.diags.push(diag);
                    None
                }
            },
            Ok(Some(_)) => unreachable!("FLOOR argument was type-checked as REAL"),
            Ok(None) => Some(()),
            Err(diag) => {
                self.diags.push(diag);
                None
            }
        }
    }

    // MIN(INTEGER) has no absolute value. Folding rejects it, so the runtime
    // form must not quietly wrap the way unary minus still does. The REAL
    // form has no such hole and is an ordinary runtime call.
    fn lower_abs_int(&mut self, arg: ir::Value, pos: Pos) -> ir::Value {
        let overflows = self.bin(
            ir::BinOp::Eq,
            ir::Ty::Int,
            arg.clone(),
            ir::Value::Int(i64::MIN),
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
    // result width, which would silently turn LSL(x, 64) into x. Diagnose a
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
                if !(0..=63).contains(n) {
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
                    ir::Value::Int(63),
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
            // of a shift by the register width.
            Builtin::Ror => {
                let right = self.bin(ir::BinOp::Shr, ir::Ty::Int, value.clone(), count.clone());
                let complement = self.bin(ir::BinOp::Sub, ir::Ty::Int, ir::Value::Int(64), count);
                let left_count = self.bin(
                    ir::BinOp::BitAnd,
                    ir::Ty::Int,
                    complement,
                    ir::Value::Int(63),
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
        // The argument is an integer variable, and BYTE is compatible with
        // INTEGER, so a BYTE variable is accepted too: it reads as an
        // INTEGER and writes through the same checked store as assignment,
        // so incrementing past 255 fails at run time — a genuine
        // out-of-range write, not an artefact of the lowering.
        let target = match self.var_actual(&actuals[0], 1) {
            Some((addr, ty @ (Type::Integer | Type::Byte))) => Some((addr, ty)),
            Some((_, found)) => {
                self.diags.push(Diagnostic::new(
                    actuals[0].pos(),
                    format!("argument 1 has type {found}, expected INTEGER or BYTE"),
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
            let next = if ty == Type::Byte {
                self.check_byte_domain(None, next, ByteDomain::Store, pos)
                    .expect("a computed value only takes the dynamic check")
            } else {
                next
            };
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

    // The address an actual supplies to a reference formal. `writable` is
    // false only for a structured value actual, which is a read and therefore
    // the one reference an imported variable or another structured value
    // parameter can be.
    fn ref_actual(&mut self, actual: &ast::Expr, number: usize, writable: bool) -> Option<Place> {
        let place = match actual {
            ast::Expr::Name(designator) => {
                self.place(designator, format!("argument {number} must be a variable"))
            }
            ast::Expr::Apply { callee, args, pos }
                if self.application_kind(callee) == ApplicationKind::Guard =>
            {
                self.terminal_guard_place(
                    callee,
                    args,
                    *pos,
                    format!("argument {number} must be a variable"),
                )
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

    // A string in actual-parameter position: a literal or a constant declared
    // from one. Recognized before lowering, the way every context that treats
    // a string specially examines the expression first.
    fn string_expr(&self, expr: &ast::Expr) -> Option<Rc<Vec<u8>>> {
        match expr {
            ast::Expr::Str { bytes, .. } => Some(Rc::new(bytes.clone())),
            ast::Expr::Name(designator) => match self.qualident(designator) {
                Ok((Symbol::Const(ConstValue::Str(bytes)), [])) => Some(bytes),
                _ => None,
            },
            _ => None,
        }
    }

    fn application_kind(&self, callee: &ast::Designator) -> ApplicationKind {
        if matches!(
            self.resolve(callee),
            Ok(Symbol::Proc { .. } | Symbol::Builtin(_))
        ) {
            return ApplicationKind::Call;
        }
        match self.designator_type(callee) {
            Ok(Type::Procedure(_)) => ApplicationKind::Call,
            Ok(Type::Pointer(_) | Type::Record(_)) => ApplicationKind::Guard,
            Ok(_) | Err(_) => ApplicationKind::Invalid,
        }
    }

    // The non-emitting type walk used only to classify a terminal `(…)` and
    // to check a call in a required constant context. Executable lowering
    // still uses place(), which evaluates and checks every selector.
    fn designator_type(&self, designator: &ast::Designator) -> Result<Type, Diagnostic> {
        let (symbol, rest) = self.qualident(designator)?;
        let mut ty = match symbol {
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
                    if let Type::Pointer(pointer) = &ty {
                        ty = pointer.record().map(Type::Record).ok_or_else(|| {
                            Diagnostic::new(*pos, "pointer has an invalid base type")
                        })?;
                    }
                    let Some(record) = ty.record() else {
                        return Err(Diagnostic::new(
                            *pos,
                            format!("cannot select '{name}' from {ty}: only a record has fields"),
                        ));
                    };
                    ty = find_field(record, name, &self.module)
                        .map(|field| field.ty.clone())
                        .ok_or_else(|| no_such_field(*pos, name, &ty))?;
                }
                ast::Selector::Index(exprs, pos) => {
                    for _ in exprs {
                        ty = ty.array_elem().cloned().ok_or_else(|| {
                            Diagnostic::new(
                                *pos,
                                format!("cannot index {ty}: only an array can be indexed"),
                            )
                        })?;
                    }
                }
                ast::Selector::Deref(pos) => {
                    let Type::Pointer(pointer) = &ty else {
                        return Err(Diagnostic::new(
                            *pos,
                            format!("cannot dereference {ty}: only a pointer can be dereferenced"),
                        ));
                    };
                    ty = pointer
                        .record()
                        .map(Type::Record)
                        .ok_or_else(|| Diagnostic::new(*pos, "pointer has an invalid base type"))?;
                }
                ast::Selector::Guard(target, _) => match self.resolve(target)? {
                    Symbol::TypeName(target) => ty = target,
                    _ => {
                        return Err(Diagnostic::new(
                            target.pos,
                            format!("'{}' is not a type", target.name()),
                        ));
                    }
                },
            }
        }
        Ok(ty)
    }

    fn terminal_guard_place(
        &mut self,
        callee: &ast::Designator,
        args: &[ast::Expr],
        pos: Pos,
        not_a_variable: String,
    ) -> Option<Place> {
        let [ast::Expr::Name(target)] = args else {
            for arg in args {
                let _ = self.lower_expr(arg);
            }
            self.diags.push(Diagnostic::new(
                pos,
                "a type guard requires exactly one qualified type name",
            ));
            return None;
        };
        if target.selectors.len() > 1
            || matches!(
                target.selectors.first(),
                Some(ast::Selector::Index(..) | ast::Selector::Deref(_) | ast::Selector::Guard(..))
            )
        {
            self.diags.push(Diagnostic::new(
                pos,
                "a type guard requires exactly one qualified type name",
            ));
            return None;
        }
        let place = self.place(callee, not_a_variable)?;
        self.guard_place(place, target, pos)
    }

    fn named_type(&mut self, target: &ast::Designator) -> Option<Type> {
        match self.resolve(target) {
            Ok(Symbol::TypeName(ty)) => Some(ty),
            Ok(_) => {
                self.diags.push(Diagnostic::new(
                    target.pos,
                    format!("'{}' is not a type", target.name()),
                ));
                None
            }
            Err(diag) => {
                self.diags.push(diag);
                None
            }
        }
    }

    fn guard_place(
        &mut self,
        mut place: Place,
        target_name: &ast::Designator,
        pos: Pos,
    ) -> Option<Place> {
        let target = self.named_type(target_name)?;
        match (&place.ty, &target) {
            (Type::Pointer(_), Type::Pointer(target_pointer))
                if pointer_extends(&target, &place.ty) =>
            {
                let pointer = self.load(place.addr.clone(), ir::Ty::Pointer);
                let target_record = target_pointer.record()?;
                let test = self.temp();
                self.emit(ir::Inst::TypeTestPointer {
                    dst: test,
                    pointer,
                    target: target_record.descriptor.clone(),
                });
                self.emit_guard_check(ir::Value::Temp(test), pos);
                place.ty = target;
                Some(place)
            }
            (Type::Pointer(_), Type::Pointer(_)) => {
                self.diags.push(Diagnostic::new(
                    target_name.pos,
                    format!("guard type {target} must extend {}", place.ty),
                ));
                None
            }
            (Type::Pointer(_), _) => {
                self.diags.push(Diagnostic::new(
                    target_name.pos,
                    format!("pointer guard requires a pointer type, found {target}"),
                ));
                None
            }
            (Type::Record(subject), Type::Record(target_record))
                if matches!(place.dynamic, Some(RecordDynamic::Incoming(_)))
                    && record_extends(target_record, subject) =>
            {
                let descriptor = self.materialize_descriptor(&place);
                let test = self.temp();
                self.emit(ir::Inst::TypeTestDescriptor {
                    dst: test,
                    descriptor,
                    target: target_record.descriptor.clone(),
                });
                self.emit_guard_check(ir::Value::Temp(test), pos);
                place.ty = target;
                Some(place)
            }
            (Type::Record(_), Type::Record(_))
                if !matches!(place.dynamic, Some(RecordDynamic::Incoming(_))) =>
            {
                self.diags.push(Diagnostic::new(
                    pos,
                    "a record guard requires a record VAR parameter",
                ));
                None
            }
            (Type::Record(_), Type::Record(_)) => {
                self.diags.push(Diagnostic::new(
                    target_name.pos,
                    format!("guard type {target} must extend {}", place.ty),
                ));
                None
            }
            (Type::Record(_), _) => {
                self.diags.push(Diagnostic::new(
                    target_name.pos,
                    format!("record guard requires a record type, found {target}"),
                ));
                None
            }
            _ => {
                self.diags.push(Diagnostic::new(
                    pos,
                    format!(
                        "a type guard requires a pointer or record VAR parameter, found {}",
                        place.ty
                    ),
                ));
                None
            }
        }
    }

    fn emit_guard_check(&mut self, test: ir::Value, pos: Pos) {
        let ok = self.label("typeguard.ok");
        let bad = self.label("typeguard.bad");
        self.emit(ir::Inst::Br {
            cond: test,
            then: ok.clone(),
            els: bad.clone(),
        });
        self.emit(ir::Inst::Label(bad));
        self.trap("oberon_type_guard_failed", pos);
        self.emit(ir::Inst::Label(ok));
    }

    fn lower_type_test(
        &mut self,
        expr: &ast::Expr,
        target_name: &ast::Designator,
        pos: Pos,
    ) -> Option<(ir::Value, Type)> {
        if let ast::Expr::Name(designator) = expr
            && matches!(self.qualident(designator), Ok((Symbol::Var { .. }, _)))
        {
            let place = self.place(
                designator,
                format!("'{}' cannot be used in a type test", designator.name()),
            )?;
            let target = self.named_type(target_name)?;
            return match (&place.ty, &target) {
                (Type::Pointer(_), Type::Pointer(target_pointer))
                    if pointer_extends(&target, &place.ty) =>
                {
                    let pointer = self.load(place.addr, ir::Ty::Pointer);
                    let record = target_pointer.record()?;
                    let dst = self.temp();
                    self.emit(ir::Inst::TypeTestPointer {
                        dst,
                        pointer,
                        target: record.descriptor.clone(),
                    });
                    Some((ir::Value::Temp(dst), Type::Boolean))
                }
                (Type::Record(subject), Type::Record(target_record))
                    if matches!(place.dynamic, Some(RecordDynamic::Incoming(_)))
                        && record_extends(target_record, subject) =>
                {
                    let descriptor = self.materialize_descriptor(&place);
                    let dst = self.temp();
                    self.emit(ir::Inst::TypeTestDescriptor {
                        dst,
                        descriptor,
                        target: target_record.descriptor.clone(),
                    });
                    Some((ir::Value::Temp(dst), Type::Boolean))
                }
                (Type::Record(_), _)
                    if !matches!(place.dynamic, Some(RecordDynamic::Incoming(_))) =>
                {
                    self.diags.push(Diagnostic::new(
                        pos,
                        "a record type test requires a record VAR parameter",
                    ));
                    None
                }
                _ => {
                    self.type_test_mismatch(pos, target_name.pos, &place.ty, &target);
                    None
                }
            };
        }

        let (value, subject) = self.lower_expr(expr)?;
        let target = self.named_type(target_name)?;
        match (&subject, &target) {
            (Type::Nil, Type::Pointer(_)) => Some((ir::Value::Bool(false), Type::Boolean)),
            (Type::Pointer(_), Type::Pointer(target_pointer))
                if pointer_extends(&target, &subject) =>
            {
                let record = target_pointer.record()?;
                let dst = self.temp();
                self.emit(ir::Inst::TypeTestPointer {
                    dst,
                    pointer: value,
                    target: record.descriptor.clone(),
                });
                Some((ir::Value::Temp(dst), Type::Boolean))
            }
            _ => {
                self.type_test_mismatch(pos, target_name.pos, &subject, &target);
                None
            }
        }
    }

    fn type_test_mismatch(&mut self, pos: Pos, target_pos: Pos, subject: &Type, target: &Type) {
        let diagnostic = match (subject, target) {
            (Type::Pointer(_), Type::Pointer(_)) | (Type::Record(_), Type::Record(_)) => {
                Diagnostic::new(
                    target_pos,
                    format!("type-test target {target} must extend {subject}"),
                )
            }
            (Type::Pointer(_) | Type::Nil, _) => Diagnostic::new(
                target_pos,
                format!("pointer type test requires a pointer type, found {target}"),
            ),
            (Type::Record(_), _) => Diagnostic::new(
                target_pos,
                format!("record type test requires a record type, found {target}"),
            ),
            _ => Diagnostic::new(
                pos,
                format!("a type test requires a pointer or record VAR parameter, found {subject}"),
            ),
        };
        self.diags.push(diagnostic);
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
            shape,
            read_only,
            dynamic,
        } = symbol
        else {
            self.diags
                .push(Diagnostic::new(designator.pos, not_a_variable));
            return None;
        };
        let mut place = Place {
            addr,
            ty,
            shape,
            read_only,
            dynamic,
        };
        for selector in rest {
            match selector {
                // Report 8.1: if r designates a record, r.f denotes the field
                // f of r.
                ast::Selector::Field(name, pos) => {
                    place = self.field(place, name, *pos)?;
                }
                // Report 8.1: a[i, j] abbreviates a[i][j], so each expression
                // of one bracket list is its own index selector.
                ast::Selector::Index(exprs, pos) => {
                    for expr in exprs {
                        place = self.index(place, expr, *pos)?;
                    }
                }
                ast::Selector::Deref(pos) => {
                    place = self.dereference(place, *pos)?;
                }
                ast::Selector::Guard(ty, pos) => {
                    place = self.guard_place(place, ty, *pos)?;
                }
            }
        }
        Some(place)
    }

    // One field. The offset is a constant the layout already fixed, so this is
    // an address computation with nothing to check, and the base's read-only
    // flag carries through: a field of an imported variable or of a structured
    // value parameter is as unwritable as the whole.
    fn field(&mut self, base: Place, name: &str, pos: Pos) -> Option<Place> {
        let base = if base.ty.pointer().is_some() {
            self.dereference(base, pos)?
        } else {
            base
        };
        let Some(record) = base.ty.record().cloned() else {
            self.diags.push(Diagnostic::new(
                pos,
                format!(
                    "cannot select '{name}' from {}: only a record has fields",
                    base.ty
                ),
            ));
            return None;
        };
        let Some(field) = find_field(&record, name, &self.module) else {
            self.diags.push(no_such_field(pos, name, &base.ty));
            return None;
        };
        let (ty, offset) = (field.ty.clone(), field.offset);
        let dst = self.temp();
        self.emit(ir::Inst::Field {
            dst,
            base: base.addr,
            offset,
        });
        Some(Place {
            addr: ir::Addr::Temp(dst),
            shape: fixed_shape(&ty),
            ty: ty.clone(),
            read_only: base.read_only,
            dynamic: ty
                .record()
                .map(|record| RecordDynamic::Static(record.descriptor.clone())),
        })
    }

    fn dereference(&mut self, base: Place, pos: Pos) -> Option<Place> {
        let Some(pointer) = base.ty.pointer().cloned() else {
            self.diags.push(Diagnostic::new(
                pos,
                format!(
                    "cannot dereference {}: only a pointer can be dereferenced",
                    base.ty
                ),
            ));
            return None;
        };
        let record = pointer.record()?;
        let value = self.load(base.addr, ir::Ty::Pointer);
        let site = self.site(pos);
        self.emit(ir::Inst::CheckNil {
            pointer: value.clone(),
            site,
        });
        let ir::Value::Temp(temp) = value else {
            unreachable!("a loaded pointer is a temporary")
        };
        Some(Place {
            addr: ir::Addr::Temp(temp),
            ty: Type::Record(record),
            shape: Vec::new(),
            read_only: base.read_only,
            // The checked pointer is enough to recover the hidden descriptor.
            // Loading it is deferred until a call or dynamic operation needs it.
            dynamic: Some(RecordDynamic::Heap(ir::Value::Temp(temp))),
        })
    }

    // One dimension. The index is evaluated before anything is done with it,
    // and the check the IR carries runs before the address is formed, so a
    // later dimension's expression cannot run ahead of an earlier dimension's
    // check and no invalid address is ever computed.
    fn index(&mut self, base: Place, expr: &ast::Expr, pos: Pos) -> Option<Place> {
        let index = self.lower_int(expr, "array index");
        let Some(elem) = base.ty.array_elem().cloned() else {
            self.diags.push(Diagnostic::new(
                pos,
                format!("cannot index {}: only an array can be indexed", base.ty),
            ));
            return None;
        };
        let index = index?;
        // An index the compiler can fold is a source error, exactly as an
        // out-of-range SET element is. A valid constant one still takes the
        // ordinary checked lowering; there is no optimization pass, and one
        // executable path is what makes the IR invariant literal.
        if let Some(array) = base.ty.array() {
            match self.try_eval_const(expr) {
                Ok(Some(ConstValue::Int(value))) => {
                    if !(0..array.len).contains(&value) {
                        self.diags
                            .push(index_range_error(expr.pos(), value, array.len));
                        return None;
                    }
                }
                Ok(Some(_)) => unreachable!("the index was type-checked as INTEGER"),
                Ok(None) => {}
                Err(diag) => {
                    self.diags.push(diag);
                    return None;
                }
            }
        }
        let len = base
            .shape
            .first()
            .cloned()
            .expect("an array place has a length");
        let remaining_shape = base.shape[1..].to_vec();
        let dynamic_count = elem.open_rank();
        let fixed_base = open_base(&elem);
        let dst = self.temp();
        self.emit(ir::Inst::Index {
            dst,
            base: base.addr,
            index,
            len,
            stride: fixed_base.size(),
            dynamic_stride: remaining_shape[..dynamic_count].to_vec(),
            site: self.site(expr.pos()),
        });
        Some(Place {
            addr: ir::Addr::Temp(dst),
            ty: elem.clone(),
            shape: remaining_shape,
            // Report 9.1: an imported variable is read-only, and selecting
            // part of it does not make that part writable.
            read_only: base.read_only,
            dynamic: elem
                .record()
                .map(|record| RecordDynamic::Static(record.descriptor.clone())),
        })
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
        self.resolve_type_named(source, None)
    }

    fn resolve_formal_type(&mut self, section: &ast::FpSection) -> Option<Type> {
        let mut ty = self.resolve_type(&section.ty)?;
        for _ in section.open_arrays.iter().rev() {
            ty = Type::OpenArray(Box::new(ty));
        }
        Some(ty)
    }

    // `name` is the type declaration this constructor is the right side of, if
    // any. A record descriptor remembers it for diagnostics, and a type
    // declaration resolves its right side before declaring its name, so the
    // name cannot be read back out of the scope afterwards and is threaded in
    // here instead. Only the outermost constructor gets it: in
    // `TYPE T = ARRAY 4 OF RECORD ... END` the name belongs to the array.
    fn resolve_type_named(&mut self, source: &ast::TypeExpr, name: Option<&str>) -> Option<Type> {
        match source {
            ast::TypeExpr::Record { base, fields, pos } => {
                self.record_type(base.as_ref(), fields, name, *pos)
            }
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
            ast::TypeExpr::Array { lengths, elem, pos } => {
                // Every length is folded even after one of them fails, so a
                // declaration reports all of its bad dimensions at once.
                let folded: Vec<_> = lengths
                    .iter()
                    .map(|length| self.array_length(length))
                    .collect();
                let elem = self.resolve_type(elem);
                let mut ty = elem?;
                // Report 6.2: ARRAY N0, N1 OF T means ARRAY N0 OF ARRAY N1 OF
                // T, so the innermost dimension is built first and the two
                // spellings produce the same descriptors.
                for len in folded.into_iter().rev() {
                    ty = self.new_array(len?, ty, *pos)?;
                }
                Some(ty)
            }
            ast::TypeExpr::Pointer { base, pos } => self.pointer_type(base, name, *pos),
            ast::TypeExpr::Procedure { params, ret, .. } => {
                self.resolve_procedure_signature(params, ret.as_deref()).0
            }
        }
    }

    // Procedure declarations and procedure type expressions use this one
    // source-signature builder. Formal names are checked here and discarded;
    // the resulting type records only mode, type, order, and result.
    fn resolve_procedure_signature(
        &mut self,
        sections: &[ast::FpSection],
        result: Option<&ast::TypeExpr>,
    ) -> ResolvedProcedure {
        let mut formals = Vec::new();
        let mut names = HashSet::new();
        let mut ok = true;
        for section in sections {
            let ty = self.resolve_formal_type(section);
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
                Some(ty) if ty.structured() => {
                    let kind = if ty.is_array() { "array" } else { "record" };
                    self.diags.push(Diagnostic::new(
                        source.pos(),
                        format!("a procedure cannot have the {kind} result type {ty}"),
                    ));
                    ok = false;
                    None
                }
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

    fn pointer_type(
        &mut self,
        source: &ast::TypeExpr,
        name: Option<&str>,
        _pos: Pos,
    ) -> Option<Type> {
        if let ast::TypeExpr::Named(designator) = source
            && designator.selectors.is_empty()
        {
            match self.resolve(designator) {
                Ok(Symbol::TypeName(Type::Record(record))) => {
                    return Some(Type::Pointer(Rc::new(PointerType {
                        name: name.map(str::to_string),
                        base: RefCell::new(PointerBase::Resolved(record)),
                    })));
                }
                Ok(Symbol::TypeName(ty)) => {
                    self.diags.push(Diagnostic::new(
                        source.pos(),
                        format!("pointer base must be a record type, found {ty}"),
                    ));
                    return Some(Type::Pointer(Rc::new(PointerType {
                        name: name.map(str::to_string),
                        base: RefCell::new(PointerBase::Invalid),
                    })));
                }
                Ok(_) => {
                    self.diags.push(Diagnostic::new(
                        source.pos(),
                        format!("'{}' is not a type", designator.name()),
                    ));
                    return Some(Type::Pointer(Rc::new(PointerType {
                        name: name.map(str::to_string),
                        base: RefCell::new(PointerBase::Invalid),
                    })));
                }
                Err(_) if self.allow_pointer_forward => {
                    let pointer = Rc::new(PointerType {
                        name: name.map(str::to_string),
                        base: RefCell::new(PointerBase::Pending {
                            name: designator.ident.clone(),
                            pos: designator.pos,
                        }),
                    });
                    self.pending_pointers.push(pointer.clone());
                    return Some(Type::Pointer(pointer));
                }
                Err(diag) => {
                    self.diags.push(diag);
                    return Some(Type::Pointer(Rc::new(PointerType {
                        name: name.map(str::to_string),
                        base: RefCell::new(PointerBase::Invalid),
                    })));
                }
            }
        }

        let state = match self.resolve_type(source) {
            Some(Type::Record(record)) => PointerBase::Resolved(record),
            Some(found) => {
                self.diags.push(Diagnostic::new(
                    source.pos(),
                    format!("pointer base must be a record type, found {found}"),
                ));
                PointerBase::Invalid
            }
            None => PointerBase::Invalid,
        };
        Some(Type::Pointer(Rc::new(PointerType {
            name: name.map(str::to_string),
            base: RefCell::new(state),
        })))
    }

    // Report 6.2: a length is a constant expression, and Report 5 leaves it an
    // INTEGER. Zero is an ordinary length; a negative one has no meaning.
    fn array_length(&mut self, expr: &ast::Expr) -> Option<i64> {
        self.check_const_expr(expr)?;
        match self.eval_const(expr) {
            Ok(ConstValue::Int(len)) if len >= 0 => Some(len),
            Ok(ConstValue::Int(len)) => {
                self.diags.push(Diagnostic::new(
                    expr.pos(),
                    format!("array length must not be negative, found {len}"),
                ));
                None
            }
            Ok(other) => {
                self.diags.push(Diagnostic::new(
                    expr.pos(),
                    format!("array length must be INTEGER, found {}", other.ty()),
                ));
                None
            }
            Err(diag) => {
                self.diags.push(diag);
                None
            }
        }
    }

    // One ARRAY constructor, and therefore one new type identity. The size is
    // computed and checked here so nothing downstream has to wonder whether an
    // array's bytes fit in the arithmetic it uses.
    fn new_array(&mut self, len: i64, elem: Type, pos: Pos) -> Option<Type> {
        let size = len
            .checked_mul(elem.size())
            .filter(|size| *size <= ir::MAX_OBJECT_SIZE);
        let Some(size) = size else {
            self.diags.push(Diagnostic::new(
                pos,
                "array type exceeds target object-size limit",
            ));
            return None;
        };
        Some(Type::Array(Rc::new(ArrayType { len, elem, size })))
    }

    // One RECORD constructor, and therefore one new type identity. Field types
    // are resolved here, at the point the constructor is read, so every one of
    // them is already declared: no legal source can name a type before its
    // declaration, and `TYPE T = RECORD f: T END` is an undeclared identifier
    // rather than a cycle. The layout below relies on that.
    fn record_type(
        &mut self,
        base_name: Option<&ast::Designator>,
        lists: &[ast::FieldList],
        name: Option<&str>,
        pos: Pos,
    ) -> Option<Type> {
        // Report 6.3 leaves layout to the implementation. Fields go in
        // declaration order, each at the next offset that is a multiple of its
        // type's alignment; the record's alignment is the largest among its
        // fields and its size is rounded up to that, so an array of records
        // strides correctly. That is the layout a C compiler gives the same
        // struct, which is worth having when the runtime is C. Project
        // Oberon's rule instead aligns everything wider than a byte to four
        // and rounds every size up to four, which would make ARRAY 3 OF CHAR
        // occupy four bytes; this compiler keeps the exact-size array rule.
        let base = match base_name {
            Some(base) => match self.resolve(base) {
                Ok(Symbol::TypeName(Type::Record(record))) => Some(record),
                Ok(Symbol::TypeName(found)) => {
                    self.diags.push(Diagnostic::new(
                        base.pos,
                        format!("record base must be a record type, found {found}"),
                    ));
                    return None;
                }
                Ok(_) => {
                    self.diags.push(Diagnostic::new(
                        base.pos,
                        format!("'{}' is not a type", base.name()),
                    ));
                    return None;
                }
                Err(diag) => {
                    self.diags.push(diag);
                    return None;
                }
            },
            None => None,
        };
        let mut fields: Vec<Field> = Vec::new();
        let mut offset: i64 = base.as_ref().map_or(0, |base| base.size);
        let mut align: i64 = base.as_ref().map_or(1, |base| base.align);
        let mut ok = true;
        for list in lists {
            // Every field list is resolved even after one of them fails, so a
            // record reports all of its bad field types at once.
            let Some(ty) = self.resolve_type(&list.ty) else {
                ok = false;
                continue;
            };
            for id in &list.names {
                if fields.iter().any(|field| field.name == id.name)
                    || base
                        .as_deref()
                        .is_some_and(|base| find_field(base, &id.name, &self.module).is_some())
                {
                    self.diags.push(Diagnostic::new(
                        id.pos,
                        format!("field '{}' is already declared", id.name),
                    ));
                    ok = false;
                    continue;
                }
                // A record declared inside a procedure can never be visible
                // outside its module, so a mark on one of its fields could
                // never mean anything. cf. ORP.CheckExport's "remove
                // asterisk", which this reuses along with its wording.
                let export = self.check_export(id);
                let field_align = ty.align();
                align = align.max(field_align);
                offset = (offset + field_align - 1) / field_align * field_align;
                fields.push(Field {
                    name: id.name.clone(),
                    ty: ty.clone(),
                    offset,
                    export,
                });
                offset += ty.size();
                // Checked as the layout grows, so the sum cannot run away:
                // every field is itself within the limit, so the total stops
                // at most one field past it.
                if offset > ir::MAX_OBJECT_SIZE {
                    self.diags.push(Diagnostic::new(
                        pos,
                        "record type exceeds target object-size limit",
                    ));
                    return None;
                }
            }
        }
        if !ok {
            return None;
        }
        let size = (offset + align - 1) / align * align;
        if size > ir::MAX_OBJECT_SIZE {
            self.diags.push(Diagnostic::new(
                pos,
                "record type exceeds target object-size limit",
            ));
            return None;
        }
        let contains_pointers = base.as_ref().is_some_and(|base| base.contains_pointers)
            || fields.iter().any(|field| field.ty.contains_pointers());
        let descriptor = format!(".{}.type{}", self.module, self.descriptors.len());
        let record = Rc::new(RecordType {
            fields,
            size,
            align,
            name: name.map(str::to_string),
            module: self.module.clone(),
            base: base.clone(),
            descriptor: descriptor.clone(),
            contains_pointers,
        });
        self.descriptors.push(ir::Descriptor {
            symbol: descriptor,
            base: base.map(|base| base.descriptor.clone()),
        });
        Some(Type::Record(record))
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
    // can carry selectors, and applying an index selector emits code.
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

    // `ty` is the operand type, which is what the emitter needs to choose
    // between an integer and a double-precision instruction.
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

    fn materialize_descriptor(&mut self, place: &Place) -> ir::Value {
        match place
            .dynamic
            .as_ref()
            .expect("a record place carries its dynamic descriptor")
            .clone()
        {
            RecordDynamic::Static(symbol) => ir::Value::Symbol(symbol),
            RecordDynamic::Incoming(value) => value,
            RecordDynamic::Heap(pointer) => {
                let dst = self.temp();
                self.emit(ir::Inst::HeapDescriptor { dst, pointer });
                ir::Value::Temp(dst)
            }
        }
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
    // The aligned bytes this procedure's parameters and locals have reserved.
    frame: i64,
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
            frame: 0,
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

// The two uses of the 0-through-255 domain. Each has its own trap and its own
// message, matching how every existing dynamic check has its own line.
#[derive(Clone, Copy)]
enum ByteDomain {
    Store,
    Chr,
}

impl ByteDomain {
    fn describe(self) -> &'static str {
        match self {
            ByteDomain::Store => "BYTE value",
            ByteDomain::Chr => "CHR argument",
        }
    }

    fn trap(self) -> &'static str {
        match self {
            ByteDomain::Store => "oberon_byte_range",
            ByteDomain::Chr => "oberon_chr_range",
        }
    }

    fn label(self) -> &'static str {
        match self {
            ByteDomain::Store => "byte",
            ByteDomain::Chr => "chr",
        }
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
// selector on a variable never comes here: that is a field or an index, and
// the designator walks handle both.
fn selector_error(designator: &ast::Designator, selector: &ast::Selector) -> Diagnostic {
    match selector {
        ast::Selector::Field(name, pos) => Diagnostic::new(
            *pos,
            format!(
                "'{name}' cannot be selected from '{}' here",
                designator.ident
            ),
        ),
        ast::Selector::Index(_, pos) => Diagnostic::new(
            *pos,
            format!("'{}' cannot be indexed here", designator.ident),
        ),
        ast::Selector::Deref(pos) => Diagnostic::new(
            *pos,
            format!("'{}' cannot be dereferenced here", designator.ident),
        ),
        ast::Selector::Guard(_, pos) => Diagnostic::new(
            *pos,
            format!("'{}' cannot be type-guarded here", designator.ident),
        ),
    }
}

// An absent field and a private one get the same message, so a client cannot
// learn a private field's name from a diagnostic.
fn no_such_field(pos: Pos, name: &str, ty: &Type) -> Diagnostic {
    Diagnostic::new(
        pos,
        format!("cannot select '{name}': {ty} has no such field"),
    )
}

// Report 9.1 requires the same type on both sides. Two structured types can
// print the same and still be different types, so when the spellings agree the
// message has to say what the difference is.
fn assign_error(pos: Pos, target: &Type, found: &Type) -> Diagnostic {
    Diagnostic::new(
        pos,
        format!(
            "cannot assign {found} to {target}{}",
            distinct_types_hint(target, found)
        ),
    )
}

fn argument_type_error(pos: Pos, number: usize, expected: &Type, found: &Type) -> Diagnostic {
    Diagnostic::new(
        pos,
        format!(
            "argument {number} has type {found}, expected {expected}{}",
            distinct_types_hint(expected, found)
        ),
    )
}

// Two arrays with the same shape, two records with the same declared name, or
// two inline records all print alike, so a diagnostic comparing two of them
// has to name the rule that keeps them apart.
fn distinct_types_hint(target: &Type, found: &Type) -> &'static str {
    if target.to_string() != found.to_string() {
        return "";
    }
    match target {
        Type::Record(_) => {
            ": these are different record types, and each RECORD in the source declares its own"
        }
        Type::Array(_) => {
            ": these are different array types, and each ARRAY in the source declares its own"
        }
        Type::Pointer(_) => {
            ": these are different pointer types, and each POINTER in the source declares its own"
        }
        Type::Procedure(_) => {
            ": these are different procedure type constructors, and this context requires an identical type"
        }
        _ => "",
    }
}

fn index_range_error(pos: Pos, index: i64, len: i64) -> Diagnostic {
    Diagnostic::new(
        pos,
        format!("index {index} is out of bounds: the array has length {len}"),
    )
}

fn shift_range_error(pos: Pos, count: i64) -> Diagnostic {
    Diagnostic::new(
        pos,
        format!("shift count {count} is out of range: must be between 0 and 63"),
    )
}

fn set_element_range_error(pos: Pos, element: i64) -> Diagnostic {
    Diagnostic::new(
        pos,
        format!("set element {element} is out of range: must be between 0 and {SET_MAX}"),
    )
}

// The bits of {low .. high}, which is empty when the range is reversed. The
// runtime form in Analyzer::set_range computes the same intersection.
fn set_range_bits(low: i64, high: i64) -> u64 {
    (SET_FULL << low) & (SET_FULL >> (SET_MAX - high))
}

fn label_text(low: i64, high: i64) -> String {
    if low == high {
        format!("case label {low}")
    } else {
        format!("case labels {low}..{high}")
    }
}

fn unary_type_error(pos: Pos, op: ast::UnOp, found: &Type) -> Diagnostic {
    let (name, expected) = match op {
        ast::UnOp::Plus => ("+", "INTEGER or REAL"),
        ast::UnOp::Neg => ("-", "INTEGER, REAL, or SET"),
        ast::UnOp::Not => ("~", "BOOLEAN"),
    };
    Diagnostic::new(
        pos,
        format!("operator '{name}' requires {expected}, found {found}"),
    )
}
