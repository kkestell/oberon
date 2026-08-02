// TODO: a typed IR belongs between sema and this file (see AGENTS.md
// architecture); it arrives when there is real lowering — addresses, type
// descriptors, bounds checks — to justify it.

use std::fmt::Write;

use crate::ast;
use crate::sema::{Scope, Symbol, Value, resolve};

pub fn emit(module: &ast::Module, scope: Scope) -> String {
    let mut g = Gen {
        out: String::new(),
        scope,
        tmp: 0,
        labels: 0,
    };
    g.module(module);
    g.out
}

struct Gen {
    out: String,
    scope: Scope,
    tmp: usize,
    labels: usize,
}

impl Gen {
    // Temps are %.t0, %.t1, …: a leading '.' is legal in QBE and cannot
    // collide with Oberon identifiers, so VARs keep their source names.
    fn temp(&mut self) -> String {
        let t = format!("%.t{}", self.tmp);
        self.tmp += 1;
        t
    }

    // Labels also start with '.', so no source-level Oberon name can collide.
    fn label(&mut self, kind: &str) -> String {
        let label = format!("@.{kind}{}", self.labels);
        self.labels += 1;
        label
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
            ast::Stmt::If {
                cond,
                then,
                elsifs,
                els,
            } => self.if_statement(cond, then, elsifs, els.as_deref()),
            ast::Stmt::While { cond, body, elsifs } => self.while_statement(cond, body, elsifs),
            ast::Stmt::Repeat { body, cond } => self.repeat_statement(body, cond),
        }
    }

    fn if_statement(
        &mut self,
        cond: &ast::Expr,
        then: &[ast::Stmt],
        elsifs: &[(ast::Expr, Vec<ast::Stmt>)],
        els: Option<&[ast::Stmt]>,
    ) {
        let end = self.label("if.end");

        self.guarded_body(cond, then, &end);
        for (cond, body) in elsifs {
            self.guarded_body(cond, body, &end);
        }

        if let Some(body) = els {
            for stmt in body {
                self.stmt(stmt);
            }
        }
        writeln!(self.out, "\tjmp {end}").unwrap();
        writeln!(self.out, "{end}").unwrap();
    }

    // Emits a condition and its body. The false label is left as the current
    // block so an ELSIF condition or ELSE body can follow immediately.
    fn guarded_body(&mut self, cond: &ast::Expr, body: &[ast::Stmt], end: &str) {
        let yes = self.label("guard.yes");
        let no = self.label("guard.no");
        let cond = self.expr(cond);
        writeln!(self.out, "\tjnz {cond}, {yes}, {no}").unwrap();
        writeln!(self.out, "{yes}").unwrap();
        for stmt in body {
            self.stmt(stmt);
        }
        writeln!(self.out, "\tjmp {end}").unwrap();
        writeln!(self.out, "{no}").unwrap();
    }

    fn while_statement(
        &mut self,
        cond: &ast::Expr,
        body: &[ast::Stmt],
        elsifs: &[(ast::Expr, Vec<ast::Stmt>)],
    ) {
        let test = self.label("while.test");
        let end = self.label("while.end");
        writeln!(self.out, "\tjmp {test}").unwrap();
        writeln!(self.out, "{test}").unwrap();

        self.loop_guard(cond, body, &test);
        for (cond, body) in elsifs {
            self.loop_guard(cond, body, &test);
        }

        writeln!(self.out, "\tjmp {end}").unwrap();
        writeln!(self.out, "{end}").unwrap();
    }

    // Report 9.6: after any WHILE or ELSIF body, control returns to the first
    // guard. The false label is where the next ELSIF guard is evaluated.
    fn loop_guard(&mut self, cond: &ast::Expr, body: &[ast::Stmt], test: &str) {
        let yes = self.label("while.body");
        let no = self.label("while.next");
        let cond = self.expr(cond);
        writeln!(self.out, "\tjnz {cond}, {yes}, {no}").unwrap();
        writeln!(self.out, "{yes}").unwrap();
        for stmt in body {
            self.stmt(stmt);
        }
        writeln!(self.out, "\tjmp {test}").unwrap();
        writeln!(self.out, "{no}").unwrap();
    }

    fn repeat_statement(&mut self, body: &[ast::Stmt], cond: &ast::Expr) {
        let body_label = self.label("repeat.body");
        let end = self.label("repeat.end");
        writeln!(self.out, "\tjmp {body_label}").unwrap();
        writeln!(self.out, "{body_label}").unwrap();
        for stmt in body {
            self.stmt(stmt);
        }
        let cond = self.expr(cond);
        writeln!(self.out, "\tjnz {cond}, {end}, {body_label}").unwrap();
        writeln!(self.out, "{end}").unwrap();
    }

    // Returns a QBE operand: a literal ("7") or a temp ("%.t3").
    fn expr(&mut self, e: &ast::Expr) -> String {
        match e {
            ast::Expr::Int { value, .. } => value.to_string(),
            ast::Expr::Bool { value, .. } => usize::from(*value).to_string(),
            ast::Expr::Name(d) => {
                let sym = resolve(&self.scope, d).expect("sema resolved this").clone();
                match sym {
                    Symbol::Const(Value::Int(v)) => v.to_string(),
                    Symbol::Const(Value::Bool(v)) => usize::from(v).to_string(),
                    Symbol::Var(_) => {
                        let t = self.temp();
                        writeln!(self.out, "\t{t} =w loadw %{}", d.ident).unwrap();
                        t
                    }
                    _ => panic!("sema checked value use"),
                }
            }
            ast::Expr::Unary { op, expr, .. } => {
                let v = self.expr(expr);
                let t = self.temp();
                match op {
                    ast::UnOp::Neg => writeln!(self.out, "\t{t} =w neg {v}").unwrap(),
                    ast::UnOp::Not => writeln!(self.out, "\t{t} =w ceqw {v}, 0").unwrap(),
                }
                t
            }
            ast::Expr::Binary { op, lhs, rhs, .. } => {
                if matches!(op, ast::BinOp::And | ast::BinOp::Or) {
                    return self.logical(*op, lhs, rhs);
                }
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
                    ast::BinOp::Eq => self.arith("ceqw", &l, &r),
                    ast::BinOp::Ne => self.arith("cnew", &l, &r),
                    ast::BinOp::Lt => self.arith("csltw", &l, &r),
                    ast::BinOp::Le => self.arith("cslew", &l, &r),
                    ast::BinOp::Gt => self.arith("csgtw", &l, &r),
                    ast::BinOp::Ge => self.arith("csgew", &l, &r),
                    ast::BinOp::And | ast::BinOp::Or => unreachable!(),
                }
            }
        }
    }

    fn logical(&mut self, op: ast::BinOp, lhs: &ast::Expr, rhs: &ast::Expr) -> String {
        let rhs_label = self.label("logic.rhs");
        let short_label = self.label("logic.short");
        let end = self.label("logic.end");
        let result = self.temp();
        let lhs = self.expr(lhs);

        match op {
            ast::BinOp::And => {
                writeln!(self.out, "\tjnz {lhs}, {rhs_label}, {short_label}").unwrap();
            }
            ast::BinOp::Or => {
                writeln!(self.out, "\tjnz {lhs}, {short_label}, {rhs_label}").unwrap();
            }
            _ => unreachable!(),
        }

        writeln!(self.out, "{short_label}").unwrap();
        let short_value = usize::from(op == ast::BinOp::Or);
        writeln!(self.out, "\t{result} =w copy {short_value}").unwrap();
        writeln!(self.out, "\tjmp {end}").unwrap();

        writeln!(self.out, "{rhs_label}").unwrap();
        let rhs = self.expr(rhs);
        writeln!(self.out, "\t{result} =w copy {rhs}").unwrap();
        writeln!(self.out, "\tjmp {end}").unwrap();

        writeln!(self.out, "{end}").unwrap();
        result
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
