use std::collections::HashMap;
use std::rc::Rc;

use crate::ir;

use super::types::{ProcedureParam, ProcedureType, Type, fixed_shape};

#[derive(Debug, Clone)]
pub(super) enum RecordDynamic {
    Static(String),
    Incoming(ir::Value),
    Heap(ir::Value),
}

pub(super) type Scope = HashMap<String, Symbol>;

// The string case ends the enum's days as a Copy type: the bytes are owned
// and shared through the Rc, so a clone is cheap and one buffer sits behind a
// constant however many modules import it.
#[derive(Debug, Clone, PartialEq)]
pub enum ConstValue {
    Int(i32),
    // Folded at binary32 precision at every source operator, so a constant
    // expression and the same expression computed at run time round the same
    // way at the same points.
    Real(f32),
    Bool(bool),
    // Unsigned so complement covers exactly the 32 supported elements and no
    // set operation can overflow a signed INTEGER.
    Set(u32),
    // The ordinal of one character. Only ORD and CHR fold to this; a
    // single-character string stays a string, because a named constant must
    // behave exactly like the literal it was declared from.
    Char(u8),
    Str(Rc<Vec<u8>>),
    Nil,
}

impl ConstValue {
    pub(super) fn ty(&self) -> Type {
        match self {
            ConstValue::Int(_) => Type::Integer,
            ConstValue::Real(_) => Type::Real,
            ConstValue::Bool(_) => Type::Boolean,
            ConstValue::Set(_) => Type::Set,
            ConstValue::Char(_) => Type::Char,
            ConstValue::Str(bytes) => Type::String(bytes.len()),
            ConstValue::Nil => Type::Nil,
        }
    }

    pub(super) fn ir(&self) -> ir::Value {
        match self {
            ConstValue::Int(v) => ir::Value::Int(*v),
            ConstValue::Real(v) => ir::Value::Real(*v),
            ConstValue::Bool(v) => ir::Value::Bool(*v),
            ConstValue::Set(v) => ir::Value::Set(*v),
            ConstValue::Char(v) => ir::Value::Byte(*v),
            ConstValue::Str(_) => panic!("a string constant has no scalar value"),
            ConstValue::Nil => ir::Value::Pointer(0),
        }
    }
}

#[derive(Debug, Clone)]
pub(super) enum Symbol {
    Const(ConstValue),
    Var {
        ty: Type,
        addr: ir::Addr,
        shape: Vec<ir::Value>,
        // Set on an imported variable, and while a FOR statement's body is
        // being lowered so the body cannot move its own control variable.
        read_only: bool,
        dynamic: Option<RecordDynamic>,
    },
    Proc {
        symbol: String,
        ty: Type,
        eligible: bool,
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
            // A string constant's clone shares the bytes behind the Rc, so
            // every client sees the one buffer.
            Member::Const(value) => Symbol::Const(value.clone()),
            Member::Type(ty) => Symbol::TypeName(ty.clone()),
            Member::Var { ty, symbol } => Symbol::Var {
                ty: ty.clone(),
                addr: ir::Addr::Global(symbol.clone()),
                shape: fixed_shape(ty),
                read_only: true,
                dynamic: ty
                    .record()
                    .map(|record| RecordDynamic::Static(record.descriptor.clone())),
            },
            Member::Proc { symbol, ty } => Symbol::Proc {
                symbol: symbol.clone(),
                ty: ty.clone(),
                eligible: true,
            },
        }
    }
}

// Report 10.2. The predefined operations available to the implemented source
// types, including pointer allocation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Builtin {
    Abs,
    Len,
    Odd,
    Floor,
    Flt,
    Lsl,
    Asr,
    Ror,
    Ord,
    Chr,
    Inc,
    Dec,
    Incl,
    Excl,
    Pack,
    Unpk,
    Assert,
    New,
}

pub(super) fn universe_scope() -> Scope {
    let mut scope = Scope::new();
    scope.insert("INTEGER".into(), Symbol::TypeName(Type::Integer));
    scope.insert("REAL".into(), Symbol::TypeName(Type::Real));
    scope.insert("BOOLEAN".into(), Symbol::TypeName(Type::Boolean));
    scope.insert("SET".into(), Symbol::TypeName(Type::Set));
    scope.insert("CHAR".into(), Symbol::TypeName(Type::Char));
    scope.insert("BYTE".into(), Symbol::TypeName(Type::Byte));
    for (name, builtin) in [
        ("ABS", Builtin::Abs),
        ("LEN", Builtin::Len),
        ("ODD", Builtin::Odd),
        ("FLOOR", Builtin::Floor),
        ("FLT", Builtin::Flt),
        ("LSL", Builtin::Lsl),
        ("ASR", Builtin::Asr),
        ("ROR", Builtin::Ror),
        ("ORD", Builtin::Ord),
        ("CHR", Builtin::Chr),
        ("INC", Builtin::Inc),
        ("DEC", Builtin::Dec),
        ("INCL", Builtin::Incl),
        ("EXCL", Builtin::Excl),
        ("PACK", Builtin::Pack),
        ("UNPK", Builtin::Unpk),
        ("ASSERT", Builtin::Assert),
        ("NEW", Builtin::New),
    ] {
        scope.insert(name.into(), Symbol::Builtin(builtin));
    }
    scope
}

// Typed native operations available only to bundled library source. The
// driver origin-gates this interface and never caches it as a source module.
pub fn runtime_interface() -> Interface {
    let mut members = HashMap::new();
    let chars = || Type::OpenArray(Box::new(Type::Char));
    let bytes = || Type::OpenArray(Box::new(Type::Byte));
    let value = |ty| (false, ty);
    let var = |ty| (true, ty);
    let mut add = |name: &str, params: Vec<(bool, Type)>, ret: Option<Type>| {
        members.insert(
            name.into(),
            Member::Proc {
                symbol: native_symbol(name),
                ty: procedure_type(params, ret),
            },
        );
    };

    add("OutChar", vec![value(Type::Char)], None);
    add("OutString", vec![value(chars())], None);
    add(
        "OutInt",
        vec![value(Type::Integer), value(Type::Integer)],
        None,
    );
    add("OutHex", vec![value(Type::Integer)], None);
    add(
        "OutReal",
        vec![value(Type::Real), value(Type::Integer)],
        None,
    );
    add("OutLn", Vec::new(), None);

    add("InOpen", Vec::new(), Some(Type::Boolean));
    add("InChar", vec![var(Type::Char)], Some(Type::Boolean));
    add("InInt", vec![var(Type::Integer)], Some(Type::Boolean));
    add("InReal", vec![var(Type::Real)], Some(Type::Boolean));
    for name in ["InString", "InName", "InLine"] {
        add(name, vec![var(chars())], Some(Type::Boolean));
    }

    for name in [
        "MathSqrt",
        "MathExp",
        "MathLn",
        "MathRound",
        "MathSin",
        "MathCos",
        "MathTan",
        "MathArcsin",
        "MathArccos",
        "MathArctan",
        "MathSinh",
        "MathCosh",
        "MathTanh",
        "MathArcsinh",
        "MathArccosh",
        "MathArctanh",
    ] {
        add(name, vec![value(Type::Real)], Some(Type::Real));
    }
    for name in ["MathPower", "MathLog", "MathArctan2"] {
        add(
            name,
            vec![value(Type::Real), value(Type::Real)],
            Some(Type::Real),
        );
    }

    add("FileOld", vec![value(chars())], Some(Type::Integer));
    add("FileNew", vec![value(chars())], Some(Type::Integer));
    for name in ["FileRelease", "FileRegister", "FileClose", "FilePurge"] {
        add(name, vec![value(Type::Integer)], None);
    }
    add("FileDelete", vec![value(chars())], Some(Type::Integer));
    add(
        "FileRename",
        vec![value(chars()), value(chars())],
        Some(Type::Integer),
    );
    add(
        "FileLength",
        vec![value(Type::Integer)],
        Some(Type::Integer),
    );
    add(
        "FileDate",
        vec![value(Type::Integer), var(Type::Integer), var(Type::Integer)],
        Some(Type::Boolean),
    );

    for (name, ty) in [
        ("FileReadByte", Type::Byte),
        ("FileReadInt", Type::Integer),
        ("FileReadReal", Type::Real),
        ("FileReadNum", Type::Integer),
        ("FileReadSet", Type::Set),
        ("FileReadBool", Type::Boolean),
    ] {
        add(
            name,
            vec![
                value(Type::Integer),
                value(Type::Integer),
                var(ty),
                var(Type::Integer),
            ],
            Some(Type::Integer),
        );
    }
    add(
        "FileReadString",
        vec![
            value(Type::Integer),
            value(Type::Integer),
            var(chars()),
            var(Type::Integer),
        ],
        Some(Type::Integer),
    );
    add(
        "FileReadBytes",
        vec![
            value(Type::Integer),
            value(Type::Integer),
            var(bytes()),
            value(Type::Integer),
            var(Type::Integer),
        ],
        Some(Type::Integer),
    );

    for (name, ty) in [
        ("FileWriteByte", Type::Byte),
        ("FileWriteInt", Type::Integer),
        ("FileWriteReal", Type::Real),
        ("FileWriteNum", Type::Integer),
        ("FileWriteSet", Type::Set),
        ("FileWriteBool", Type::Boolean),
    ] {
        add(
            name,
            vec![
                value(Type::Integer),
                value(Type::Integer),
                value(ty),
                var(Type::Integer),
            ],
            Some(Type::Integer),
        );
    }
    add(
        "FileWriteString",
        vec![
            value(Type::Integer),
            value(Type::Integer),
            value(chars()),
            var(Type::Integer),
        ],
        Some(Type::Integer),
    );
    add(
        "FileWriteBytes",
        vec![
            value(Type::Integer),
            value(Type::Integer),
            var(bytes()),
            value(Type::Integer),
            var(Type::Integer),
        ],
        Some(Type::Integer),
    );
    add("FileError", vec![var(chars())], None);

    add("ProgramArgCount", Vec::new(), Some(Type::Integer));
    add(
        "ProgramArg",
        vec![value(Type::Integer), var(chars())],
        Some(Type::Integer),
    );
    add("ProgramExit", vec![value(Type::Integer)], None);

    Interface { members }
}

fn native_symbol(name: &str) -> String {
    let mut symbol = String::from("oberon_lib_");
    for (i, ch) in name.chars().enumerate() {
        if ch.is_ascii_uppercase() && i != 0 {
            symbol.push('_');
        }
        symbol.push(ch.to_ascii_lowercase());
    }
    symbol
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

// Report 10.2 gives ABS one INTEGER form and one REAL form, and its result is
// whichever type it was given. Nothing else here is generic in its result, so
// the two cases are named directly instead of through a signature framework.
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
// Each parameter carries the types it accepts, which is one type everywhere
// except ABS and ORD: Report 10.2 gives ORD a CHAR form, a BOOLEAN form, and
// a SET form. INC, DEC, INCL, EXCL, PACK, UNPK, and ASSERT are proper
// procedures and have no entry: a None here is what makes them "cannot be
// used as a value" in an expression.
pub(super) fn builtin_signature(
    builtin: Builtin,
) -> Option<(&'static [&'static [Type]], BuiltinResult)> {
    use BuiltinResult::{Argument, Fixed};
    match builtin {
        Builtin::Abs => Some((&[&[Type::Integer, Type::Real]], Argument)),
        Builtin::Odd => Some((&[&[Type::Integer]], Fixed(Type::Boolean))),
        Builtin::Floor => Some((&[&[Type::Real]], Fixed(Type::Integer))),
        Builtin::Flt => Some((&[&[Type::Integer]], Fixed(Type::Real))),
        Builtin::Lsl | Builtin::Asr | Builtin::Ror => {
            Some((&[&[Type::Integer], &[Type::Integer]], Fixed(Type::Integer)))
        }
        Builtin::Ord => Some((
            &[&[Type::Char, Type::Boolean, Type::Set]],
            Fixed(Type::Integer),
        )),
        Builtin::Chr => Some((&[&[Type::Integer]], Fixed(Type::Char))),
        // LEN takes a designator rather than a value, so it has no entry in
        // the value signature table either; lower_len and check_const_len
        // handle it.
        Builtin::Len
        | Builtin::Inc
        | Builtin::Dec
        | Builtin::Incl
        | Builtin::Excl
        | Builtin::Pack
        | Builtin::Unpk
        | Builtin::Assert
        | Builtin::New => None,
    }
}

pub(super) fn type_list(types: &[Type]) -> String {
    types
        .iter()
        .map(Type::to_string)
        .collect::<Vec<_>>()
        .join(" or ")
}
