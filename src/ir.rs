// Every source module of one build, in the order their initializers must run:
// a module's dependencies precede it.
#[derive(Debug)]
pub struct Program {
    pub modules: Vec<Module>,
}

#[derive(Debug)]
pub struct Module {
    pub name: String,
    pub descriptors: Vec<Descriptor>,
    pub globals: Vec<Global>,
    // The string literals whose bytes have to exist at run time: one data
    // object each, numbered within the module and never shared between two
    // occurrences of the same text.
    pub literals: Vec<Literal>,
    pub procs: Vec<Proc>,
}

#[derive(Debug)]
pub struct Descriptor {
    pub symbol: String,
    pub base: Option<String>,
}

// The characters of one string literal. The emitted data object appends the
// null terminator, so `bytes` is exactly what the source wrote and the object
// is one byte longer.
#[derive(Debug)]
pub struct Literal {
    pub symbol: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug)]
pub struct Global {
    pub symbol: String,
    pub ty: Storage,
}

#[derive(Debug)]
pub struct Proc {
    pub symbol: String,
    pub params: Vec<Param>,
    pub ret: Option<Ty>,
    pub slots: Vec<(String, Storage)>,
    pub insts: Vec<Inst>,
}

// The largest object the compiler will lay out, in bytes. One GiB is well
// under QBE's signed stack-offset range and under the reach of the small code
// model's data references, so a program built from objects this size still
// leaves room for the code, the runtime's own objects, and linker placement.
// Semantic analysis checks each type and each procedure's and module's total
// against it, and the driver checks the whole program's static data.
pub const MAX_OBJECT_SIZE: i64 = 1 << 30;

// What a global or a slot reserves. Values still travel as a scalar `Ty`; this
// is the shape of the storage they live in, which for an array is not a value
// type at all. Sema has already checked every size against MAX_OBJECT_SIZE, so
// the arithmetic here cannot overflow.
#[derive(Debug, Clone)]
pub enum Storage {
    Scalar(Ty),
    Array { len: i32, elem: Box<Storage> },
    // Where a record's fields sit is already baked into the field instructions
    // the front end emitted, so reserving storage only needs the two numbers
    // sema computed when it laid the record out.
    Record { size: i64, align: i64 },
}

impl Storage {
    pub fn size(&self) -> i64 {
        match self {
            Storage::Scalar(ty) => scalar_size(*ty),
            Storage::Array { len, elem } => i64::from(*len)
                .checked_mul(elem.size())
                .expect("sema checked this layout"),
            Storage::Record { size, .. } => *size,
        }
    }

    // An array is contiguous and needs no padding of its own, so it is aligned
    // exactly as its elements are.
    pub fn align(&self) -> i64 {
        match self {
            Storage::Scalar(ty) => scalar_size(*ty),
            Storage::Array { elem, .. } => elem.align(),
            Storage::Record { align, .. } => *align,
        }
    }
}

pub fn scalar_size(ty: Ty) -> i64 {
    match ty {
        Ty::Int | Ty::Bool | Ty::Set | Ty::Real => 4,
        Ty::Byte => 1,
        Ty::Pointer | Ty::Procedure => 8,
    }
}

#[derive(Debug)]
pub struct Param {
    pub temp: usize,
    pub pass: ParamPass,
}

#[derive(Debug, Clone, Copy)]
pub enum ParamPass {
    Value(Ty),
    Ref,
}

// Int, Bool, and Set are four bytes and travel in a QBE word; Real is IEEE
// 754 binary32 and travels in a QBE single, which is a distinct calling class
// and a distinct set of arithmetic and comparison operations. Byte is one
// unsigned byte in storage — CHAR and BYTE share it, because they differ only
// in source rules — and still travels in a word: it loads zero-extended, so a
// register value is always 0 through 255 and the signed word comparisons give
// the correct unsigned ordering.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ty {
    Int,
    Bool,
    Set,
    Real,
    Byte,
    Pointer,
    // A code address has the target's pointer-sized machine class, but it is
    // not a source pointer and must never affect heap scanning decisions.
    Procedure,
}

// A SET is a bit vector, so its immediate is unsigned: bit 31 is an ordinary
// element and not a sign.
#[derive(Debug, Clone)]
pub enum Value {
    Int(i32),
    Bool(bool),
    Set(u32),
    Real(f32),
    Byte(u8),
    Pointer(u64),
    Symbol(String),
    Temp(usize),
}

#[derive(Debug, Clone)]
pub enum Addr {
    Global(String),
    Slot(String),
    Temp(usize),
}

#[derive(Debug)]
pub enum Inst {
    Label(String),
    Load {
        dst: usize,
        ty: Ty,
        addr: Addr,
    },
    Store {
        ty: Ty,
        val: Value,
        addr: Addr,
    },
    Copy {
        dst: usize,
        ty: Ty,
        src: Value,
    },
    // `ty` is the operand type, which is also the result type: negation of a
    // REAL is a single-precision operation and negation of an INTEGER is a
    // word one.
    Un {
        dst: usize,
        op: UnOp,
        ty: Ty,
        arg: Value,
    },
    // `ty` is the type the operation computes in, which is the type of the
    // operands and, for everything but a comparison, of the result too. A
    // comparison always yields BOOLEAN and picks its QBE instruction from the
    // operand type.
    Bin {
        dst: usize,
        op: BinOp,
        ty: Ty,
        lhs: Value,
        rhs: Value,
    },
    // Report 10.2 FLT: the only conversion between machine classes this
    // compiler has, so it needs no general conversion matrix.
    IntToReal {
        dst: usize,
        arg: Value,
    },
    CheckNil {
        pointer: Value,
    },
    CheckProcedure {
        procedure: Value,
    },
    Alloc {
        dst: usize,
        size: i64,
        scanned: bool,
        descriptor: String,
    },
    HeapDescriptor {
        dst: usize,
        pointer: Value,
    },
    TypeTestPointer {
        dst: usize,
        pointer: Value,
        target: String,
    },
    TypeTestDescriptor {
        dst: usize,
        descriptor: Value,
        target: String,
    },
    // The address of one element: `dst = base + index * stride`, after
    // checking `0 <= index < len`. The applicable length travels with the
    // instruction rather than being recovered from the base allocation,
    // because the base of an inner dimension is an address with no allocation
    // of its own, and because an open dimension's length exists only as an
    // incoming value that no allocation records.
    Index {
        dst: usize,
        base: Addr,
        index: Value,
        len: Value,
        // The byte stride is `stride` multiplied by each dynamic inner
        // length. Fixed inner dimensions are already folded into stride.
        stride: i64,
        dynamic_stride: Vec<Value>,
    },
    // The address of one field: `dst = base + offset`. The offset is the one
    // the record's layout assigned, so nothing downstream recomputes it, and
    // no check applies: a field selection cannot be out of range.
    Field {
        dst: usize,
        base: Addr,
        offset: i64,
    },
    // A whole-value copy of `size` bytes. The count is the source type's exact
    // size, zero included, and the copy tolerates the source and destination
    // being the same region.
    CopyBytes {
        dst: Addr,
        src: Addr,
        size: i64,
    },
    // Report 9.1's open-array assignment fits only when the source is no
    // longer than the destination. The check is its own instruction so it can
    // be emitted before the copy and fail without moving a byte.
    CheckArrayCopy {
        source_len: Value,
        destination_len: Value,
    },
    // Copy `count` elements whose complete immediate type occupies `stride`
    // bytes. The count is an INTEGER value and is widened before byte
    // multiplication in the backend.
    CopyElements {
        temp: usize,
        dst: Addr,
        src: Addr,
        count: Value,
        stride: i64,
    },
    Call {
        dst: Option<(usize, Ty)>,
        target: CallTarget,
        args: Vec<Arg>,
    },
    Jmp(String),
    Br {
        cond: Value,
        then: String,
        els: String,
    },
    Ret(Option<Value>),
    Halt,
}

#[derive(Debug)]
pub enum CallTarget {
    Direct(String),
    Indirect(Value),
}

#[derive(Debug)]
pub enum Arg {
    Val(Ty, Value),
    Ref(Addr),
}

#[derive(Debug, Clone, Copy)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Shl,
    Shr,
    Sar,
    BitAnd,
    BitOr,
    BitXor,
}

impl BinOp {
    // A comparison returns BOOLEAN whatever its operands are; every other
    // binary operation returns the operand type.
    pub fn is_comparison(self) -> bool {
        matches!(
            self,
            BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge
        )
    }
}
