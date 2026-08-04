use std::cell::RefCell;
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
    // Report 6.1: INTEGER is compatible with BYTE. The store checks that the
    // value lies in 0 through 255, because that is BYTE's whole value set and
    // target truncation does not get to define the language.
    ByteRange,
    // Report 9.1: a single-character string stands for its CHAR.
    CharFromString,
    // Report 9.1: a string copies into a character array with a null
    // appended. The length rule is checked at the assignment, the one site
    // that can reach this.
    StringCopy,
    // The same string rule with a dynamic open destination capacity. The
    // statement emits the runtime fit check before its fixed-size copy.
    OpenStringCopy,
    // Report 9.1: two identical structured types copy the whole
    // representation, padding included. For records the Report asks for the
    // source to be an extension of the destination, which reduces to identity
    // until extension exists.
    WholeCopy,
    // Report 9.1's open-array exception. The statement checks a dynamic
    // source length against a fixed destination before copying that prefix.
    OpenPrefixCopy,
}

pub(super) fn open_actual_compatible(formal: &Type, actual: &Type) -> bool {
    match formal {
        Type::OpenArray(formal_elem) => actual
            .array_elem()
            .is_some_and(|actual_elem| open_actual_compatible(formal_elem, actual_elem)),
        _ => formal == actual,
    }
}

pub(super) fn open_string_formal(ty: &Type) -> bool {
    matches!(ty, Type::OpenArray(elem) if **elem == Type::Char)
}

pub(super) fn assign_kind(target: &Type, found: &Type) -> Option<AssignKind> {
    if target == found {
        if matches!(target, Type::OpenArray(_)) {
            return None;
        }
        return Some(if target.structured() {
            AssignKind::WholeCopy
        } else {
            AssignKind::Store
        });
    }
    match (target, found) {
        (Type::Pointer(_), _) if pointer_value_compatible(target, found) => Some(AssignKind::Store),
        (Type::Byte, Type::Integer) => Some(AssignKind::ByteRange),
        (Type::Char, Type::String(1)) => Some(AssignKind::CharFromString),
        (Type::Array(_), Type::String(_)) if target.char_array() => Some(AssignKind::StringCopy),
        (Type::OpenArray(_), Type::String(_)) if target.char_array() => {
            Some(AssignKind::OpenStringCopy)
        }
        (Type::Array(target), Type::OpenArray(source)) if target.elem == **source => {
            Some(AssignKind::OpenPrefixCopy)
        }
        _ => None,
    }
}

pub(super) fn pointer_value_compatible(lhs: &Type, rhs: &Type) -> bool {
    match (lhs, rhs) {
        (Type::Nil, Type::Nil) | (Type::Pointer(_), Type::Nil) | (Type::Nil, Type::Pointer(_)) => {
            true
        }
        (Type::Pointer(a), Type::Pointer(b)) => match (a.record(), b.record()) {
            (Some(a), Some(b)) => Rc::ptr_eq(&a, &b),
            _ => false,
        },
        _ => false,
    }
}

// The text pairs Report 8.2.4 lets a relation compare. Character arrays reach
// this only while a required constant expression is being type-checked; the
// later constant evaluation still rejects their variables as nonconstant.
pub(super) fn text_relation_ok(lhs: &Type, rhs: &Type) -> bool {
    if matches!(
        (lhs, rhs),
        (Type::Char, Type::Char)
            | (Type::Char, Type::String(1))
            | (Type::String(1), Type::Char)
            | (Type::String(_), Type::String(_))
    ) {
        return true;
    }
    let lhs_text = lhs.char_array() || matches!(lhs, Type::String(_));
    let rhs_text = rhs.char_array() || matches!(rhs, Type::String(_));
    lhs_text && rhs_text && (lhs.char_array() || rhs.char_array())
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

// Report 8.2.4 orders the numeric types and CHAR; SET and BOOLEAN have
// equality and inequality only. The operand type is returned rather than
// BOOLEAN, because the comparison instruction is chosen from it.
pub(super) fn check_order_types(
    pos: Pos,
    op: ast::BinOp,
    lhs: Type,
    rhs: Type,
    diags: &mut Vec<Diagnostic>,
) -> Option<Type> {
    check_operand_types(
        pos,
        op,
        &[Type::Integer, Type::Real, Type::Char],
        lhs,
        rhs,
        diags,
    )
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
        Type::OpenArray(_) => panic!("an open array needs incoming lengths"),
        _ => Vec::new(),
    }
}

pub(super) fn open_base(mut ty: &Type) -> &Type {
    while let Type::OpenArray(elem) = ty {
        ty = elem;
    }
    ty
}

pub(super) fn shape_with_open_lengths(ty: &Type, open_lengths: &[ir::Value]) -> Vec<ir::Value> {
    fn walk(ty: &Type, lengths: &[ir::Value], next: &mut usize, shape: &mut Vec<ir::Value>) {
        match ty {
            Type::OpenArray(elem) => {
                shape.push(lengths[*next].clone());
                *next += 1;
                walk(elem, lengths, next, shape);
            }
            Type::Array(array) => {
                shape.push(ir::Value::Int(array.len));
                walk(&array.elem, lengths, next, shape);
            }
            _ => {}
        }
    }

    let mut next = 0;
    let mut shape = Vec::new();
    walk(ty, open_lengths, &mut next, &mut shape);
    assert_eq!(
        next,
        open_lengths.len(),
        "every open length belongs to a dimension"
    );
    shape
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
    // Report 6.1. A CHAR is one unsigned byte, so the character set is exactly
    // the ordinals 0 through 255 and a string literal's bytes are its
    // characters. BYTE has the same storage but is an integer type: reading
    // one yields an INTEGER and writing one checks the range.
    Char,
    Byte,
    // The type of a string constant, carrying its character count. No source
    // identifier names it, so no variable, formal, or element can have it: a
    // string exists only as a literal or a constant declared from one.
    String(usize),
    // Report 6.2. Each ARRAY constructor in the source builds one descriptor,
    // and sharing that descriptor is what makes two types the same type. A
    // declaration's names, an alias, an interface member, and a client's view
    // of an exported type all hold the same handle.
    Array(Rc<ArrayType>),
    // Report 10.1. An open array exists only as a formal type. It owns no
    // layout; a bound for this dimension arrives beside the data address.
    OpenArray(Box<Type>),
    // Report 6.3, under the same identity rule as Array: one RECORD
    // constructor in the source is one type.
    Record(Rc<RecordType>),
    Pointer(Rc<PointerType>),
    // NIL is a polymorphic constant, not a source type. It has a pointer-class
    // value so it can be passed, returned, assigned, and compared where a
    // pointer context accepts it, but it has no storage layout of its own.
    Nil,
}

#[derive(Debug)]
pub struct ArrayType {
    pub(super) len: i32,
    pub(super) elem: Type,
    // Checked against ir::MAX_OBJECT_SIZE when the descriptor was built, so
    // every later layout sum can stay ordinary i64 arithmetic.
    pub(super) size: i64,
}

pub struct RecordType {
    // In declaration order, each with the offset the layout rule gave it.
    pub(super) fields: Vec<Field>,
    // Both computed once, when the constructor was resolved, and checked
    // against ir::MAX_OBJECT_SIZE there. Nothing recomputes layout later.
    pub(super) size: i64,
    pub(super) align: i64,
    // The name of the type declaration whose right side this constructor was.
    // A record's field list is too large to print in a diagnostic, so the
    // declared name is what diagnostics show; an inline constructor has none
    // and prints as RECORD.
    pub(super) name: Option<String>,
    // The module that declared the constructor. Report 6.3 makes an unmarked
    // field private to it, and the descriptor carries that home wherever it
    // travels, so an imported type, a re-exported alias, and an exported
    // variable of a private type all answer the same way.
    pub(super) module: String,
    pub(super) contains_pointers: bool,
}

impl fmt::Debug for RecordType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RecordType")
            .field("name", &self.name)
            .field("size", &self.size)
            .field("align", &self.align)
            .field("field_count", &self.fields.len())
            .field("contains_pointers", &self.contains_pointers)
            .finish()
    }
}

pub struct PointerType {
    pub(super) name: Option<String>,
    pub(super) base: RefCell<PointerBase>,
}

#[derive(Clone)]
pub(super) enum PointerBase {
    Pending { name: String, pos: Pos },
    Resolved(Rc<RecordType>),
    Invalid,
}

impl fmt::Debug for PointerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let base = match &*self.base.borrow() {
            PointerBase::Pending { name, .. } => format!("pending {name}"),
            PointerBase::Resolved(record) => record.name.clone().unwrap_or_else(|| "RECORD".into()),
            PointerBase::Invalid => "invalid".into(),
        };
        f.debug_struct("PointerType")
            .field("name", &self.name)
            .field("base", &base)
            .finish()
    }
}

impl PointerType {
    pub(super) fn record(&self) -> Option<Rc<RecordType>> {
        match &*self.base.borrow() {
            PointerBase::Resolved(record) => Some(record.clone()),
            PointerBase::Pending { .. } | PointerBase::Invalid => None,
        }
    }
}

#[derive(Debug)]
pub(super) struct Field {
    pub(super) name: String,
    pub(super) ty: Type,
    pub(super) offset: i64,
    pub(super) export: bool,
}

impl Type {
    // The IR type of a value of this type, when a value of it exists. An
    // array or a record has none: it is storage, and asking for one is how a
    // load, a store, an argument, or a result finds out it may not have this
    // type at all. A string has none either, so no string can reach a load, a
    // store, an argument, or a result by accident.
    pub(super) fn scalar(&self) -> Option<ir::Ty> {
        match self {
            Type::Integer => Some(ir::Ty::Int),
            Type::Real => Some(ir::Ty::Real),
            Type::Boolean => Some(ir::Ty::Bool),
            Type::Set => Some(ir::Ty::Set),
            Type::Char | Type::Byte => Some(ir::Ty::Byte),
            Type::Pointer(_) | Type::Nil => Some(ir::Ty::Pointer),
            Type::String(_) | Type::Array(_) | Type::OpenArray(_) | Type::Record(_) => None,
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
            Type::OpenArray(elem) => Some(elem),
            _ => None,
        }
    }

    pub(super) fn is_array(&self) -> bool {
        self.array_elem().is_some()
    }

    pub(super) fn open_rank(&self) -> usize {
        match self {
            Type::OpenArray(elem) => 1 + elem.open_rank(),
            _ => 0,
        }
    }

    pub(super) fn record(&self) -> Option<&Rc<RecordType>> {
        match self {
            Type::Record(record) => Some(record),
            _ => None,
        }
    }

    pub(super) fn pointer(&self) -> Option<&Rc<PointerType>> {
        match self {
            Type::Pointer(pointer) => Some(pointer),
            _ => None,
        }
    }

    // Report 9.1 and 10.1 say "structured (of array or record type)". These
    // are the types that live in storage and travel by address: a parameter of
    // one is a reference, and an assignment between two of them is a copy.
    pub(super) fn structured(&self) -> bool {
        matches!(self, Type::Array(_) | Type::OpenArray(_) | Type::Record(_))
    }

    // A one-dimensional fixed or open array whose element type is CHAR.
    // Report 9.1's string assignment and 8.2.4's array relations apply to
    // exactly these; the place supplies its fixed or dynamic bound.
    pub(super) fn char_array(&self) -> bool {
        matches!(self.array_elem(), Some(Type::Char))
    }

    pub(super) fn size(&self) -> i64 {
        match self {
            Type::Array(array) => array.size,
            Type::OpenArray(_) => panic!("an open array has no storage size"),
            Type::Record(record) => record.size,
            Type::Nil => panic!("NIL has no storage size"),
            scalar => ir::scalar_size(scalar.ir()),
        }
    }

    // An array is contiguous and takes its element's alignment, so the rule
    // works for the one-byte CHAR and BYTE types without a special case. A
    // record's alignment is the largest among its fields, computed when the
    // constructor was resolved.
    pub(super) fn align(&self) -> i64 {
        match self {
            Type::Array(array) => array.elem.align(),
            Type::OpenArray(_) => panic!("an open array has no storage alignment"),
            Type::Record(record) => record.align,
            Type::Nil => panic!("NIL has no storage alignment"),
            scalar => ir::scalar_size(scalar.ir()),
        }
    }

    pub(super) fn storage(&self) -> ir::Storage {
        match self {
            Type::Array(array) => ir::Storage::Array {
                len: array.len,
                elem: Box::new(array.elem.storage()),
            },
            Type::OpenArray(_) => panic!("an open array has no storage"),
            Type::Record(record) => ir::Storage::Record {
                size: record.size,
                align: record.align,
            },
            Type::Nil => panic!("NIL cannot be stored"),
            scalar => ir::Storage::Scalar(scalar.ir()),
        }
    }

    pub(super) fn contains_pointers(&self) -> bool {
        match self {
            Type::Pointer(_) => true,
            Type::Array(array) => array.len > 0 && array.elem.contains_pointers(),
            Type::OpenArray(_) => panic!("an open array owns no pointer-containing storage"),
            Type::Record(record) => record.contains_pointers,
            _ => false,
        }
    }
}

// Report 6.3: the scope of a field identifier is the record itself, and a
// field that is to be visible outside the declaring module must be marked.
// Inside the declaring module every field is visible, marked or not; outside
// it, only marked fields exist, so an unmarked one is indistinguishable from
// a field that was never declared. All three reference compilers behave this
// way — Project Oberon and OBNC both omit private fields from what a client
// can see — and it keeps private names out of other modules' diagnostics.
pub(super) fn find_field<'a>(
    record: &'a RecordType,
    name: &str,
    module: &str,
) -> Option<&'a Field> {
    record
        .fields
        .iter()
        .find(|field| field.name == name && (field.export || record.module == module))
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
            | (Type::Set, Type::Set)
            | (Type::Char, Type::Char)
            | (Type::Byte, Type::Byte) => true,
            (Type::String(a), Type::String(b)) => a == b,
            (Type::Array(a), Type::Array(b)) => Rc::ptr_eq(a, b),
            (Type::OpenArray(a), Type::OpenArray(b)) => a == b,
            (Type::Record(a), Type::Record(b)) => Rc::ptr_eq(a, b),
            (Type::Pointer(a), Type::Pointer(b)) => Rc::ptr_eq(a, b),
            (Type::Nil, Type::Nil) => true,
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
            Type::Char => write!(f, "CHAR"),
            Type::Byte => write!(f, "BYTE"),
            Type::String(1) => write!(f, "string of 1 character"),
            Type::String(n) => write!(f, "string of {n} characters"),
            Type::Array(array) => write!(f, "ARRAY {} OF {}", array.len, array.elem),
            Type::OpenArray(elem) => write!(f, "ARRAY OF {elem}"),
            // A record's field list would swamp the message it appears in, so
            // the declared name stands for it. A constructor written inline in
            // a variable declaration or a field list never had one.
            Type::Record(record) => match &record.name {
                Some(name) => write!(f, "{name}"),
                None => write!(f, "RECORD"),
            },
            Type::Pointer(pointer) => match &pointer.name {
                Some(name) => write!(f, "{name}"),
                None => match pointer.record() {
                    Some(record) => match &record.name {
                        Some(name) => write!(f, "POINTER TO {name}"),
                        None => write!(f, "POINTER TO RECORD"),
                    },
                    None => write!(f, "POINTER"),
                },
            },
            Type::Nil => write!(f, "NIL"),
        }
    }
}
