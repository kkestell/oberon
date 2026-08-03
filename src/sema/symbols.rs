use std::collections::HashMap;
use std::rc::Rc;

use crate::ir;

use super::types::{ProcedureParam, ProcedureType, Type};

pub(super) type Scope = HashMap<String, Symbol>;

#[derive(Debug, Clone, PartialEq)]
pub enum ConstValue {
    Int(i32),
    Bool(bool),
}

impl ConstValue {
    pub(super) fn ty(&self) -> Type {
        match self {
            ConstValue::Int(_) => Type::Integer,
            ConstValue::Bool(_) => Type::Boolean,
        }
    }

    pub(super) fn ir(&self) -> ir::Value {
        match self {
            ConstValue::Int(v) => ir::Value::Int(*v),
            ConstValue::Bool(v) => ir::Value::Bool(*v),
        }
    }
}

#[derive(Debug, Clone)]
pub(super) enum Symbol {
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
        ty: Type,
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
// This is a build result held in memory, not a symbol file. Cloning it clones
// shared type handles rather than rebuilding types, so an exported type keeps
// one identity across every client, however many of them there are and however
// many times it is re-exported.
#[derive(Debug, Clone, Default)]
pub struct Interface {
    pub members: HashMap<String, Member>,
}

#[derive(Debug, Clone)]
pub enum Member {
    Const(ConstValue),
    Type(Type),
    Var { ty: Type, symbol: String },
    Proc { symbol: String, ty: Type },
}

impl Member {
    // Report 9.1 and 11: an imported variable is read-only in the client. The
    // declaring module keeps the writable symbol it built for itself.
    pub(super) fn client_symbol(&self) -> Symbol {
        match self {
            Member::Const(value) => Symbol::Const(value.clone()),
            Member::Type(ty) => Symbol::TypeName(ty.clone()),
            Member::Var { ty, symbol } => Symbol::Var {
                ty: ty.clone(),
                addr: ir::Addr::Global(symbol.clone()),
                read_only: true,
            },
            Member::Proc { symbol, ty } => Symbol::Proc {
                symbol: symbol.clone(),
                ty: ty.clone(),
            },
        }
    }
}

// Report 10.2. The predefined operations available to the implemented source
// types.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Builtin {
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

pub(super) fn universe_scope() -> Scope {
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

// The native Out: an interface with no Oberon source behind it, whose
// procedures are the C runtime's. The driver installs it only when no source
// module of that name is found.
pub fn out_interface() -> Interface {
    let mut members = HashMap::new();
    let value = |ty| (false, ty);
    let mut add = |name: &str, symbol: &str, params: Vec<(bool, Type)>| {
        members.insert(
            name.into(),
            Member::Proc {
                symbol: symbol.into(),
                ty: procedure_type(params, None),
            },
        );
    };

    add(
        "Int",
        "oberon_out_int",
        vec![value(Type::Integer), value(Type::Integer)],
    );
    add("Ln", "oberon_out_ln", Vec::new());

    Interface { members }
}

fn procedure_type(params: Vec<(bool, Type)>, ret: Option<Type>) -> Type {
    Type::Procedure(Rc::new(ProcedureType {
        params: params
            .into_iter()
            .map(|(var, ty)| ProcedureParam { var, ty })
            .collect(),
        ret,
    }))
}

// Report 10.2 gives ABS a result of whichever type it was given. Nothing else
// here is generic in its result, so the two cases are named directly instead
// of through a signature framework.
#[derive(Debug, Clone)]
pub(super) enum BuiltinResult {
    Fixed(Type),
    Argument,
}

impl BuiltinResult {
    pub(super) fn ty(&self, first_arg: Type) -> Type {
        match self {
            BuiltinResult::Fixed(ty) => ty.clone(),
            BuiltinResult::Argument => first_arg,
        }
    }
}

// Argument types and result type of the function-like predefined operations.
// Each parameter carries the types it accepts. INC, DEC, and ASSERT are proper
// procedures and have no entry: a None here is what makes them "cannot be
// used as a value" in an expression.
pub(super) fn builtin_signature(
    builtin: Builtin,
) -> Option<(&'static [&'static [Type]], BuiltinResult)> {
    use BuiltinResult::{Argument, Fixed};
    match builtin {
        Builtin::Abs => Some((&[&[Type::Integer]], Argument)),
        Builtin::Odd => Some((&[&[Type::Integer]], Fixed(Type::Boolean))),
        Builtin::Lsl | Builtin::Asr | Builtin::Ror => {
            Some((&[&[Type::Integer], &[Type::Integer]], Fixed(Type::Integer)))
        }
        Builtin::Ord => Some((&[&[Type::Boolean]], Fixed(Type::Integer))),
        Builtin::Inc | Builtin::Dec | Builtin::Assert => None,
    }
}

pub(super) fn type_list(types: &[Type]) -> String {
    types
        .iter()
        .map(Type::to_string)
        .collect::<Vec<_>>()
        .join(" or ")
}
