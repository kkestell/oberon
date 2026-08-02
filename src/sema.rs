use std::collections::HashMap;
use std::fmt;

use crate::ast;
use crate::diag::{Diagnostic, Pos};

pub type Scope = HashMap<String, Symbol>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
    Integer,
    Boolean,
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
pub enum Value {
    Int(i32),
    Bool(bool),
}

impl Value {
    fn ty(self) -> Type {
        match self {
            Value::Int(_) => Type::Integer,
            Value::Bool(_) => Type::Boolean,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Symbol {
    Const(Value),
    Var(Type),
    Proc {
        runtime_name: String,
        params: Vec<Type>,
    },
    Module(HashMap<String, Symbol>), // "Out" pseudo-scope
    TypeName(Type),
}

pub fn analyze(module: &ast::Module) -> (Scope, Vec<Diagnostic>) {
    let mut scope: Scope = HashMap::new();
    scope.insert("INTEGER".into(), Symbol::TypeName(Type::Integer));
    scope.insert("BOOLEAN".into(), Symbol::TypeName(Type::Boolean));
    let mut diags = Vec::new();

    for import in &module.imports {
        if import.name == "Out" {
            let visible = import.alias.clone().unwrap_or_else(|| import.name.clone());
            declare(
                &mut scope,
                &visible,
                import.pos,
                Symbol::Module(out_scope()),
                &mut diags,
            );
        } else {
            diags.push(Diagnostic::new(
                import.pos,
                format!("unknown module '{}'", import.name),
            ));
        }
    }

    // Folded in declaration order, so CONST A = 2; B = A * 3 works.
    for c in &module.consts {
        // Type-check the whole expression before folding it. Folding still
        // short-circuits, but an unreachable right operand must be well-typed.
        if check_expr(&c.expr, &scope, &mut diags).is_none() {
            continue;
        }
        match eval_const(&c.expr, &scope) {
            Ok(v) => declare(&mut scope, &c.name, c.pos, Symbol::Const(v), &mut diags),
            Err(d) => diags.push(d),
        }
    }

    for v in &module.vars {
        let ty = match resolve(&scope, &v.ty) {
            Ok(Symbol::TypeName(ty)) => Some(*ty),
            Ok(_) => {
                diags.push(Diagnostic::new(
                    v.ty.pos,
                    format!("'{}' is not a type", v.ty.name()),
                ));
                None
            }
            Err(d) => {
                diags.push(d);
                None
            }
        };
        if let Some(ty) = ty {
            for (name, pos) in &v.names {
                declare(&mut scope, name, *pos, Symbol::Var(ty), &mut diags);
            }
        }
    }

    for stmt in &module.body {
        check_stmt(stmt, &scope, &mut diags);
    }

    (scope, diags)
}

// Report 4: "No identifier may denote more than one object within a given
// scope." A plain insert would let a redeclaration silently overwrite, which
// costs a VAR its storage to a later one of the same name and can knock an
// imported module out of scope entirely. First declaration wins, so the rest
// of the module still checks against something sensible.
fn declare(scope: &mut Scope, name: &str, pos: Pos, sym: Symbol, diags: &mut Vec<Diagnostic>) {
    if scope.contains_key(name) {
        diags.push(Diagnostic::new(
            pos,
            format!("'{name}' is already declared"),
        ));
    } else {
        scope.insert(name.to_string(), sym);
    }
}

fn out_scope() -> HashMap<String, Symbol> {
    HashMap::from([
        (
            "Int".into(),
            Symbol::Proc {
                runtime_name: "oberon_out_int".into(),
                params: vec![Type::Integer, Type::Integer],
            },
        ),
        (
            "Ln".into(),
            Symbol::Proc {
                runtime_name: "oberon_out_ln".into(),
                params: Vec::new(),
            },
        ),
    ])
}

// Designator resolution, shared with codegen: the parser cannot tell module
// qualification from field selection, so this is where "Out.Int" becomes a
// member lookup.
pub fn resolve<'a>(scope: &'a Scope, d: &ast::Designator) -> Result<&'a Symbol, Diagnostic> {
    let mut sym = scope
        .get(&d.ident)
        .ok_or_else(|| Diagnostic::new(d.pos, format!("undeclared identifier '{}'", d.ident)))?;
    for sel in &d.selectors {
        match (sym, sel) {
            (Symbol::Module(members), ast::Selector::Field(name, pos)) => {
                sym = members.get(name).ok_or_else(|| {
                    Diagnostic::new(
                        *pos,
                        format!("'{}' is not declared in module '{}'", name, d.ident),
                    )
                })?;
            }
            (_, ast::Selector::Field(name, pos)) => {
                return Err(Diagnostic::new(
                    *pos,
                    format!(
                        "cannot select '{name}' from '{}': record field selection is not yet supported",
                        d.ident
                    ),
                ));
            }
        }
    }
    Ok(sym)
}

fn eval_const(e: &ast::Expr, scope: &Scope) -> Result<Value, Diagnostic> {
    match e {
        ast::Expr::Int { value, pos } => i32::try_from(*value)
            .map(Value::Int)
            .map_err(|_| Diagnostic::new(*pos, "integer literal out of range")),
        ast::Expr::Bool { value, .. } => Ok(Value::Bool(*value)),
        ast::Expr::Name(d) => match resolve(scope, d)? {
            Symbol::Const(v) => Ok(*v),
            _ => Err(Diagnostic::new(
                d.pos,
                format!("'{}' is not a constant", d.name()),
            )),
        },
        // Type errors can't happen past here: analyze() runs check_expr on the
        // whole constant expression before folding it, so a Value of the wrong
        // shape is a compiler bug, not a user error.
        ast::Expr::Unary { op, expr, pos } => {
            let v = eval_const(expr, scope)?;
            match (op, v) {
                (ast::UnOp::Neg, Value::Int(v)) => v
                    .checked_neg()
                    .map(Value::Int)
                    .ok_or_else(|| Diagnostic::new(*pos, "constant expression overflows")),
                (ast::UnOp::Not, Value::Bool(v)) => Ok(Value::Bool(!v)),
                _ => unreachable!("type-checked before folding"),
            }
        }
        ast::Expr::Binary {
            op: ast::BinOp::And,
            lhs,
            rhs,
            ..
        } => match eval_const(lhs, scope)? {
            Value::Bool(false) => Ok(Value::Bool(false)),
            Value::Bool(true) => eval_const(rhs, scope),
            Value::Int(_) => unreachable!("type-checked before folding"),
        },
        ast::Expr::Binary {
            op: ast::BinOp::Or,
            lhs,
            rhs,
            ..
        } => match eval_const(lhs, scope)? {
            Value::Bool(true) => Ok(Value::Bool(true)),
            Value::Bool(false) => eval_const(rhs, scope),
            Value::Int(_) => unreachable!("type-checked before folding"),
        },
        ast::Expr::Binary { op, lhs, rhs, pos } => {
            let l = eval_const(lhs, scope)?;
            let r = eval_const(rhs, scope)?;
            eval_const_binary(*op, l, r, *pos)
        }
    }
}

fn eval_const_binary(op: ast::BinOp, l: Value, r: Value, pos: Pos) -> Result<Value, Diagnostic> {
    use ast::BinOp;
    match op {
        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
            let (Value::Int(l), Value::Int(r)) = (l, r) else {
                unreachable!("type-checked before folding");
            };
            if r == 0 && matches!(op, BinOp::Div | BinOp::Mod) {
                return Err(Diagnostic::new(pos, "constant DIV or MOD by zero"));
            }
            let v = match op {
                BinOp::Add => l.checked_add(r),
                BinOp::Sub => l.checked_sub(r),
                BinOp::Mul => l.checked_mul(r),
                // Report 8.2.2 requires 0 <= x MOD y < y, so DIV floors instead
                // of truncating toward zero. This is deliberately the same
                // ((x REM y) + y) REM y that qbe.rs emits, spelled out here so a
                // folded constant and the same expression computed at runtime
                // can never disagree. cf. obnc lib/obnc/OBNC.h OBNC_MOD.
                BinOp::Div | BinOp::Mod => {
                    let m = l
                        .checked_rem(r)
                        .and_then(|m| m.checked_add(r))
                        .and_then(|m| m.checked_rem(r));
                    match op {
                        BinOp::Mod => m,
                        _ => m
                            .and_then(|m| l.checked_sub(m))
                            .and_then(|n| n.checked_div(r)),
                    }
                }
                _ => unreachable!(),
            };
            v.map(Value::Int)
                .ok_or_else(|| Diagnostic::new(pos, "constant expression overflows"))
        }
        BinOp::Eq | BinOp::Ne => {
            assert_eq!(l.ty(), r.ty(), "type-checked before folding");
            let equal = l == r;
            Ok(Value::Bool(if op == BinOp::Eq { equal } else { !equal }))
        }
        BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
            let (Value::Int(l), Value::Int(r)) = (l, r) else {
                unreachable!("type-checked before folding");
            };
            Ok(Value::Bool(match op {
                BinOp::Lt => l < r,
                BinOp::Le => l <= r,
                BinOp::Gt => l > r,
                BinOp::Ge => l >= r,
                _ => unreachable!(),
            }))
        }
        BinOp::And | BinOp::Or => unreachable!("short-circuit operators handled by eval_const"),
    }
}

fn check_stmt(stmt: &ast::Stmt, scope: &Scope, diags: &mut Vec<Diagnostic>) {
    match stmt {
        ast::Stmt::Assign { lhs, rhs, .. } => {
            let lhs_ty = match resolve(scope, lhs) {
                Ok(Symbol::Var(ty)) => Some(*ty),
                Ok(_) => {
                    diags.push(Diagnostic::new(
                        lhs.pos,
                        format!("cannot assign to '{}'", lhs.name()),
                    ));
                    None
                }
                Err(d) => {
                    diags.push(d);
                    None
                }
            };
            let rhs_ty = check_expr(rhs, scope, diags);
            if let (Some(lhs_ty), Some(rhs_ty)) = (lhs_ty, rhs_ty)
                && lhs_ty != rhs_ty
            {
                diags.push(Diagnostic::new(
                    rhs.pos(),
                    format!("cannot assign {rhs_ty} to {lhs_ty}"),
                ));
            }
        }
        ast::Stmt::Call { proc, args, pos } => {
            let params = match resolve(scope, proc) {
                Ok(Symbol::Proc { params, .. }) => Some(params.clone()),
                Ok(_) => {
                    diags.push(Diagnostic::new(
                        *pos,
                        format!("'{}' is not a procedure", proc.name()),
                    ));
                    None
                }
                Err(d) => {
                    diags.push(d);
                    None
                }
            };
            if let Some(params) = &params
                && args.len() != params.len()
            {
                diags.push(Diagnostic::new(
                    *pos,
                    format!(
                        "wrong number of arguments: expected {}, found {}",
                        params.len(),
                        args.len()
                    ),
                ));
            }
            for (i, arg) in args.iter().enumerate() {
                let actual = check_expr(arg, scope, diags);
                let expected = params.as_ref().and_then(|params| params.get(i)).copied();
                if let (Some(actual), Some(expected)) = (actual, expected)
                    && actual != expected
                {
                    diags.push(Diagnostic::new(
                        arg.pos(),
                        format!("argument {} has type {actual}, expected {expected}", i + 1),
                    ));
                }
            }
        }
        ast::Stmt::If {
            cond,
            then,
            elsifs,
            els,
        } => {
            check_condition(cond, scope, diags);
            check_stmts(then, scope, diags);
            for (cond, body) in elsifs {
                check_condition(cond, scope, diags);
                check_stmts(body, scope, diags);
            }
            if let Some(body) = els {
                check_stmts(body, scope, diags);
            }
        }
        ast::Stmt::While { cond, body, elsifs } => {
            check_condition(cond, scope, diags);
            check_stmts(body, scope, diags);
            for (cond, body) in elsifs {
                check_condition(cond, scope, diags);
                check_stmts(body, scope, diags);
            }
        }
        ast::Stmt::Repeat { body, cond } => {
            check_stmts(body, scope, diags);
            check_condition(cond, scope, diags);
        }
    }
}

fn check_stmts(stmts: &[ast::Stmt], scope: &Scope, diags: &mut Vec<Diagnostic>) {
    for stmt in stmts {
        check_stmt(stmt, scope, diags);
    }
}

fn check_condition(e: &ast::Expr, scope: &Scope, diags: &mut Vec<Diagnostic>) {
    if let Some(ty) = check_expr(e, scope, diags)
        && ty != Type::Boolean
    {
        diags.push(Diagnostic::new(
            e.pos(),
            format!("condition must be BOOLEAN, found {ty}"),
        ));
    }
}

fn check_expr(e: &ast::Expr, scope: &Scope, diags: &mut Vec<Diagnostic>) -> Option<Type> {
    match e {
        ast::Expr::Int { value, pos } => match i32::try_from(*value) {
            Ok(_) => Some(Type::Integer),
            Err(_) => {
                diags.push(Diagnostic::new(*pos, "integer literal out of range"));
                None
            }
        },
        ast::Expr::Bool { .. } => Some(Type::Boolean),
        ast::Expr::Name(d) => match resolve(scope, d) {
            Ok(Symbol::Var(ty)) => Some(*ty),
            Ok(Symbol::Const(v)) => Some(v.ty()),
            Ok(_) => {
                diags.push(Diagnostic::new(
                    d.pos,
                    format!("'{}' cannot be used as a value", d.name()),
                ));
                None
            }
            Err(diag) => {
                diags.push(diag);
                None
            }
        },
        ast::Expr::Unary { op, expr, pos } => {
            let found = check_expr(expr, scope, diags)?;
            let expected = match op {
                ast::UnOp::Neg => Type::Integer,
                ast::UnOp::Not => Type::Boolean,
            };
            if found == expected {
                Some(expected)
            } else {
                diags.push(unary_type_error(
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
            let lhs_ty = check_expr(lhs, scope, diags);
            let rhs_ty = check_expr(rhs, scope, diags);
            let (Some(lhs_ty), Some(rhs_ty)) = (lhs_ty, rhs_ty) else {
                return None;
            };
            use ast::BinOp;
            match op {
                BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                    check_binary_types(
                        *pos,
                        bin_op_name(*op),
                        Type::Integer,
                        lhs_ty,
                        rhs_ty,
                        Type::Integer,
                        diags,
                    )
                }
                BinOp::And | BinOp::Or => check_binary_types(
                    *pos,
                    bin_op_name(*op),
                    Type::Boolean,
                    lhs_ty,
                    rhs_ty,
                    Type::Boolean,
                    diags,
                ),
                BinOp::Eq | BinOp::Ne => {
                    if lhs_ty == rhs_ty {
                        Some(Type::Boolean)
                    } else {
                        diags.push(Diagnostic::new(
                            *pos,
                            format!(
                                "operator '{}' requires operands of the same type, found {lhs_ty} and {rhs_ty}",
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
                    lhs_ty,
                    rhs_ty,
                    Type::Boolean,
                    diags,
                ),
            }
        }
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
