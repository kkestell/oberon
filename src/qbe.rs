use std::fmt::Write;

use crate::ir;

// Every source module of the build goes into one QBE unit, so a call between
// two of them needs no linkage annotation: Oberon export marks are a semantic
// rule that sema has already applied by this point.
pub fn emit(program: &ir::Program) -> String {
    let mut out = String::new();
    for module in &program.modules {
        for global in &module.globals {
            let size = size(global.ty);
            writeln!(out, "data ${} = align {size} {{ z {size} }}", global.symbol).unwrap();
        }
        if !module.globals.is_empty() {
            writeln!(out).unwrap();
        }

        for proc in &module.procs {
            emit_proc(&mut out, proc);
            writeln!(out).unwrap();
        }
    }

    writeln!(out, "export function w $main() {{").unwrap();
    writeln!(out, "@start").unwrap();
    writeln!(out, "\tcall $oberon_init()").unwrap();
    // Report 11: a module body runs when the module is loaded. The list is
    // already dependency-first, so each body runs after the bodies of every
    // module it imports, and exactly once.
    for module in &program.modules {
        writeln!(out, "\tcall $.{}.init()", module.name).unwrap();
    }
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
        writeln!(out, "\t%{slot} =l alloc4 {}", size(*ty)).unwrap();
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
        ir::Inst::Copy { dst, ty, src } => {
            writeln!(out, "\t{} ={} copy {}", temp(*dst), class(*ty), value(src)).unwrap();
        }
        ir::Inst::Un { dst, op, ty, arg } => match op {
            ir::UnOp::Neg => {
                writeln!(out, "\t{} ={} neg {}", temp(*dst), class(*ty), value(arg)).unwrap();
            }
            // QBE has no logical negation, so ~b is b = 0. The operand is
            // always BOOLEAN, which is already 0 or 1 in a word.
            ir::UnOp::Not => {
                writeln!(out, "\t{} =w ceqw {}, 0", temp(*dst), value(arg)).unwrap();
            }
        },
        ir::Inst::Bin {
            dst,
            op,
            ty,
            lhs,
            rhs,
        } => {
            let result = if op.is_comparison() { "w" } else { class(*ty) };
            writeln!(
                out,
                "\t{} ={result} {} {}, {}",
                temp(*dst),
                bin_op(*op, *ty),
                value(lhs),
                value(rhs)
            )
            .unwrap();
        }
        ir::Inst::IntToReal { dst, arg } => {
            writeln!(out, "\t{} =s swtof {}", temp(*dst), value(arg)).unwrap();
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

// QBE passes an `s` in the platform's floating-point class and a `w` in its
// integer class, which is the native C float and int32_t ABI. Nothing here
// needs a compiler-specific calling convention.
fn class(ty: ir::Ty) -> &'static str {
    match ty {
        ir::Ty::Int | ir::Ty::Bool | ir::Ty::Set => "w",
        ir::Ty::Real => "s",
    }
}

fn size(ty: ir::Ty) -> u32 {
    match ty {
        ir::Ty::Int | ir::Ty::Bool | ir::Ty::Set | ir::Ty::Real => 4,
    }
}

fn temp(id: usize) -> String {
    format!("%.t{id}")
}

fn value(value: &ir::Value) -> String {
    match value {
        ir::Value::Int(value) => value.to_string(),
        ir::Value::Bool(value) => usize::from(*value).to_string(),
        // QBE parses a word immediate as a signed 32-bit number, so a set
        // with bit 31 in it has to be spelled negative. The bits are the same.
        ir::Value::Set(bits) => (*bits as i32).to_string(),
        // A QBE constant is an untyped bit string, so in an `s` context the
        // signed spelling of the binary32 pattern reproduces the value
        // exactly. That covers negative zero and the infinities and NaNs
        // constant arithmetic can produce, none of which a decimal spelling
        // states directly.
        ir::Value::Real(v) => (v.to_bits() as i32).to_string(),
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

// The arithmetic mnemonics are shared: `div` on two singles is already the
// floating quotient. Only comparisons differ, because QBE names the operand
// class in the instruction and gives a signedness to the integer forms only.
// The remaining operations are word-only and never see a REAL operand.
fn bin_op(op: ir::BinOp, ty: ir::Ty) -> &'static str {
    let real = ty == ir::Ty::Real;
    match op {
        ir::BinOp::Add => "add",
        ir::BinOp::Sub => "sub",
        ir::BinOp::Mul => "mul",
        ir::BinOp::Div => "div",
        ir::BinOp::Rem => "rem",
        ir::BinOp::Eq if real => "ceqs",
        ir::BinOp::Ne if real => "cnes",
        ir::BinOp::Lt if real => "clts",
        ir::BinOp::Le if real => "cles",
        ir::BinOp::Gt if real => "cgts",
        ir::BinOp::Ge if real => "cges",
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
        ir::BinOp::BitXor => "xor",
    }
}
