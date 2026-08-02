use std::collections::HashMap;
use std::rc::Rc;

use crate::ir;

use super::types::{ProcedureParam, ProcedureType, Type};

pub(super) type Scope = HashMap<String, Symbol>;

#[derive(Debug, Clone, PartialEq)]
pub enum ConstValue {
    Int(i32),
}

impl ConstValue {
    pub(super) fn ty(&self) -> Type {
        match self {
            ConstValue::Int(_) => Type::Integer,
        }
    }

    pub(super) fn ir(&self) -> ir::Value {
        match self {
            ConstValue::Int(v) => ir::Value::Int(*v),
        }
    }
}

#[derive(Debug, Clone)]
pub(super) enum Symbol {
    Const(ConstValue),
    Var { ty: Type, addr: ir::Addr },
    Proc { symbol: String, ty: Type },
    Module(HashMap<String, Symbol>),
    TypeName(Type),
}

// The public face of a module. The interface owns its data and is cloned into
// each client.
#[derive(Debug, Clone, Default)]
pub struct Interface {
    pub members: HashMap<String, Member>,
}

#[derive(Debug, Clone)]
pub enum Member {
    Proc { symbol: String, ty: Type },
}

impl Member {
    pub(super) fn client_symbol(&self) -> Symbol {
        match self {
            Member::Proc { symbol, ty } => Symbol::Proc {
                symbol: symbol.clone(),
                ty: ty.clone(),
            },
        }
    }
}

pub(super) fn universe_scope() -> Scope {
    let mut scope = Scope::new();
    scope.insert("INTEGER".into(), Symbol::TypeName(Type::Integer));
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
