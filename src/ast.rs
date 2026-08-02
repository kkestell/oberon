use crate::diag::Pos;

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub pos: Pos,
    pub imports: Vec<Import>,
    pub consts: Vec<ConstDecl>,
    pub types: Vec<TypeDecl>,
    pub vars: Vec<VarDecl>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct IdentDef {
    pub name: String,
    pub pos: Pos,
}

#[derive(Debug, Clone)]
pub struct Import {
    pub name: String,
    pub pos: Pos,
}

#[derive(Debug, Clone)]
pub struct ConstDecl {
    pub id: IdentDef,
    pub expr: Expr,
}

#[derive(Debug, Clone)]
pub struct TypeDecl {
    pub id: IdentDef,
    pub ty: TypeExpr,
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub names: Vec<IdentDef>,
    pub ty: TypeExpr,
}

// type = qualident | StrucType. The parser preserves every source constructor;
// sema resolves names and gives each constructor its language-level identity.
#[derive(Debug, Clone)]
pub enum TypeExpr {
    Named(Designator),
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
    Field(String, Pos),
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
    },
    Call {
        proc: Designator,
        args: Vec<Expr>,
        pos: Pos,
    },
}

#[derive(Debug, Clone)]
pub enum Expr {
    Int {
        value: i64,
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

// Report 8.2.2 defines unary "+" for numeric operands only, so it is a real
// operator here rather than something the parser discards.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnOp {
    Plus,
    Neg,
}

// Source operators, not resolved operations. Div is the DIV keyword.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

impl Expr {
    pub fn pos(&self) -> Pos {
        match self {
            Expr::Int { pos, .. } | Expr::Unary { pos, .. } | Expr::Binary { pos, .. } => *pos,
            Expr::Name(d) => d.pos,
        }
    }
}
