use std::fmt;
use std::rc::Rc;

use crate::ast;
use crate::diag::{Diagnostic, Pos};
use crate::ir;

pub(super) fn bin_op_name(op: ast::BinOp) -> &'static str {
    match op {
        ast::BinOp::Add => "+",
        ast::BinOp::Sub => "-",
        ast::BinOp::Mul => "*",
        ast::BinOp::Slash => "/",
        ast::BinOp::In => "IN",
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

// Report 9.1's assignment compatibility, with each outcome named. The
// assignment statement, the value parameter, and the RETURN expression all
// ask this one function and act on its answer; a variable parameter does not,
// because Report 10.1 demands an identical type there.
pub(super) enum AssignKind {
    // The same scalar type on both sides: one ordinary store.
    Store,
}

pub(super) fn assign_kind(target: &Type, found: &Type) -> Option<AssignKind> {
    if target == found {
        return Some(AssignKind::Store);
    }
    None
}

// Report 8.2 overloads "+", "-", "*", and "/". The first three take two
// INTEGERs or two SETs; "/" means symmetric set difference and has no INTEGER
// meaning. The operation is chosen from the operand types, not from the
// token, so the AST keeps the source operator.

pub(super) fn check_arith_types(
    pos: Pos,
    op: ast::BinOp,
    lhs: Type,
    rhs: Type,
    diags: &mut Vec<Diagnostic>,
) -> Option<Type> {
    let accepted: &[Type] = if op == ast::BinOp::Slash {
        &[Type::Set]
    } else {
        &[Type::Integer, Type::Set]
    };
    check_operand_types(pos, op, accepted, lhs, rhs, diags)
}

// Report 8.2.4 orders INTEGER; SET and BOOLEAN have equality and inequality
// only. The operand type is returned rather than
// BOOLEAN, because the comparison instruction is chosen from it.
pub(super) fn check_order_types(
    pos: Pos,
    op: ast::BinOp,
    lhs: Type,
    rhs: Type,
    diags: &mut Vec<Diagnostic>,
) -> Option<Type> {
    check_operand_types(pos, op, &[Type::Integer], lhs, rhs, diags)
}

pub(super) fn check_operand_types(
    pos: Pos,
    op: ast::BinOp,
    accepted: &[Type],
    lhs: Type,
    rhs: Type,
    diags: &mut Vec<Diagnostic>,
) -> Option<Type> {
    if lhs == rhs && accepted.contains(&lhs) {
        return Some(lhs);
    }
    let expected = accepted
        .iter()
        .map(|ty| format!("two {ty}"))
        .collect::<Vec<_>>();
    let expected = match expected.split_last() {
        Some((last, [])) => last.clone(),
        Some((last, rest)) if rest.len() == 1 => format!("{} or {last}", rest[0]),
        Some((last, rest)) => format!("{}, or {last}", rest.join(", ")),
        None => unreachable!("an operator accepts at least one operand type"),
    };
    diags.push(Diagnostic::new(
        pos,
        format!(
            "operator '{}' requires {expected} operands, found {lhs} and {rhs}",
            bin_op_name(op)
        ),
    ));
    None
}

pub(super) fn check_binary_types(
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

#[derive(Debug, Clone)]
pub enum Type {
    Integer,
    Boolean,
    Set,
    // A declared procedure's signature. No source names it, so it has no
    // value and no storage.
    Procedure(Rc<ProcedureType>),
}

#[derive(Debug)]
pub struct ProcedureType {
    pub(super) params: Vec<ProcedureParam>,
    pub(super) ret: Option<Type>,
}

#[derive(Debug)]
pub(super) struct ProcedureParam {
    pub(super) var: bool,
    pub(super) ty: Type,
}

impl Type {
    // The IR type of a value of this type, when a value of it exists.
    pub(super) fn scalar(&self) -> Option<ir::Ty> {
        match self {
            Type::Integer => Some(ir::Ty::Int),
            Type::Boolean => Some(ir::Ty::Bool),
            Type::Set => Some(ir::Ty::Set),
            Type::Procedure(_) => None,
        }
    }

    pub(super) fn ir(&self) -> ir::Ty {
        self.scalar()
            .expect("a scalar type reached a value operation")
    }

    pub(super) fn procedure(&self) -> Option<&Rc<ProcedureType>> {
        match self {
            Type::Procedure(procedure) => Some(procedure),
            _ => None,
        }
    }

    pub(super) fn storage(&self) -> ir::Storage {
        ir::Storage::Scalar(self.ir())
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Type) -> bool {
        match (self, other) {
            (Type::Integer, Type::Integer)
            | (Type::Boolean, Type::Boolean)
            | (Type::Set, Type::Set) => true,
            (Type::Procedure(a), Type::Procedure(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Integer => write!(f, "INTEGER"),
            Type::Boolean => write!(f, "BOOLEAN"),
            Type::Set => write!(f, "SET"),
            Type::Procedure(procedure) => {
                write!(f, "PROCEDURE")?;
                if !procedure.params.is_empty() || procedure.ret.is_some() {
                    write!(f, "(")?;
                    for (i, param) in procedure.params.iter().enumerate() {
                        if i != 0 {
                            write!(f, "; ")?;
                        }
                        if param.var {
                            write!(f, "VAR ")?;
                        }
                        write!(f, "{}", param.ty)?;
                    }
                    write!(f, ")")?;
                }
                if let Some(ret) = &procedure.ret {
                    write!(f, ": {ret}")?;
                }
                Ok(())
            }
        }
    }
}
