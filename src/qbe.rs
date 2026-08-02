use std::fmt::Write;

use crate::ir;

pub fn emit(program: &ir::Program) -> String {
    let mut out = String::new();
    for global in &program.globals {
        let size = match global.ty {
            ir::Ty::Int | ir::Ty::Bool => 4,
        };
        writeln!(out, "data ${} = align {size} {{ z {size} }}", global.symbol).unwrap();
    }
    if !program.globals.is_empty() {
        writeln!(out).unwrap();
    }

    for proc in &program.procs {
        emit_proc(&mut out, proc);
        writeln!(out).unwrap();
    }

    writeln!(out, "export function w $main() {{").unwrap();
    writeln!(out, "@start").unwrap();
    writeln!(out, "\tcall $oberon_init()").unwrap();
    writeln!(out, "\tcall $.{}.init()", program.module).unwrap();
    writeln!(out, "\tret 0").unwrap();
    writeln!(out, "}}").unwrap();
    out
}

fn emit_proc(out: &mut String, proc: &ir::Proc) {
    write!(out, "function ").unwrap();
    if let Some(ty) = proc.ret {
        write!(out, "{} ", class(ty)).unwrap();
    }
    write!(out, "${}(", proc.symbol).unwrap();
    for (i, param) in proc.params.iter().enumerate() {
        if i != 0 {
            write!(out, ", ").unwrap();
        }
        let class = match param.pass {
            ir::ParamPass::Value(ty) => class(ty),
            ir::ParamPass::Ref => "l",
        };
        write!(out, "{class} {}", temp(param.temp)).unwrap();
    }
    writeln!(out, ") {{").unwrap();
    writeln!(out, "@start").unwrap();
    for (slot, ty) in &proc.slots {
        let bytes = match ty {
            ir::Ty::Int | ir::Ty::Bool => 4,
        };
        writeln!(out, "\t%{slot} =l alloc4 {bytes}").unwrap();
    }

    let mut terminated = false;
    for inst in &proc.insts {
        if terminated {
            assert!(
                matches!(inst, ir::Inst::Label(_)),
                "instruction after terminator must be a label"
            );
        }
        emit_inst(out, inst);
        terminated = matches!(
            inst,
            ir::Inst::Jmp(_) | ir::Inst::Br { .. } | ir::Inst::Ret(_) | ir::Inst::Halt
        );
        if matches!(inst, ir::Inst::Label(_)) {
            terminated = false;
        }
    }
    writeln!(out, "}}").unwrap();
}

fn emit_inst(out: &mut String, inst: &ir::Inst) {
    match inst {
        ir::Inst::Label(label) => writeln!(out, "@{label}").unwrap(),
        ir::Inst::Load { dst, ty, addr } => {
            writeln!(
                out,
                "\t{} ={} load{} {}",
                temp(*dst),
                class(*ty),
                class(*ty),
                address(addr)
            )
            .unwrap();
        }
        ir::Inst::Store { ty, val, addr } => {
            writeln!(
                out,
                "\tstore{} {}, {}",
                class(*ty),
                value(val),
                address(addr)
            )
            .unwrap();
        }
        ir::Inst::Copy { dst, src } => {
            writeln!(out, "\t{} =w copy {}", temp(*dst), value(src)).unwrap();
        }
        ir::Inst::Un { dst, op, arg } => {
            let op = match op {
                ir::UnOp::Neg => "neg",
                ir::UnOp::Not => "ceqw",
            };
            if matches!(op, "ceqw") {
                writeln!(out, "\t{} =w {op} {}, 0", temp(*dst), value(arg)).unwrap();
            } else {
                writeln!(out, "\t{} =w {op} {}", temp(*dst), value(arg)).unwrap();
            }
        }
        ir::Inst::Bin { dst, op, lhs, rhs } => {
            writeln!(
                out,
                "\t{} =w {} {}, {}",
                temp(*dst),
                bin_op(*op),
                value(lhs),
                value(rhs)
            )
            .unwrap();
        }
        ir::Inst::Call { dst, symbol, args } => {
            write!(out, "\t").unwrap();
            if let Some((dst, ty)) = dst {
                write!(out, "{} ={} ", temp(*dst), class(*ty)).unwrap();
            }
            write!(out, "call ${symbol}(").unwrap();
            for (i, arg) in args.iter().enumerate() {
                if i != 0 {
                    write!(out, ", ").unwrap();
                }
                match arg {
                    ir::Arg::Val(ty, val) => write!(out, "{} {}", class(*ty), value(val)).unwrap(),
                    ir::Arg::Ref(addr) => write!(out, "l {}", address(addr)).unwrap(),
                }
            }
            writeln!(out, ")").unwrap();
        }
        ir::Inst::Jmp(label) => writeln!(out, "\tjmp @{label}").unwrap(),
        ir::Inst::Br { cond, then, els } => {
            writeln!(out, "\tjnz {}, @{then}, @{els}", value(cond)).unwrap();
        }
        ir::Inst::Ret(value_) => {
            if let Some(value_) = value_ {
                writeln!(out, "\tret {}", value(value_)).unwrap();
            } else {
                writeln!(out, "\tret").unwrap();
            }
        }
        ir::Inst::Halt => writeln!(out, "\thlt").unwrap(),
    }
}

fn class(ty: ir::Ty) -> &'static str {
    match ty {
        ir::Ty::Int | ir::Ty::Bool => "w",
    }
}

fn temp(id: usize) -> String {
    format!("%.t{id}")
}

fn value(value: &ir::Value) -> String {
    match value {
        ir::Value::Int(value) => value.to_string(),
        ir::Value::Bool(value) => usize::from(*value).to_string(),
        ir::Value::Temp(id) => temp(*id),
    }
}

fn address(addr: &ir::Addr) -> String {
    match addr {
        ir::Addr::Global(symbol) => format!("${symbol}"),
        ir::Addr::Slot(name) => format!("%{name}"),
        ir::Addr::Temp(id) => temp(*id),
    }
}

fn bin_op(op: ir::BinOp) -> &'static str {
    match op {
        ir::BinOp::Add => "add",
        ir::BinOp::Sub => "sub",
        ir::BinOp::Mul => "mul",
        ir::BinOp::Div => "div",
        ir::BinOp::Rem => "rem",
        ir::BinOp::Eq => "ceqw",
        ir::BinOp::Ne => "cnew",
        ir::BinOp::Lt => "csltw",
        ir::BinOp::Le => "cslew",
        ir::BinOp::Gt => "csgtw",
        ir::BinOp::Ge => "csgew",
        ir::BinOp::Shl => "shl",
        ir::BinOp::Shr => "shr",
        ir::BinOp::Sar => "sar",
        ir::BinOp::BitAnd => "and",
        ir::BinOp::BitOr => "or",
    }
}
