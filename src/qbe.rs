// TODO: a typed IR belongs between sema and this file (see AGENTS.md
// architecture); it arrives when there is real lowering — addresses, type
// descriptors, bounds checks — to justify it.

use std::fmt::Write;

use crate::ast;
use crate::sema::{Scope, Symbol, resolve};

pub fn emit(module: &ast::Module, scope: Scope) -> String {
    let mut g = Gen {
        out: String::new(),
        scope,
        tmp: 0,
    };
    g.module(module);
    g.out
}

struct Gen {
    out: String,
    scope: Scope,
    tmp: usize,
}

impl Gen {
    // Temps are %.t0, %.t1, …: a leading '.' is legal in QBE and cannot
    // collide with Oberon identifiers, so VARs keep their source names.
    fn temp(&mut self) -> String {
        let t = format!("%.t{}", self.tmp);
        self.tmp += 1;
        t
    }

    fn module(&mut self, m: &ast::Module) {
        // Mangling is $<Module>_<name>: '_' cannot appear in Oberon idents.
        writeln!(self.out, "function ${}_init() {{", m.name).unwrap();
        writeln!(self.out, "@start").unwrap();
        for v in &m.vars {
            for (name, _) in &v.names {
                // TODO: must become data globals once procedures reference them.
                writeln!(self.out, "\t%{name} =l alloc4 4").unwrap();
            }
        }
        for stmt in &m.body {
            self.stmt(stmt);
        }
        writeln!(self.out, "\tret").unwrap();
        writeln!(self.out, "}}").unwrap();

        writeln!(self.out, "export function w $main() {{").unwrap();
        writeln!(self.out, "@start").unwrap();
        writeln!(self.out, "\tcall $oberon_init()").unwrap();
        writeln!(self.out, "\tcall ${}_init()", m.name).unwrap();
        writeln!(self.out, "\tret 0").unwrap();
        writeln!(self.out, "}}").unwrap();
    }

    fn stmt(&mut self, stmt: &ast::Stmt) {
        match stmt {
            ast::Stmt::Assign { lhs, rhs, .. } => {
                let v = self.expr(rhs);
                writeln!(self.out, "\tstorew {v}, %{}", lhs.ident).unwrap();
            }
            ast::Stmt::Call { proc, args, .. } => {
                let sym = resolve(&self.scope, proc).expect("sema resolved this");
                let Symbol::Proc { runtime_name, .. } = sym else {
                    panic!("sema checked the call target")
                };
                let runtime_name = runtime_name.clone();
                let vals: Vec<String> = args.iter().map(|a| self.expr(a)).collect();
                let arglist = vals
                    .iter()
                    .map(|v| format!("w {v}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                writeln!(self.out, "\tcall ${runtime_name}({arglist})").unwrap();
            }
        }
    }

    // Returns a QBE operand: a literal ("7") or a temp ("%.t3").
    fn expr(&mut self, e: &ast::Expr) -> String {
        match e {
            ast::Expr::Int { value, .. } => value.to_string(),
            ast::Expr::Name(d) => {
                let sym = resolve(&self.scope, d).expect("sema resolved this").clone();
                match sym {
                    Symbol::Const(v) => v.to_string(),
                    Symbol::Var => {
                        let t = self.temp();
                        writeln!(self.out, "\t{t} =w loadw %{}", d.ident).unwrap();
                        t
                    }
                    _ => panic!("sema checked value use"),
                }
            }
            ast::Expr::Unary {
                op: ast::UnOp::Neg,
                expr,
                ..
            } => {
                let v = self.expr(expr);
                let t = self.temp();
                writeln!(self.out, "\t{t} =w neg {v}").unwrap();
                t
            }
            ast::Expr::Binary { op, lhs, rhs, .. } => {
                let l = self.expr(lhs);
                let r = self.expr(rhs);
                match op {
                    ast::BinOp::Add => self.arith("add", &l, &r),
                    ast::BinOp::Sub => self.arith("sub", &l, &r),
                    ast::BinOp::Mul => self.arith("mul", &l, &r),
                    ast::BinOp::Mod => self.floored_mod(&l, &r),
                    // Once the remainder floors, the quotient divides exactly.
                    ast::BinOp::Div => {
                        let m = self.floored_mod(&l, &r);
                        let n = self.arith("sub", &l, &m);
                        self.arith("div", &n, &r)
                    }
                }
            }
        }
    }

    fn arith(&mut self, op: &str, l: &str, r: &str) -> String {
        let t = self.temp();
        writeln!(self.out, "\t{t} =w {op} {l}, {r}").unwrap();
        t
    }

    // Report 8.2.2 requires 0 <= x MOD y < y, but QBE's rem truncates toward
    // zero, so -7 MOD 2 would come out -1 instead of 1. ((x REM y) + y) REM y
    // is the floored remainder for every combination of signs and needs no
    // branches, which keeps procedure bodies straight-line until control flow
    // lands. cf. obnc lib/obnc/OBNC.h OBNC_MOD. sema.rs folds constants with
    // this same formula so the two stages cannot drift apart.
    fn floored_mod(&mut self, l: &str, r: &str) -> String {
        let t = self.arith("rem", l, r);
        let t = self.arith("add", &t, r);
        self.arith("rem", &t, r)
    }
}
