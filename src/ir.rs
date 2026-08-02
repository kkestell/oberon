#[derive(Debug)]
pub struct Program {
    pub module: String,
    pub globals: Vec<Global>,
    pub procs: Vec<Proc>,
}

#[derive(Debug)]
pub struct Global {
    pub symbol: String,
    pub ty: Ty,
}

#[derive(Debug)]
pub struct Proc {
    pub symbol: String,
    pub params: Vec<Param>,
    pub ret: Option<Ty>,
    pub slots: Vec<(String, Ty)>,
    pub insts: Vec<Inst>,
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

#[derive(Debug, Clone, Copy)]
pub enum Ty {
    Int,
    Bool,
}

#[derive(Debug, Clone)]
pub enum Value {
    Int(i32),
    Bool(bool),
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
        src: Value,
    },
    Un {
        dst: usize,
        op: UnOp,
        arg: Value,
    },
    Bin {
        dst: usize,
        op: BinOp,
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
}
