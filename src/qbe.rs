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
        for descriptor in &module.descriptors {
            match &descriptor.base {
                Some(base) => writeln!(
                    out,
                    "data ${} = align 8 {{ l ${} }}",
                    names.symbol(&descriptor.symbol),
                    names.symbol(base)
                )
                .unwrap(),
                None => writeln!(
                    out,
                    "data ${} = align 8 {{ l 0 }}",
                    names.symbol(&descriptor.symbol)
                )
                .unwrap(),
            }
        }
        for global in &module.globals {
            // A zero-length array reserves nothing; QBE accepts `z 0` and the
            // assembler emits an empty, still-addressable object.
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
            // every value including zero is emitted the same way. The
            // terminator the Report appends on assignment is part of the data
            // object, so one literal serves both the copy and the comparison.
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
        if !module.descriptors.is_empty()
            || !module.globals.is_empty()
            || !module.literals.is_empty()
        {
            writeln!(out).unwrap();
        }

        for proc in &module.procs {
            emit_proc(&mut out, proc, &names);
            writeln!(out).unwrap();
        }
    }

    writeln!(out, "export function w $main(w %argc, l %argv) {{").unwrap();
    writeln!(out, "@start").unwrap();
    writeln!(out, "\tcall $oberon_init(w %argc, l %argv)").unwrap();
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
            // always BOOLEAN, which is already 0 or 1 in a long.
            ir::UnOp::Not => {
                writeln!(out, "\t{} =l ceql {}, 0", temp(*dst), value(arg, names)).unwrap();
            }
        },
        ir::Inst::Bin {
            dst,
            op,
            ty,
            lhs,
            rhs,
        } => {
            let result = if op.is_comparison() { "l" } else { class(*ty) };
            writeln!(
                out,
                "\t{} ={result} {} {}, {}",
                temp(*dst),
                bin_op(*op, *ty),
                value(lhs, names),
                value(rhs, names)
            )
            .unwrap();
        }
        ir::Inst::IntToReal { dst, arg } => {
            writeln!(out, "\t{} =d sltof {}", temp(*dst), value(arg, names)).unwrap();
        }
        ir::Inst::CheckNil { pointer, site } => {
            writeln!(
                out,
                "\tcall $oberon_check_nil(l {}, {})",
                value(pointer, names),
                args(&site.args(), names, slots)
            )
            .unwrap();
        }
        ir::Inst::CheckProcedure { procedure, site } => {
            writeln!(
                out,
                "\tcall $oberon_check_procedure(l {}, {})",
                value(procedure, names),
                args(&site.args(), names, slots)
            )
            .unwrap();
        }
        ir::Inst::Alloc {
            dst,
            size,
            scanned,
            descriptor,
        } => {
            let symbol = if *scanned {
                "oberon_alloc"
            } else {
                "oberon_alloc_atomic"
            };
            writeln!(
                out,
                "\t{} =l call ${symbol}(l {size}, l ${})",
                temp(*dst),
                names.symbol(descriptor)
            )
            .unwrap();
        }
        ir::Inst::HeapDescriptor { dst, pointer } => {
            writeln!(
                out,
                "\t{} =l call $oberon_heap_descriptor(l {})",
                temp(*dst),
                value(pointer, names)
            )
            .unwrap();
        }
        ir::Inst::TypeTestPointer {
            dst,
            pointer,
            target,
        } => {
            writeln!(
                out,
                "\t{} =l call $oberon_type_test_pointer(l {}, l ${})",
                temp(*dst),
                value(pointer, names),
                names.symbol(target)
            )
            .unwrap();
        }
        ir::Inst::TypeTestDescriptor {
            dst,
            descriptor,
            target,
        } => {
            writeln!(
                out,
                "\t{} =l call $oberon_type_test_descriptor(l {}, l ${})",
                temp(*dst),
                value(descriptor, names),
                names.symbol(target)
            )
            .unwrap();
        }
        // The check is a call, so it is textually and dynamically ahead of
        // every part of the address calculation: nothing scales or adds an
        // index the runtime has not accepted. The intermediate temporaries
        // are named from the destination number, so they cannot collide with
        // the value temporaries, which are all `%.t<n>`. Every operand is
        // already a long, so the scaling needs no extension.
        ir::Inst::Index {
            dst,
            base,
            index,
            len,
            stride,
            dynamic_stride,
            site,
        } => {
            writeln!(
                out,
                "\t%.i{dst} =l call $oberon_check_index(l {}, l {}, {})",
                value(index, names),
                value(len, names),
                args(&site.args(), names, slots)
            )
            .unwrap();
            if dynamic_stride.is_empty() {
                writeln!(out, "\t%.s{dst} =l mul %.i{dst}, {stride}").unwrap();
                writeln!(
                    out,
                    "\t{} =l add {}, %.s{dst}",
                    temp(*dst),
                    address(base, names, slots)
                )
                .unwrap();
            } else {
                writeln!(out, "\t%.s{dst}.0 =l mul %.i{dst}, {stride}").unwrap();
                for (i, factor) in dynamic_stride.iter().enumerate() {
                    writeln!(
                        out,
                        "\t%.s{dst}.{} =l mul %.s{dst}.{i}, {}",
                        i + 1,
                        value(factor, names)
                    )
                    .unwrap();
                }
                writeln!(
                    out,
                    "\t{} =l add {}, %.s{dst}.{}",
                    temp(*dst),
                    address(base, names, slots),
                    dynamic_stride.len()
                )
                .unwrap();
            }
        }
        // One add, offset zero included: there is no optimization pass, and
        // one literal path is the same choice the constant index made.
        ir::Inst::Field { dst, base, offset } => {
            writeln!(
                out,
                "\t{} =l add {}, {offset}",
                temp(*dst),
                address(base, names, slots)
            )
            .unwrap();
        }
        ir::Inst::CopyBytes { dst, src, size } => {
            writeln!(
                out,
                "\tcall $oberon_copy(l {}, l {}, l {size})",
                address(dst, names, slots),
                address(src, names, slots)
            )
            .unwrap();
        }
        ir::Inst::CheckArrayCopy {
            source_len,
            destination_len,
            site,
        } => {
            writeln!(
                out,
                "\tcall $oberon_check_array_copy(l {}, l {}, {})",
                value(source_len, names),
                value(destination_len, names),
                args(&site.args(), names, slots)
            )
            .unwrap();
        }
        ir::Inst::CopyElements {
            temp: copy_temp,
            dst,
            src,
            count,
            stride,
        } => {
            writeln!(
                out,
                "\t%.b{copy_temp} =l mul {}, {stride}",
                value(count, names)
            )
            .unwrap();
            writeln!(
                out,
                "\tcall $oberon_copy(l {}, l {}, l %.b{copy_temp})",
                address(dst, names, slots),
                address(src, names, slots)
            )
            .unwrap();
        }
        ir::Inst::Call {
            dst,
            target,
            args: call_args,
        } => {
            write!(out, "\t").unwrap();
            if let Some((dst, ty)) = dst {
                write!(out, "{} ={} ", temp(*dst), class(*ty)).unwrap();
            }
            match target {
                ir::CallTarget::Direct(symbol) => {
                    write!(out, "call ${}(", names.symbol(symbol)).unwrap()
                }
                ir::CallTarget::Indirect(value_) => {
                    write!(out, "call {}(", value(value_, names)).unwrap()
                }
            }
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

// QBE passes a `d` in the platform's floating-point class and an `l` in its
// integer class, which is the native C double and int64_t ABI. Every
// integer-like value is a long in every calling position; only the memory
// traffic of a BOOLEAN and a byte is narrower.
fn class(ty: ir::Ty) -> &'static str {
    match ty {
        ir::Ty::Real => "d",
        ir::Ty::Int
        | ir::Ty::Bool
        | ir::Ty::Set
        | ir::Ty::Byte
        | ir::Ty::Pointer
        | ir::Ty::Procedure => "l",
    }
}

// The memory mnemonics come from the type, not from its register class: a
// BOOLEAN and a byte load zero-extended into a long, and a store keeps only
// the low four bytes or the low byte. An `l` may stand where a `w` operand
// is expected, so the narrow stores need no truncation.
fn load_op(ty: ir::Ty) -> &'static str {
    match ty {
        ir::Ty::Int | ir::Ty::Set | ir::Ty::Pointer | ir::Ty::Procedure => "loadl",
        ir::Ty::Bool => "loaduw",
        ir::Ty::Real => "loadd",
        ir::Ty::Byte => "loadub",
    }
}

fn store_op(ty: ir::Ty) -> &'static str {
    match ty {
        ir::Ty::Int | ir::Ty::Set | ir::Ty::Pointer | ir::Ty::Procedure => "storel",
        ir::Ty::Bool => "storew",
        ir::Ty::Real => "stored",
        ir::Ty::Byte => "storeb",
    }
}

// QBE names the alignment in the instruction rather than taking it as an
// operand, and alloc4 is its smallest form; the caller rounds a smaller
// alignment up. An eight-byte scalar takes alloc8, and alloc16 is here so
// the day a record needs it, the slot is wrong loudly rather than quietly.
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
        // QBE parses an immediate as a signed 64-bit number, so a set with
        // bit 63 in it has to be spelled negative. The bits are the same.
        ir::Value::Set(bits) => (*bits as i64).to_string(),
        // A QBE constant is an untyped bit string, so in a `d` context the
        // signed spelling of the binary64 pattern reproduces the value
        // exactly. That covers negative zero and the infinities and NaNs
        // constant arithmetic can produce, none of which a decimal spelling
        // states directly.
        ir::Value::Real(v) => (v.to_bits() as i64).to_string(),
        ir::Value::Byte(v) => v.to_string(),
        ir::Value::Pointer(v) => v.to_string(),
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
                .descriptors
                .iter()
                .map(|item| &item.symbol)
                .chain(module.globals.iter().map(|item| &item.symbol))
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

// The arithmetic mnemonics are shared: `div` on two doubles is already the
// floating quotient. Only comparisons differ, because QBE names the operand
// class in the instruction and gives a signedness to the integer forms only.
// The remaining operations are integer-only and never see a REAL operand.
fn bin_op(op: ir::BinOp, ty: ir::Ty) -> &'static str {
    let real = ty == ir::Ty::Real;
    match op {
        ir::BinOp::Add => "add",
        ir::BinOp::Sub => "sub",
        ir::BinOp::Mul => "mul",
        ir::BinOp::Div => "div",
        ir::BinOp::Rem => "rem",
        ir::BinOp::Eq if real => "ceqd",
        ir::BinOp::Ne if real => "cned",
        ir::BinOp::Lt if real => "cltd",
        ir::BinOp::Le if real => "cled",
        ir::BinOp::Gt if real => "cgtd",
        ir::BinOp::Ge if real => "cged",
        ir::BinOp::Eq => "ceql",
        ir::BinOp::Ne => "cnel",
        ir::BinOp::Lt => "csltl",
        ir::BinOp::Le => "cslel",
        ir::BinOp::Gt => "csgtl",
        ir::BinOp::Ge => "csgel",
        ir::BinOp::Shl => "shl",
        ir::BinOp::Shr => "shr",
        ir::BinOp::Sar => "sar",
        ir::BinOp::BitAnd => "and",
        ir::BinOp::BitOr => "or",
        ir::BinOp::BitXor => "xor",
    }
}
