use crate::diag::Pos;

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub pos: Pos,
    pub imports: Vec<Import>,
    pub consts: Vec<ConstDecl>,
    pub types: Vec<TypeDecl>,
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
    // ArrayType = ARRAY length {"," length} OF type. The comma list is kept as
    // written: Report 6.2 defines it as nested arrays, and sema expands it.
    Array {
        lengths: Vec<Expr>,
        elem: Box<TypeExpr>,
        pos: Pos,
    },
    // RecordType = RECORD ["(" BaseType ")"] [FieldListSequence] END.
    Record {
        base: Option<Designator>,
        fields: Vec<FieldList>,
        pos: Pos,
    },
    Pointer {
        base: Box<TypeExpr>,
        pos: Pos,
    },
    Procedure {
        params: Vec<FpSection>,
        ret: Option<Box<TypeExpr>>,
        pos: Pos,
    },
}

// FieldList = IdentList ":" type. The names are identdefs, so a field may
// carry an export mark and sema decides whether its scope permits one.
#[derive(Debug, Clone)]
pub struct FieldList {
    pub names: Vec<IdentDef>,
    pub ty: TypeExpr,
}

impl TypeExpr {
    pub fn pos(&self) -> Pos {
        match self {
            TypeExpr::Named(d) => d.pos,
            TypeExpr::Array { pos, .. }
            | TypeExpr::Record { pos, .. }
            | TypeExpr::Pointer { pos, .. }
            | TypeExpr::Procedure { pos, .. } => *pos,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProcDecl {
    pub id: IdentDef,
    pub params: Vec<FpSection>,
    pub ret: Option<TypeExpr>,
    pub consts: Vec<ConstDecl>,
    pub types: Vec<TypeDecl>,
    pub vars: Vec<VarDecl>,
    pub procs: Vec<ProcDecl>,
    pub body: Vec<Stmt>,
    pub ret_val: Option<Expr>,
}

// FormalType = {ARRAY OF} qualident. The ordinary TypeExpr stays unable to
// represent an open array, so only this formal-parameter production can make
// one. Each position names one ARRAY prefix, outermost first.
#[derive(Debug, Clone)]
pub struct FpSection {
    pub var: bool,
    pub names: Vec<(String, Pos)>,
    pub open_arrays: Vec<Pos>,
    pub ty: TypeExpr,
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
    // selector = "[" ExpList "]". One source selector holds the whole comma
    // list, which Report 8.1 defines as one index selector per expression.
    Index(Vec<Expr>, Pos),
    Deref(Pos),
    Guard(Designator, Pos),
}

impl Designator {
    // For diagnostics: "Out.Int", not just the base "Out". An index selector
    // prints as "[...]": the message names the designator, and printing the
    // index expressions back out would need a whole expression printer.
    pub fn name(&self) -> String {
        let mut s = self.ident.clone();
        for sel in &self.selectors {
            match sel {
                Selector::Field(f, _) => {
                    s.push('.');
                    s.push_str(f);
                }
                Selector::Index(..) => s.push_str("[...]"),
                Selector::Deref(_) => s.push('^'),
                Selector::Guard(ty, _) => {
                    s.push('(');
                    s.push_str(&ty.name());
                    s.push(')');
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
    // Already rounded to binary32 by the lexer, so nothing downstream rounds
    // a second time.
    Real {
        value: f32,
        pos: Pos,
    },
    Bool {
        value: bool,
        pos: Pos,
    },
    Nil {
        pos: Pos,
    },
    // Both of Report 3's string forms: the lexer has already turned 41X into
    // a one-byte string, so no character literal exists as a separate node.
    Str {
        bytes: Vec<u8>,
        pos: Pos,
    },
    Name(Designator),
    // set = "{" [element {"," element}] "}". The empty constructor is legal
    // and has no elements.
    Set {
        elements: Vec<SetElement>,
        pos: Pos,
    },
    // A terminal parenthesized postfix is ambiguous until the prefix is
    // resolved: it can be a call or the final type guard of a designator.
    Apply {
        callee: Designator,
        args: Vec<Expr>,
        pos: Pos,
    },
    TypeTest {
        expr: Box<Expr>,
        ty: Designator,
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

// element = expression [".." expression]. Both endpoints keep their own
// position so an out-of-range diagnostic points at the offending value.
#[derive(Debug, Clone)]
pub struct SetElement {
    pub low: Expr,
    pub high: Option<Expr>,
}

// Report 8.2.2 defines unary "+" for numeric operands only, so it is a real
// operator here rather than something the parser discards.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnOp {
    Plus,
    Neg,
    Not,
}

// Source operators, not resolved operations: Report 8.2 overloads "+", "-",
// "*", and "/", and sema picks INTEGER or SET behaviour once it knows both
// operand types. Div is the DIV keyword; Slash is "/".
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Slash,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    In,
    And,
    Or,
}

impl Expr {
    pub fn pos(&self) -> Pos {
        match self {
            Expr::Int { pos, .. }
            | Expr::Real { pos, .. }
            | Expr::Bool { pos, .. }
            | Expr::Nil { pos }
            | Expr::Str { pos, .. }
            | Expr::Set { pos, .. }
            | Expr::Apply { pos, .. }
            | Expr::TypeTest { pos, .. }
            | Expr::Unary { pos, .. }
            | Expr::Binary { pos, .. } => *pos,
            Expr::Name(d) => d.pos,
        }
    }
}
