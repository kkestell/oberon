use crate::diag::Pos;

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
    // The string literals whose bytes have to exist at run time: one data
    // object each, numbered within the module and never shared between two
    // occurrences of the same text. The module's own name is one of them, so
    // a failing runtime check can report it.
    pub literals: Vec<Literal>,
    pub procs: Vec<Proc>,
}

// The characters of one string literal. The emitted data object appends the
// null terminator, so `bytes` is exactly what the source wrote and the object
// is one byte longer.
#[derive(Debug)]
pub struct Literal {
    pub symbol: String,
    pub bytes: Vec<u8>,
}

// Where a runtime check came from: the data object holding the module name
// and the 1-based position in that module's source. The runtime prints both
// when the check fails.
#[derive(Debug, Clone)]
pub struct Site {
    pub module: String,
    pub pos: Pos,
}

impl Site {
    // The three trailing operands every runtime check receives, in the order
    // its C parameters declare them.
    pub fn args(&self) -> [Arg; 3] {
        [
            Arg::Val(Ty::Pointer, Value::Symbol(self.module.clone())),
            Arg::Val(Ty::Int, Value::Int(position(self.pos.line))),
            Arg::Val(Ty::Int, Value::Int(position(self.pos.col))),
        ]
    }
}

fn position(n: u32) -> i32 {
    i32::try_from(n).expect("a source position fits an INTEGER")
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

// What a global or a slot reserves. Values still travel as a scalar `Ty`; this
// is the shape of the storage they live in.
#[derive(Debug, Clone)]
pub enum Storage {
    Scalar(Ty),
}

impl Storage {
    pub fn size(&self) -> i64 {
        match self {
            Storage::Scalar(ty) => scalar_size(*ty),
        }
    }

    pub fn align(&self) -> i64 {
        match self {
            Storage::Scalar(ty) => scalar_size(*ty),
        }
    }
}

pub fn scalar_size(ty: Ty) -> i64 {
    match ty {
        Ty::Int | Ty::Bool | Ty::Set => 4,
        Ty::Pointer => 8,
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

// Int, Bool, and Set are four bytes and travel in a QBE word.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ty {
    Int,
    Bool,
    Set,
    Pointer,
}

// A SET is a bit vector, so its immediate is unsigned: bit 31 is an ordinary
// element and not a sign.
#[derive(Debug, Clone)]
pub enum Value {
    Int(i32),
    Bool(bool),
    Set(u32),
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
    // `ty` is the operand type, which is also the result type.
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
    // A call to a runtime routine that reports the failure and ends the
    // process, so it terminates its block.
    Trap {
        symbol: String,
        site: Site,
    },
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
