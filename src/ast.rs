use crate::diag::Pos;

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    #[allow(dead_code)] // diagnostics will want these positions soon
    pub pos: Pos,
    pub imports: Vec<Import>,
    pub consts: Vec<ConstDecl>,
    pub vars: Vec<VarDecl>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct Import {
    pub name: String,
    pub alias: Option<String>,
    pub pos: Pos,
}

#[derive(Debug, Clone)]
pub struct ConstDecl {
    pub name: String,
    pub pos: Pos,
    pub expr: Expr,
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub names: Vec<(String, Pos)>,
    pub ty: Designator, // TODO: enum when StrucType lands
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
            | Expr::Unary { pos, .. }
            | Expr::Binary { pos, .. } => *pos,
            Expr::Name(d) => d.pos,
        }
    }
}
