// Every source module of one build, in the order their initializers must run:
// a module's dependencies precede it.
#[derive(Debug)]
pub struct Program {
    pub modules: Vec<Module>,
}

#[derive(Debug)]
pub struct Module {
    pub name: String,
    pub globals: Vec<Global>,
    pub procs: Vec<Proc>,
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
}

impl Storage {
    pub fn size(&self) -> i64 {
        match self {
            Storage::Scalar(ty) => scalar_size(*ty),
            Storage::Array { len, elem } => i64::from(*len)
                .checked_mul(elem.size())
                .expect("sema checked this layout"),
        }
    }

    // An array is contiguous and needs no padding of its own, so it is aligned
    // exactly as its elements are.
    pub fn align(&self) -> i64 {
        match self {
            Storage::Scalar(ty) => scalar_size(*ty),
            Storage::Array { elem, .. } => elem.align(),
        }
    }
}

pub fn scalar_size(ty: Ty) -> i64 {
    match ty {
        Ty::Int | Ty::Bool | Ty::Set | Ty::Real => 4,
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

// Every type is four bytes wide. Int, Bool, and Set travel in a QBE word;
// Real is IEEE 754 binary32 and travels in a QBE single, which is a distinct
// calling class and a distinct set of arithmetic and comparison operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ty {
    Int,
    Bool,
    Set,
    Real,
}

// A SET is a bit vector, so its immediate is unsigned: bit 31 is an ordinary
// element and not a sign.
#[derive(Debug, Clone)]
pub enum Value {
    Int(i32),
    Bool(bool),
    Set(u32),
    Real(f32),
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
    // The address of one element: `dst = base + index * stride`, after
    // checking `0 <= index < len`. The applicable length travels with the
    // instruction rather than being recovered from the base allocation,
    // because the base of an inner dimension is an address with no allocation
    // of its own, and because an open array will supply a dynamic length here
    // without changing the address rule.
    Index {
        dst: usize,
        base: Addr,
        index: Value,
        len: i32,
        stride: i64,
    },
    // A whole-value copy of `size` bytes. The count is the source type's exact
    // size, zero included, and the copy tolerates the source and destination
    // being the same region.
    CopyBytes {
        dst: Addr,
        src: Addr,
        size: i64,
    },
    Call {
        dst: Option<(usize, Ty)>,
        symbol: String,
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
