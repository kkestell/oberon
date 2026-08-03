use std::collections::HashMap;
use std::fmt::Write;

use crate::ir;

// Every source module of the build goes into one QBE unit, so a call between
// two of them needs no linkage annotation: Oberon export marks are a semantic
// rule that sema has already applied by this point.
pub fn emit(program: &ir::Program) -> String {
    let mut out = String::new();
    let names = Names::new(program);
    for module in &program.modules {
        for global in &module.globals {
            writeln!(
                out,
                "data ${} = align {} {{ z {} }}",
                names.symbol(&global.symbol),
                global.ty.align(),
                global.ty.size()
            )
            .unwrap();
        }
        for literal in &module.literals {
            // Every byte is a decimal item, so no escaping rule is needed and
            // every value including zero is emitted the same way. The null
            // terminator makes each object a C string.
            write!(
                out,
                "data ${} = align 1 {{ b",
                names.symbol(&literal.symbol)
            )
            .unwrap();
            for byte in &literal.bytes {
                write!(out, " {byte}").unwrap();
            }
            writeln!(out, " 0 }}").unwrap();
        }
        if !module.globals.is_empty() || !module.literals.is_empty() {
            writeln!(out).unwrap();
        }

        for proc in &module.procs {
            emit_proc(&mut out, proc, &names);
            writeln!(out).unwrap();
        }
    }

    writeln!(out, "export function w $main() {{").unwrap();
    writeln!(out, "@start").unwrap();
    // Report 11: a module body runs when the module is loaded. The list is
    // already dependency-first, so each body runs after the bodies of every
    // module it imports, and exactly once.
    for module in &program.modules {
        writeln!(
            out,
            "\tcall ${}()",
            names.symbol(&format!(".{}.init", module.name))
        )
        .unwrap();
    }
    writeln!(out, "\tret 0").unwrap();
    writeln!(out, "}}").unwrap();
    out
}

fn emit_proc(out: &mut String, proc: &ir::Proc, names: &Names) {
    let slots: HashMap<_, _> = proc
        .slots
        .iter()
        .enumerate()
        .map(|(i, (name, _))| (name.as_str(), format!("v{i}")))
        .collect();
    write!(out, "function ").unwrap();
    if let Some(ty) = proc.ret {
        write!(out, "{} ", class(ty)).unwrap();
    }
    write!(out, "${}(", names.symbol(&proc.symbol)).unwrap();
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
        // alloc4 is QBE's smallest allocation, so a byte-aligned slot is
        // over-aligned, which is harmless.
        writeln!(
            out,
            "\t%{} =l {} {}",
            slots[slot.as_str()],
            alloc(ty.align().max(4)),
            ty.size()
        )
        .unwrap();
    }

    let mut terminated = false;
    for inst in &proc.insts {
        if terminated {
            assert!(
                matches!(inst, ir::Inst::Label(_)),
                "instruction after terminator must be a label"
            );
        }
        emit_inst(out, inst, names, &slots);
        terminated = matches!(
            inst,
            ir::Inst::Jmp(_) | ir::Inst::Br { .. } | ir::Inst::Ret(_) | ir::Inst::Trap { .. }
        );
        if matches!(inst, ir::Inst::Label(_)) {
            terminated = false;
        }
    }
    writeln!(out, "}}").unwrap();
}

fn emit_inst(out: &mut String, inst: &ir::Inst, names: &Names, slots: &HashMap<&str, String>) {
    match inst {
        ir::Inst::Label(label) => writeln!(out, "@{label}").unwrap(),
        ir::Inst::Load { dst, ty, addr } => {
            writeln!(
                out,
                "\t{} ={} {} {}",
                temp(*dst),
                class(*ty),
                load_op(*ty),
                address(addr, names, slots)
            )
            .unwrap();
        }
        ir::Inst::Store { ty, val, addr } => {
            writeln!(
                out,
                "\t{} {}, {}",
                store_op(*ty),
                value(val, names),
                address(addr, names, slots)
            )
            .unwrap();
        }
        ir::Inst::Copy { dst, ty, src } => {
            writeln!(
                out,
                "\t{} ={} copy {}",
                temp(*dst),
                class(*ty),
                value(src, names)
            )
            .unwrap();
        }
        ir::Inst::Un { dst, op, ty, arg } => match op {
            ir::UnOp::Neg => {
                writeln!(
                    out,
                    "\t{} ={} neg {}",
                    temp(*dst),
                    class(*ty),
                    value(arg, names)
                )
                .unwrap();
            }
            // QBE has no logical negation, so ~b is b = 0. The operand is
            // always BOOLEAN, which is already 0 or 1 in a word.
            ir::UnOp::Not => {
                writeln!(out, "\t{} =w ceqw {}, 0", temp(*dst), value(arg, names)).unwrap();
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
                bin_op(*op),
                value(lhs, names),
                value(rhs, names)
            )
            .unwrap();
        }
        ir::Inst::Call {
            dst,
            symbol,
            args: call_args,
        } => {
            write!(out, "\t").unwrap();
            if let Some((dst, ty)) = dst {
                write!(out, "{} ={} ", temp(*dst), class(*ty)).unwrap();
            }
            write!(out, "call ${}(", names.symbol(symbol)).unwrap();
            writeln!(out, "{})", args(call_args, names, slots)).unwrap();
        }
        ir::Inst::Jmp(label) => writeln!(out, "\tjmp @{label}").unwrap(),
        ir::Inst::Br { cond, then, els } => {
            writeln!(out, "\tjnz {}, @{then}, @{els}", value(cond, names)).unwrap();
        }
        ir::Inst::Ret(value_) => {
            if let Some(value_) = value_ {
                writeln!(out, "\tret {}", value(value_, names)).unwrap();
            } else {
                writeln!(out, "\tret").unwrap();
            }
        }
        // The runtime routine never returns, but QBE still needs the block
        // to end in a terminator.
        ir::Inst::Trap { symbol, site } => {
            writeln!(
                out,
                "\tcall ${symbol}({})",
                args(&site.args(), names, slots)
            )
            .unwrap();
            writeln!(out, "\thlt").unwrap();
        }
    }
}

fn args(args: &[ir::Arg], names: &Names, slots: &HashMap<&str, String>) -> String {
    args.iter()
        .map(|a| arg(a, names, slots))
        .collect::<Vec<_>>()
        .join(", ")
}

fn arg(arg: &ir::Arg, names: &Names, slots: &HashMap<&str, String>) -> String {
    match arg {
        ir::Arg::Val(ty, val) => format!("{} {}", class(*ty), value(val, names)),
        ir::Arg::Ref(addr) => format!("l {}", address(addr, names, slots)),
    }
}

fn class(ty: ir::Ty) -> &'static str {
    match ty {
        ir::Ty::Int | ir::Ty::Bool | ir::Ty::Set => "w",
        ir::Ty::Pointer => "l",
    }
}

// The memory mnemonics come from the type, not from its register class.
fn load_op(ty: ir::Ty) -> &'static str {
    match ty {
        ir::Ty::Int | ir::Ty::Bool | ir::Ty::Set => "loadw",
        ir::Ty::Pointer => "loadl",
    }
}

fn store_op(ty: ir::Ty) -> &'static str {
    match ty {
        ir::Ty::Int | ir::Ty::Bool | ir::Ty::Set => "storew",
        ir::Ty::Pointer => "storel",
    }
}

// QBE names the alignment in the instruction rather than taking it as an
// operand, and alloc4 is its smallest form; the caller rounds a smaller
// alignment up. The other two forms are here so the day a record changes
// that, the slot is wrong loudly rather than quietly.
fn alloc(align: i64) -> &'static str {
    match align {
        4 => "alloc4",
        8 => "alloc8",
        16 => "alloc16",
        other => panic!("no QBE allocation with alignment {other}"),
    }
}

fn temp(id: usize) -> String {
    format!("%.t{id}")
}

fn value(value: &ir::Value, names: &Names) -> String {
    match value {
        ir::Value::Int(value) => value.to_string(),
        ir::Value::Bool(value) => usize::from(*value).to_string(),
        // QBE parses a word immediate as a signed 32-bit number, so a set
        // with bit 31 in it has to be spelled negative. The bits are the same.
        ir::Value::Set(bits) => (*bits as i32).to_string(),
        ir::Value::Symbol(symbol) => format!("${}", names.symbol(symbol)),
        ir::Value::Temp(id) => temp(*id),
    }
}

fn address(addr: &ir::Addr, names: &Names, slots: &HashMap<&str, String>) -> String {
    match addr {
        ir::Addr::Global(symbol) => format!("${}", names.symbol(symbol)),
        ir::Addr::Slot(name) => format!("%{}", slots[name.as_str()]),
        ir::Addr::Temp(id) => temp(*id),
    }
}

// Oberon identifiers have no length limit, while QBE identifiers do. Keep
// source names in the IR and assign compact, collision-free names only at the
// backend boundary. Unknown symbols are C runtime entry points and retain the
// spelling their definitions export.
struct Names {
    symbols: HashMap<String, String>,
}

impl Names {
    fn new(program: &ir::Program) -> Self {
        let mut symbols = HashMap::new();
        for module in &program.modules {
            for symbol in module
                .globals
                .iter()
                .map(|item| &item.symbol)
                .chain(module.literals.iter().map(|item| &item.symbol))
                .chain(module.procs.iter().map(|item| &item.symbol))
            {
                let next = symbols.len();
                symbols.insert(symbol.clone(), format!("g{next}"));
            }
        }
        Self { symbols }
    }

    fn symbol<'a>(&'a self, name: &'a str) -> &'a str {
        self.symbols.get(name).map_or(name, String::as_str)
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
        ir::BinOp::BitXor => "xor",
    }
}
