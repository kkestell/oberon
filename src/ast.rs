use crate::diag::Pos;

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub pos: Pos,
    pub imports: Vec<Import>,
    pub consts: Vec<ConstDecl>,
    pub vars: Vec<VarDecl>,
    pub procs: Vec<ProcDecl>,
    pub body: Vec<Stmt>,
}

// identdef = ident ["*"]. The grammar allows the mark on a local declaration
// too, so the parser records it and sema decides whether the scope permits it.
#[derive(Debug, Clone)]
pub struct IdentDef {
    pub name: String,
    pub pos: Pos,
    pub export: bool,
}

// `IMPORT X := M` declares the qualifier X and selects the module M. The two
// names get separate positions: a lookup failure belongs on M, a duplicate
// qualifier on X. An unaliased import repeats one name in both roles.
#[derive(Debug, Clone)]
pub struct Import {
    pub name: String,
    pub pos: Pos,
    pub qualifier: String,
    pub qualifier_pos: Pos,
}

#[derive(Debug, Clone)]
pub struct ConstDecl {
    pub id: IdentDef,
    pub expr: Expr,
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub names: Vec<IdentDef>,
    pub ty: Designator, // TODO: enum when StrucType lands
}

#[derive(Debug, Clone)]
pub struct ProcDecl {
    pub id: IdentDef,
    pub params: Vec<FpSection>,
    pub ret: Option<Designator>,
    pub consts: Vec<ConstDecl>,
    pub vars: Vec<VarDecl>,
    pub procs: Vec<ProcDecl>,
    pub body: Vec<Stmt>,
    pub ret_val: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct FpSection {
    pub var: bool,
    pub names: Vec<(String, Pos)>,
    pub ty: Designator,
}

// `Out.Int` parses as base "Out" + Field("Int"): the parser cannot tell module
// qualification from field selection, so sema decides.
#[derive(Debug, Clone)]
pub struct Designator {
    pub ident: String,
    pub selectors: Vec<Selector>,
    pub pos: Pos,
}

#[derive(Debug, Clone)]
pub enum Selector {
    Field(String, Pos), // TODO: Index, Deref, TypeGuard
}

impl Designator {
    // For diagnostics: "Out.Int", not just the base "Out".
    pub fn name(&self) -> String {
        let mut s = self.ident.clone();
        for sel in &self.selectors {
            match sel {
                Selector::Field(f, _) => {
                    s.push('.');
                    s.push_str(f);
                }
            }
        }
        s
    }
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Assign {
        lhs: Designator,
        rhs: Expr,
        #[allow(dead_code)]
        pos: Pos,
    },
    Call {
        proc: Designator,
        args: Vec<Expr>,
        pos: Pos,
    },
    If {
        cond: Expr,
        then: Vec<Stmt>,
        elsifs: Vec<(Expr, Vec<Stmt>)>,
        els: Option<Vec<Stmt>>,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
        elsifs: Vec<(Expr, Vec<Stmt>)>,
    },
    Repeat {
        body: Vec<Stmt>,
        cond: Expr,
    },
    For {
        // The grammar spells the control variable as a bare ident; it is a
        // Designator so sema can resolve it with the same code as everything
        // else. The selector list is always empty.
        var: Designator,
        start: Expr,
        limit: Expr,
        step: Option<Expr>,
        body: Vec<Stmt>,
    },
    Case {
        expr: Expr,
        arms: Vec<CaseArm>,
    },
}

#[derive(Debug, Clone)]
pub struct CaseArm {
    pub labels: Vec<LabelRange>,
    pub body: Vec<Stmt>,
}

// label = integer | string | qualident, so the parser puts only Int and Name
// expressions here. It is an Expr because sema folds it like any constant.
#[derive(Debug, Clone)]
pub struct LabelRange {
    pub low: Expr,
    pub high: Option<Expr>,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Int {
        value: i64,
        pos: Pos,
    },
    Bool {
        value: bool,
        pos: Pos,
    },
    Name(Designator),
    Call {
        callee: Designator,
        args: Vec<Expr>,
        pos: Pos,
    },
    Unary {
        op: UnOp,
        expr: Box<Expr>,
        pos: Pos,
    },
    Binary {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        pos: Pos,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnOp {
    Neg, // unary + is dropped in the parser
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

impl Expr {
    pub fn pos(&self) -> Pos {
        match self {
            Expr::Int { pos, .. }
            | Expr::Bool { pos, .. }
            | Expr::Call { pos, .. }
            | Expr::Unary { pos, .. }
            | Expr::Binary { pos, .. } => *pos,
            Expr::Name(d) => d.pos,
        }
    }
}
