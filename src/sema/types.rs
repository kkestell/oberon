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
    // Report 9.1: two identical structured types copy the whole
    // representation, padding included.
    WholeCopy,
}

pub(super) fn assign_kind(target: &Type, found: &Type) -> Option<AssignKind> {
    if target == found {
        return Some(if target.structured() {
            AssignKind::WholeCopy
        } else {
            AssignKind::Store
        });
    }
    None
}

// Report 8.2 overloads "+", "-", "*", and "/". The first three take two
// INTEGERs, two REALs, or two SETs; "/" means REAL quotient or symmetric set
// difference and has no INTEGER meaning. INTEGER and REAL never mix
// implicitly, here or anywhere else. The operation is chosen from the operand
// types, not from the token, so the AST keeps the source operator.

pub(super) fn check_arith_types(
    pos: Pos,
    op: ast::BinOp,
    lhs: Type,
    rhs: Type,
    diags: &mut Vec<Diagnostic>,
) -> Option<Type> {
    let accepted: &[Type] = if op == ast::BinOp::Slash {
        &[Type::Real, Type::Set]
    } else {
        &[Type::Integer, Type::Real, Type::Set]
    };
    check_operand_types(pos, op, accepted, lhs, rhs, diags)
}

// Report 8.2.4 orders the numeric types; SET and BOOLEAN have
// equality and inequality only. The operand type is returned rather than
// BOOLEAN, because the comparison instruction is chosen from it.
pub(super) fn check_order_types(
    pos: Pos,
    op: ast::BinOp,
    lhs: Type,
    rhs: Type,
    diags: &mut Vec<Diagnostic>,
) -> Option<Type> {
    check_operand_types(pos, op, &[Type::Integer, Type::Real], lhs, rhs, diags)
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

pub(super) fn fixed_shape(ty: &Type) -> Vec<ir::Value> {
    match ty {
        Type::Array(array) => {
            let mut shape = vec![ir::Value::Int(array.len)];
            shape.extend(fixed_shape(&array.elem));
            shape
        }
        _ => Vec::new(),
    }
}

// One object's contribution to a running frame or static-storage total, padded
// to its own alignment first. None means the total would leave what the target
// can address, which is a source error rather than something to discover in
// QBE or the linker.
pub(super) fn reserve(total: i64, ty: &Type) -> Option<i64> {
    let align = ty.align();
    let start = total.checked_add(align - 1)? / align * align;
    let end = start.checked_add(ty.size())?;
    (end <= ir::MAX_OBJECT_SIZE).then_some(end)
}

#[derive(Debug, Clone)]
pub enum Type {
    Integer,
    Real,
    Boolean,
    Set,
    // Report 6.2. Each ARRAY constructor in the source builds one descriptor,
    // and sharing that descriptor is what makes two types the same type. A
    // declaration's names, an alias, an interface member, and a client's view
    // of an exported type all hold the same handle.
    Array(Rc<ArrayType>),
    // A declared procedure's signature. No source names it, so it has no
    // value and no storage.
    Procedure(Rc<ProcedureType>),
}

#[derive(Debug)]
pub struct ArrayType {
    pub(super) len: i32,
    pub(super) elem: Type,
    // Checked against ir::MAX_OBJECT_SIZE when the descriptor was built, so
    // every later layout sum can stay ordinary i64 arithmetic.
    pub(super) size: i64,
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
    // The IR type of a value of this type, when a value of it exists. An
    // array has none: it is storage, and asking for one is how a
    // load, a store, an argument, or a result finds out it may not have this
    // type at all.
    pub(super) fn scalar(&self) -> Option<ir::Ty> {
        match self {
            Type::Integer => Some(ir::Ty::Int),
            Type::Real => Some(ir::Ty::Real),
            Type::Boolean => Some(ir::Ty::Bool),
            Type::Set => Some(ir::Ty::Set),
            Type::Array(_) | Type::Procedure(_) => None,
        }
    }

    pub(super) fn ir(&self) -> ir::Ty {
        self.scalar()
            .expect("a scalar type reached a value operation")
    }

    pub(super) fn array(&self) -> Option<&Rc<ArrayType>> {
        match self {
            Type::Array(array) => Some(array),
            _ => None,
        }
    }

    pub(super) fn array_elem(&self) -> Option<&Type> {
        match self {
            Type::Array(array) => Some(&array.elem),
            _ => None,
        }
    }

    pub(super) fn is_array(&self) -> bool {
        self.array_elem().is_some()
    }

    pub(super) fn procedure(&self) -> Option<&Rc<ProcedureType>> {
        match self {
            Type::Procedure(procedure) => Some(procedure),
            _ => None,
        }
    }

    // Report 9.1 and 10.1 say "structured (of array or record type)". These
    // are the types that live in storage and travel by address: a parameter of
    // one is a reference, and an assignment between two of them is a copy.
    pub(super) fn structured(&self) -> bool {
        matches!(self, Type::Array(_))
    }

    pub(super) fn size(&self) -> i64 {
        match self {
            Type::Array(array) => array.size,
            scalar => ir::scalar_size(scalar.ir()),
        }
    }

    // An array is contiguous and takes its element's alignment.
    pub(super) fn align(&self) -> i64 {
        match self {
            Type::Array(array) => array.elem.align(),
            scalar => ir::scalar_size(scalar.ir()),
        }
    }

    pub(super) fn storage(&self) -> ir::Storage {
        match self {
            Type::Array(array) => ir::Storage::Array {
                len: array.len,
                elem: Box::new(array.elem.storage()),
            },
            scalar => ir::Storage::Scalar(scalar.ir()),
        }
    }
}

// Report 6.2 and 9.1 ask whether two types are *the same type*, not whether
// they have the same shape. Two separately written ARRAY constructors of equal
// length and element type are different types, and only sharing a descriptor
// makes them one. cf. OBNC's Types_Same, which compares type structures by
// identity; Project Oberon additionally treats two arrays with equal length
// and the same base as compatible, which this compiler does not adopt because
// it would not extend to the equivalent nested declarations.
impl PartialEq for Type {
    fn eq(&self, other: &Type) -> bool {
        match (self, other) {
            (Type::Integer, Type::Integer)
            | (Type::Real, Type::Real)
            | (Type::Boolean, Type::Boolean)
            | (Type::Set, Type::Set) => true,
            (Type::Array(a), Type::Array(b)) => Rc::ptr_eq(a, b),
            (Type::Procedure(a), Type::Procedure(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

// The printed shape, which two different types can share. A diagnostic that
// compares two types says so itself when their shapes print the same.
impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Integer => write!(f, "INTEGER"),
            Type::Real => write!(f, "REAL"),
            Type::Boolean => write!(f, "BOOLEAN"),
            Type::Set => write!(f, "SET"),
            Type::Array(array) => write!(f, "ARRAY {} OF {}", array.len, array.elem),
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
