use std::collections::HashMap;

use crate::ast;
use crate::diag::{Diagnostic, Pos};

pub type Scope = HashMap<String, Symbol>;

#[derive(Debug, Clone)]
pub enum Symbol {
    Const(i32),                                  // folded at declaration
    Var,                                         // INTEGER-only this slice
    Proc { runtime_name: String, arity: usize }, // e.g. "oberon_out_int"
    Module(HashMap<String, Symbol>),             // "Out" pseudo-scope
    TypeName,                                    // pre-seeded "INTEGER"
}

pub fn analyze(module: &ast::Module) -> (Scope, Vec<Diagnostic>) {
    let mut scope: Scope = HashMap::new();
    scope.insert("INTEGER".into(), Symbol::TypeName);
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
        match eval_const(&c.expr, &scope) {
            Ok(v) => declare(&mut scope, &c.name, c.pos, Symbol::Const(v), &mut diags),
            Err(d) => diags.push(d),
        }
    }

    for v in &module.vars {
        match resolve(&scope, &v.ty) {
            Ok(Symbol::TypeName) => {}
            Ok(_) => diags.push(Diagnostic::new(
                v.ty.pos,
                format!("'{}' is not a type", v.ty.name()),
            )),
            Err(d) => diags.push(d),
        }
        for (name, pos) in &v.names {
            declare(&mut scope, name, *pos, Symbol::Var, &mut diags);
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
                arity: 2,
            },
        ),
        (
            "Ln".into(),
            Symbol::Proc {
                runtime_name: "oberon_out_ln".into(),
                arity: 0,
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

fn eval_const(e: &ast::Expr, scope: &Scope) -> Result<i32, Diagnostic> {
    match e {
        ast::Expr::Int { value, pos } => {
            i32::try_from(*value).map_err(|_| Diagnostic::new(*pos, "integer literal out of range"))
        }
        ast::Expr::Name(d) => match resolve(scope, d)? {
            Symbol::Const(v) => Ok(*v),
            _ => Err(Diagnostic::new(
                d.pos,
                format!("'{}' is not a constant", d.name()),
            )),
        },
        ast::Expr::Unary {
            op: ast::UnOp::Neg,
            expr,
            pos,
        } => {
            let v = eval_const(expr, scope)?;
            v.checked_neg()
                .ok_or_else(|| Diagnostic::new(*pos, "constant expression overflows"))
        }
        ast::Expr::Binary { op, lhs, rhs, .. } => {
            let l = eval_const(lhs, scope)?;
            let r = eval_const(rhs, scope)?;
            if r == 0 && matches!(op, ast::BinOp::Div | ast::BinOp::Mod) {
                return Err(Diagnostic::new(e.pos(), "constant DIV or MOD by zero"));
            }
            let v = match op {
                ast::BinOp::Add => l.checked_add(r),
                ast::BinOp::Sub => l.checked_sub(r),
                ast::BinOp::Mul => l.checked_mul(r),
                // Report 8.2.2 requires 0 <= x MOD y < y, so DIV floors instead
                // of truncating toward zero. This is deliberately the same
                // ((x REM y) + y) REM y that qbe.rs emits, spelled out here so a
                // folded constant and the same expression computed at runtime
                // can never disagree. cf. obnc lib/obnc/OBNC.h OBNC_MOD.
                ast::BinOp::Div | ast::BinOp::Mod => {
                    let m = l
                        .checked_rem(r)
                        .and_then(|m| m.checked_add(r))
                        .and_then(|m| m.checked_rem(r));
                    match op {
                        ast::BinOp::Mod => m,
                        _ => m
                            .and_then(|m| l.checked_sub(m))
                            .and_then(|n| n.checked_div(r)),
                    }
                }
            };
            v.ok_or_else(|| Diagnostic::new(e.pos(), "constant expression overflows"))
        }
    }
}

fn check_stmt(stmt: &ast::Stmt, scope: &Scope, diags: &mut Vec<Diagnostic>) {
    match stmt {
        ast::Stmt::Assign { lhs, rhs, .. } => {
            match resolve(scope, lhs) {
                Ok(Symbol::Var) => {}
                Ok(_) => diags.push(Diagnostic::new(
                    lhs.pos,
                    format!("cannot assign to '{}'", lhs.name()),
                )),
                Err(d) => diags.push(d),
            }
            check_expr(rhs, scope, diags);
        }
        ast::Stmt::Call { proc, args, pos } => {
            match resolve(scope, proc) {
                Ok(Symbol::Proc { arity, .. }) => {
                    if args.len() != *arity {
                        diags.push(Diagnostic::new(
                            *pos,
                            format!(
                                "wrong number of arguments: expected {arity}, found {}",
                                args.len()
                            ),
                        ));
                    }
                }
                Ok(_) => diags.push(Diagnostic::new(
                    *pos,
                    format!("'{}' is not a procedure", proc.name()),
                )),
                Err(d) => diags.push(d),
            }
            for arg in args {
                check_expr(arg, scope, diags);
            }
        }
    }
}

// Everything is INTEGER this slice, so "type checking" is only about symbols
// used as values and literal ranges.
fn check_expr(e: &ast::Expr, scope: &Scope, diags: &mut Vec<Diagnostic>) {
    match e {
        ast::Expr::Int { value, pos } => {
            if i32::try_from(*value).is_err() {
                diags.push(Diagnostic::new(*pos, "integer literal out of range"));
            }
        }
        ast::Expr::Name(d) => match resolve(scope, d) {
            Ok(Symbol::Var | Symbol::Const(_)) => {}
            Ok(_) => diags.push(Diagnostic::new(
                d.pos,
                format!("'{}' cannot be used as a value", d.name()),
            )),
            Err(diag) => diags.push(diag),
        },
        ast::Expr::Unary { expr, .. } => check_expr(expr, scope, diags),
        ast::Expr::Binary { lhs, rhs, .. } => {
            check_expr(lhs, scope, diags);
            check_expr(rhs, scope, diags);
        }
    }
}
