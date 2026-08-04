use std::fmt::Write;

use crate::ir;

// Every source module of the build goes into one QBE unit, so a call between
// two of them needs no linkage annotation: Oberon export marks are a semantic
// rule that sema has already applied by this point.
pub fn emit(program: &ir::Program) -> String {
    let mut out = String::new();
    for module in &program.modules {
        for descriptor in &module.descriptors {
            match &descriptor.base {
                Some(base) => writeln!(
                    out,
                    "data ${} = align 8 {{ l ${} }}",
                    descriptor.symbol, base
                )
                .unwrap(),
                None => writeln!(out, "data ${} = align 8 {{ l 0 }}", descriptor.symbol).unwrap(),
            }
        }
        for global in &module.globals {
            // A zero-length array reserves nothing; QBE accepts `z 0` and the
            // assembler emits an empty, still-addressable object.
            writeln!(
                out,
                "data ${} = align {} {{ z {} }}",
                global.symbol,
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
            write!(out, "data ${} = align 1 {{ b", literal.symbol).unwrap();
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
        // alloc4 is QBE's smallest allocation, so a byte-aligned slot is
        // over-aligned, which is harmless.
        writeln!(
            out,
            "\t%{slot} =l {} {}",
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
                "\t{} ={} {} {}",
                temp(*dst),
                class(*ty),
                load_op(*ty),
                address(addr)
            )
            .unwrap();
        }
        ir::Inst::Store { ty, val, addr } => {
            writeln!(out, "\t{} {}, {}", store_op(*ty), value(val), address(addr)).unwrap();
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
        ir::Inst::CheckNil { pointer } => {
            writeln!(out, "\tcall $oberon_check_nil(l {})", value(pointer)).unwrap();
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
                "\t{} =l call ${symbol}(l {size}, l ${descriptor})",
                temp(*dst)
            )
            .unwrap();
        }
        ir::Inst::HeapDescriptor { dst, pointer } => {
            writeln!(
                out,
                "\t{} =l call $oberon_heap_descriptor(l {})",
                temp(*dst),
                value(pointer)
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
                "\t{} =w call $oberon_type_test_pointer(l {}, l ${target})",
                temp(*dst),
                value(pointer)
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
                "\t{} =w call $oberon_type_test_descriptor(l {}, l ${target})",
                temp(*dst),
                value(descriptor)
            )
            .unwrap();
        }
        // The check is a call, so it is textually and dynamically ahead of
        // every part of the address calculation: nothing scales or adds an
        // index the runtime has not accepted. The two intermediate temporaries
        // are named from the destination number, so they cannot collide with
        // the value temporaries, which are all `%.t<n>`.
        ir::Inst::Index {
            dst,
            base,
            index,
            len,
            stride,
            dynamic_stride,
        } => {
            writeln!(
                out,
                "\tcall $oberon_check_index(w {}, w {})",
                value(index),
                value(len)
            )
            .unwrap();
            writeln!(out, "\t%.x{dst} =l extsw {}", value(index)).unwrap();
            if dynamic_stride.is_empty() {
                writeln!(out, "\t%.s{dst} =l mul %.x{dst}, {stride}").unwrap();
                writeln!(out, "\t{} =l add {}, %.s{dst}", temp(*dst), address(base)).unwrap();
            } else {
                writeln!(out, "\t%.s{dst}.0 =l mul %.x{dst}, {stride}").unwrap();
                for (i, factor) in dynamic_stride.iter().enumerate() {
                    writeln!(out, "\t%.f{dst}.{i} =l extsw {}", value(factor)).unwrap();
                    writeln!(
                        out,
                        "\t%.s{dst}.{} =l mul %.s{dst}.{i}, %.f{dst}.{i}",
                        i + 1
                    )
                    .unwrap();
                }
                writeln!(
                    out,
                    "\t{} =l add {}, %.s{dst}.{}",
                    temp(*dst),
                    address(base),
                    dynamic_stride.len()
                )
                .unwrap();
            }
        }
        // One add, offset zero included: there is no optimization pass, and
        // one literal path is the same choice the constant index made.
        ir::Inst::Field { dst, base, offset } => {
            writeln!(out, "\t{} =l add {}, {offset}", temp(*dst), address(base)).unwrap();
        }
        ir::Inst::CopyBytes { dst, src, size } => {
            writeln!(
                out,
                "\tcall $oberon_copy(l {}, l {}, l {size})",
                address(dst),
                address(src)
            )
            .unwrap();
        }
        ir::Inst::CheckArrayCopy {
            source_len,
            destination_len,
        } => {
            writeln!(
                out,
                "\tcall $oberon_check_array_copy(w {}, w {})",
                value(source_len),
                value(destination_len)
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
            writeln!(out, "\t%.c{copy_temp} =l extsw {}", value(count)).unwrap();
            writeln!(out, "\t%.b{copy_temp} =l mul %.c{copy_temp}, {stride}").unwrap();
            writeln!(
                out,
                "\tcall $oberon_copy(l {}, l {}, l %.b{copy_temp})",
                address(dst),
                address(src)
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

// QBE passes an `s` in the platform's floating-point class and a `w` in its
// integer class, which is the native C float and int32_t ABI. A byte is a
// word in every calling position; only its memory traffic is byte-wide.
fn class(ty: ir::Ty) -> &'static str {
    match ty {
        ir::Ty::Int | ir::Ty::Bool | ir::Ty::Set | ir::Ty::Byte => "w",
        ir::Ty::Real => "s",
        ir::Ty::Pointer => "l",
    }
}

// The memory mnemonics come from the type, not from its register class: a
// byte loads zero-extended into a word and stores its low byte back.
fn load_op(ty: ir::Ty) -> &'static str {
    match ty {
        ir::Ty::Int | ir::Ty::Bool | ir::Ty::Set => "loadw",
        ir::Ty::Real => "loads",
        ir::Ty::Byte => "loadub",
        ir::Ty::Pointer => "loadl",
    }
}

fn store_op(ty: ir::Ty) -> &'static str {
    match ty {
        ir::Ty::Int | ir::Ty::Bool | ir::Ty::Set => "storew",
        ir::Ty::Real => "stores",
        ir::Ty::Byte => "storeb",
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
        ir::Value::Byte(v) => v.to_string(),
        ir::Value::Pointer(v) => v.to_string(),
        ir::Value::Symbol(symbol) => format!("${symbol}"),
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
    let pointer = ty == ir::Ty::Pointer;
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
        ir::BinOp::Eq if pointer => "ceql",
        ir::BinOp::Ne if pointer => "cnel",
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
