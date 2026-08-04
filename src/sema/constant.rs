use super::*;

fn eval_const_binary(
    op: ast::BinOp,
    lhs: ConstValue,
    rhs: ConstValue,
    pos: Pos,
) -> Result<ConstValue, Diagnostic> {
    use ast::BinOp;
    // Same bit rules as the runtime forms in Analyzer::lower_binary: union,
    // difference, intersection, symmetric difference.
    if let (ConstValue::Set(lhs), ConstValue::Set(rhs)) = (&lhs, &rhs)
        && let BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Slash = op
    {
        return Ok(ConstValue::Set(match op {
            BinOp::Add => lhs | rhs,
            BinOp::Sub => lhs & !rhs,
            BinOp::Mul => lhs & rhs,
            BinOp::Slash => lhs ^ rhs,
            _ => unreachable!(),
        }));
    }
    // CHAR and string relations, folded under the same bounded rule the
    // runtime comparison uses, so the folded and computed forms agree.
    if let BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge = op
        && let (Some(a), Some(b)) = (text_bytes(&lhs), text_bytes(&rhs))
    {
        return Ok(ConstValue::Bool(relation_holds(op, str_const_cmp(&a, &b))));
    }
    // Every source operator rounds at binary32, so a folded expression takes
    // the same rounding steps as the same expression computed at run time.
    // IEEE behaviour is the whole answer here: overflow yields an infinity,
    // division by zero yields an infinity or a NaN, and neither is a
    // diagnostic. A NaN is unequal to everything, itself included, and every
    // ordering comparison against one is false.
    if let (ConstValue::Real(lhs), ConstValue::Real(rhs)) = (&lhs, &rhs) {
        let (lhs, rhs) = (*lhs, *rhs);
        return Ok(match op {
            BinOp::Add => ConstValue::Real(lhs + rhs),
            BinOp::Sub => ConstValue::Real(lhs - rhs),
            BinOp::Mul => ConstValue::Real(lhs * rhs),
            BinOp::Slash => ConstValue::Real(lhs / rhs),
            BinOp::Eq => ConstValue::Bool(lhs == rhs),
            BinOp::Ne => ConstValue::Bool(lhs != rhs),
            BinOp::Lt => ConstValue::Bool(lhs < rhs),
            BinOp::Le => ConstValue::Bool(lhs <= rhs),
            BinOp::Gt => ConstValue::Bool(lhs > rhs),
            BinOp::Ge => ConstValue::Bool(lhs >= rhs),
            BinOp::Div | BinOp::Mod | BinOp::In | BinOp::And | BinOp::Or => {
                unreachable!("constant expression was type-checked")
            }
        });
    }
    match op {
        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
            let (ConstValue::Int(lhs), ConstValue::Int(rhs)) = (lhs, rhs) else {
                unreachable!("constant expression was type-checked");
            };
            if rhs == 0 && matches!(op, BinOp::Div | BinOp::Mod) {
                return Err(Diagnostic::new(pos, "constant DIV or MOD by zero"));
            }
            let value = match op {
                BinOp::Add => lhs.checked_add(rhs),
                BinOp::Sub => lhs.checked_sub(rhs),
                BinOp::Mul => lhs.checked_mul(rhs),
                // Report 8.2.2 requires floored DIV and MOD. Keep this
                // adjustment identical to the sequence floor_adjust emits.
                BinOp::Div | BinOp::Mod => {
                    let remainder = lhs.checked_rem(rhs);
                    let adjust = matches!(remainder, Some(r) if r != 0 && (r < 0) != (rhs < 0));
                    if op == BinOp::Mod {
                        if adjust {
                            remainder.and_then(|r| r.checked_add(rhs))
                        } else {
                            remainder
                        }
                    } else {
                        let quotient = lhs.checked_div(rhs);
                        if adjust {
                            quotient.and_then(|q| q.checked_sub(1))
                        } else {
                            quotient
                        }
                    }
                }
                _ => unreachable!(),
            };
            value
                .map(ConstValue::Int)
                .ok_or_else(|| Diagnostic::new(pos, "constant expression overflows"))
        }
        BinOp::Eq | BinOp::Ne => {
            assert_eq!(lhs.ty(), rhs.ty(), "constant expression was type-checked");
            let equal = lhs == rhs;
            Ok(ConstValue::Bool(if op == BinOp::Eq {
                equal
            } else {
                !equal
            }))
        }
        BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
            let (ConstValue::Int(lhs), ConstValue::Int(rhs)) = (lhs, rhs) else {
                unreachable!("constant expression was type-checked");
            };
            Ok(ConstValue::Bool(match op {
                BinOp::Lt => lhs < rhs,
                BinOp::Le => lhs <= rhs,
                BinOp::Gt => lhs > rhs,
                BinOp::Ge => lhs >= rhs,
                _ => unreachable!(),
            }))
        }
        BinOp::Slash => unreachable!("'/' on non-SET operands was rejected"),
        BinOp::In => unreachable!("membership is folded with the element check"),
        BinOp::And | BinOp::Or => unreachable!("short-circuit operators handled separately"),
    }
}

pub(super) fn floor_const(value: f32, pos: Pos) -> Result<i32, Diagnostic> {
    if !value.is_finite() || !(FLOOR_MIN..FLOOR_LIMIT).contains(&value) {
        Err(Diagnostic::new(
            pos,
            "constant FLOOR result is outside INTEGER range",
        ))
    } else {
        Ok(value.floor() as i32)
    }
}

// The bounded comparison rule of Report 8.2.4, applied to two constants. Each
// operand is its characters with the terminator appended, and the walk stops
// at the first differing pair, at a null present in both, or at the shorter
// operand's length. oberon_str_cmp applies the same rule to the same byte
// sequences, so a folded comparison and a computed one always agree.
pub(super) fn str_const_cmp(a: &[u8], b: &[u8]) -> std::cmp::Ordering {
    let bound = (a.len() + 1).min(b.len() + 1);
    for i in 0..bound {
        let x = a.get(i).copied().unwrap_or(0);
        let y = b.get(i).copied().unwrap_or(0);
        if x != y {
            return x.cmp(&y);
        }
        if x == 0 {
            break;
        }
    }
    std::cmp::Ordering::Equal
}

pub(super) fn relation_holds(op: ast::BinOp, ordering: std::cmp::Ordering) -> bool {
    use std::cmp::Ordering;
    match op {
        ast::BinOp::Eq => ordering == Ordering::Equal,
        ast::BinOp::Ne => ordering != Ordering::Equal,
        ast::BinOp::Lt => ordering == Ordering::Less,
        ast::BinOp::Le => ordering != Ordering::Greater,
        ast::BinOp::Gt => ordering == Ordering::Greater,
        ast::BinOp::Ge => ordering != Ordering::Less,
        _ => unreachable!("only a relation is folded from an ordering"),
    }
}

// The character values a constant relation can hold: a CHAR is its one
// character and a string is its characters, so one bounded rule serves every
// combination.
fn text_bytes(value: &ConstValue) -> Option<Vec<u8>> {
    match value {
        ConstValue::Char(c) => Some(vec![*c]),
        ConstValue::Str(bytes) => Some(bytes.as_ref().clone()),
        _ => None,
    }
}

impl Analyzer {
    pub(super) fn check_const_expr(&mut self, expr: &ast::Expr) -> Option<Type> {
        match expr {
            ast::Expr::Int { value, pos } => match i32::try_from(*value) {
                Ok(_) => Some(Type::Integer),
                Err(_) => {
                    self.diags
                        .push(Diagnostic::new(*pos, "integer literal out of range"));
                    None
                }
            },
            ast::Expr::Real { .. } => Some(Type::Real),
            ast::Expr::Bool { .. } => Some(Type::Boolean),
            ast::Expr::Nil { .. } => Some(Type::Nil),
            // A constant declaration keeps the string type of its right-hand
            // side, so a single-character string stays a string here and the
            // CHAR rule applies at each use site instead.
            ast::Expr::Str { bytes, .. } => Some(Type::String(bytes.len())),
            ast::Expr::Set { elements, .. } => {
                // Only the element types are checked here. The domain check
                // belongs to eval_const, which has the values.
                let mut ok = true;
                for element in elements {
                    for endpoint in [Some(&element.low), element.high.as_ref()]
                        .into_iter()
                        .flatten()
                    {
                        match self.check_const_expr(endpoint) {
                            Some(Type::Integer) => {}
                            Some(found) => {
                                self.diags.push(Diagnostic::new(
                                    endpoint.pos(),
                                    format!("set element must be INTEGER, found {found}"),
                                ));
                                ok = false;
                            }
                            None => ok = false,
                        }
                    }
                }
                ok.then_some(Type::Set)
            }
            // A variable is not a constant, but its type still has to be known
            // so the rest of the expression can be checked. eval_const is what
            // reports that it cannot be folded.
            ast::Expr::Name(designator) => self.check_const_designator_type(designator),
            ast::Expr::Apply { callee, args, pos } => {
                if self.application_is_call(callee) {
                    self.check_const_call(callee, args, *pos)
                } else {
                    self.check_const_guard(callee, args, *pos)
                }
            }
            ast::Expr::TypeTest { expr, ty, pos } => self.check_const_type_test(expr, ty, *pos),
            ast::Expr::Unary { op, expr, pos } => {
                let found = self.check_const_expr(expr)?;
                match (op, &found) {
                    (ast::UnOp::Plus, Type::Integer | Type::Real)
                    | (ast::UnOp::Neg, Type::Integer | Type::Real) => Some(found),
                    (ast::UnOp::Neg, Type::Set) => Some(Type::Set),
                    (ast::UnOp::Not, Type::Boolean) => Some(Type::Boolean),
                    _ => {
                        self.diags.push(unary_type_error(*pos, *op, &found));
                        None
                    }
                }
            }
            ast::Expr::Binary { op, lhs, rhs, pos } => {
                let lhs = self.check_const_expr(lhs);
                let rhs = self.check_const_expr(rhs);
                let (Some(lhs), Some(rhs)) = (lhs, rhs) else {
                    return None;
                };
                use ast::BinOp;
                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Slash => {
                        check_arith_types(*pos, *op, lhs, rhs, &mut self.diags)
                    }
                    BinOp::Div | BinOp::Mod => check_binary_types(
                        *pos,
                        bin_op_name(*op),
                        Type::Integer,
                        lhs,
                        rhs,
                        Type::Integer,
                        &mut self.diags,
                    ),
                    BinOp::In => {
                        if lhs == Type::Integer && rhs == Type::Set {
                            Some(Type::Boolean)
                        } else {
                            self.diags.push(Diagnostic::new(
                                *pos,
                                format!(
                                    "operator 'IN' requires INTEGER and SET, found {lhs} and {rhs}"
                                ),
                            ));
                            None
                        }
                    }
                    BinOp::And | BinOp::Or => check_binary_types(
                        *pos,
                        bin_op_name(*op),
                        Type::Boolean,
                        lhs,
                        rhs,
                        Type::Boolean,
                        &mut self.diags,
                    ),
                    BinOp::Eq | BinOp::Ne => {
                        if (lhs == rhs && lhs.scalar().is_some())
                            || pointer_value_compatible(&lhs, &rhs)
                            || text_relation_ok(&lhs, &rhs)
                        {
                            Some(Type::Boolean)
                        } else {
                            let message = if lhs == rhs {
                                format!(
                                    "operator '{}' is not defined for {lhs} operands",
                                    bin_op_name(*op)
                                )
                            } else {
                                format!(
                                    "operator '{}' requires operands of the same type, found {lhs} and {rhs}",
                                    bin_op_name(*op)
                                )
                            };
                            self.diags.push(Diagnostic::new(*pos, message));
                            None
                        }
                    }
                    BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                        if text_relation_ok(&lhs, &rhs) {
                            Some(Type::Boolean)
                        } else {
                            check_order_types(*pos, *op, lhs, rhs, &mut self.diags)?;
                            Some(Type::Boolean)
                        }
                    }
                }
            }
        }
    }

    fn check_const_type_test(
        &mut self,
        expr: &ast::Expr,
        target_name: &ast::Designator,
        pos: Pos,
    ) -> Option<Type> {
        let subject = self.check_const_expr(expr)?;
        let target = self.named_type(target_name)?;
        let valid = match (&subject, &target) {
            (Type::Nil, Type::Pointer(_)) => true,
            (Type::Pointer(_), Type::Pointer(_)) => pointer_extends(&target, &subject),
            (Type::Record(subject), Type::Record(target)) => {
                self.record_var_formal_expr(expr) && record_extends(target, subject)
            }
            _ => false,
        };
        if valid {
            Some(Type::Boolean)
        } else {
            if matches!(subject, Type::Record(_)) && !self.record_var_formal_expr(expr) {
                self.diags.push(Diagnostic::new(
                    pos,
                    "a record type test requires a record VAR parameter",
                ));
            } else {
                self.type_test_mismatch(pos, target_name.pos, &subject, &target);
            }
            None
        }
    }

    fn check_const_guard(
        &mut self,
        callee: &ast::Designator,
        args: &[ast::Expr],
        pos: Pos,
    ) -> Option<Type> {
        let subject = self.check_const_designator_type(callee)?;
        let [ast::Expr::Name(target_name)] = args else {
            for arg in args {
                let _ = self.check_const_expr(arg);
            }
            self.diags.push(Diagnostic::new(
                pos,
                "a type guard requires exactly one qualified type name",
            ));
            return None;
        };
        let target = self.named_type(target_name)?;
        let valid = match (&subject, &target) {
            (Type::Pointer(_), Type::Pointer(_)) => pointer_extends(&target, &subject),
            (Type::Record(subject), Type::Record(target)) => {
                self.record_var_formal_designator(callee) && record_extends(target, subject)
            }
            _ => false,
        };
        if valid {
            Some(target)
        } else {
            if matches!(subject, Type::Record(_)) && !self.record_var_formal_designator(callee) {
                self.diags.push(Diagnostic::new(
                    pos,
                    "a record guard requires a record VAR parameter",
                ));
            } else {
                self.type_test_mismatch(pos, target_name.pos, &subject, &target);
            }
            None
        }
    }

    fn record_var_formal_expr(&self, expr: &ast::Expr) -> bool {
        matches!(expr, ast::Expr::Name(designator) if self.record_var_formal_designator(designator))
    }

    fn record_var_formal_designator(&self, designator: &ast::Designator) -> bool {
        matches!(
            self.qualident(designator),
            Ok((
                Symbol::Var {
                    dynamic: Some(RecordDynamic::Incoming(_)),
                    ..
                },
                []
            ))
        )
    }

    fn check_const_call(
        &mut self,
        callee: &ast::Designator,
        actuals: &[ast::Expr],
        pos: Pos,
    ) -> Option<Type> {
        let proc = match self.resolve(callee) {
            Ok(Symbol::Builtin(builtin)) => {
                return self.check_const_builtin(builtin, callee, actuals, pos);
            }
            Ok(Symbol::Proc { params, ret, .. }) => Some((params, ret)),
            Ok(_) => {
                self.diags.push(Diagnostic::new(
                    pos,
                    format!("'{}' is not a procedure", callee.name()),
                ));
                None
            }
            Err(diag) => {
                self.diags.push(diag);
                None
            }
        };
        let Some((params, ret)) = proc else {
            for actual in actuals {
                let _ = self.check_const_expr(actual);
            }
            return None;
        };
        if actuals.len() != params.len() {
            self.diags.push(Diagnostic::new(
                pos,
                format!(
                    "wrong number of arguments: expected {}, found {}",
                    params.len(),
                    actuals.len()
                ),
            ));
        }
        for (i, actual) in actuals.iter().enumerate() {
            let found = self.check_const_expr(actual);
            if let (Some(found), Some((var, expected))) = (found, params.get(i))
                // The same identical-type rule as the executable path for
                // reference formals, structured value formals included, so
                // the constant precheck cannot accept what lowering rejects.
                && if *var || expected.structured() {
                    if expected.open_rank() > 0 {
                        !(open_actual_compatible(expected, &found)
                            || (!*var
                                && matches!(found, Type::String(_))
                                && open_string_formal(expected)))
                    } else if let (Some(expected), Some(found)) =
                        (expected.record(), found.record())
                    {
                        !record_extends(found, expected)
                    } else {
                        found != *expected
                    }
                } else {
                    assign_kind(expected, &found).is_none()
                }
            {
                self.diags.push(Diagnostic::new(
                    actual.pos(),
                    format!("argument {} has type {found}, expected {expected}", i + 1),
                ));
            }
        }
        match ret {
            // A BYTE result is read as an INTEGER at the call, just as it is
            // in executable lowering. Keeping the constant precheck in step
            // prevents a valid surrounding INTEGER expression from gaining
            // a second, false type diagnostic before the call is rejected as
            // nonconstant.
            Some(Type::Byte) => Some(Type::Integer),
            Some(ty) => Some(ty),
            None => {
                self.diags.push(Diagnostic::new(
                    pos,
                    format!("'{}' cannot be used as a value", callee.name()),
                ));
                None
            }
        }
    }

    fn check_const_builtin(
        &mut self,
        builtin: Builtin,
        callee: &ast::Designator,
        actuals: &[ast::Expr],
        pos: Pos,
    ) -> Option<Type> {
        if builtin == Builtin::Len {
            return self.check_const_len(actuals, pos);
        }
        let Some((params, result)) = builtin_signature(builtin) else {
            for actual in actuals {
                let _ = self.check_const_expr(actual);
            }
            self.diags.push(Diagnostic::new(
                pos,
                format!("'{}' cannot be used as a value", callee.name()),
            ));
            return None;
        };
        if actuals.len() != params.len() {
            self.diags.push(Diagnostic::new(
                pos,
                format!(
                    "wrong number of arguments: expected {}, found {}",
                    params.len(),
                    actuals.len()
                ),
            ));
            for actual in actuals {
                let _ = self.check_const_expr(actual);
            }
            // Unlike a user procedure, stop here: eval_const would otherwise
            // fold the same call and report the same arity twice.
            return None;
        }
        let mut first = None;
        let mut ok = true;
        for (i, (actual, expected)) in actuals.iter().zip(params).enumerate() {
            let Some(found) = self.check_const_expr(actual) else {
                ok = false;
                continue;
            };
            // The single-character rule applies in the constant world too,
            // so ORD("A") folds exactly as ORD of a CHAR constant does.
            let char_from_string = expected.contains(&Type::Char) && found == Type::String(1);
            if !expected.contains(&found) && !char_from_string {
                self.diags.push(Diagnostic::new(
                    actual.pos(),
                    format!(
                        "argument {} has type {found}, expected {}",
                        i + 1,
                        type_list(expected)
                    ),
                ));
                ok = false;
            }
            if i == 0 {
                first = Some(found);
            }
        }
        ok.then(|| result.ty(first.expect("a function-like builtin takes an argument")))
    }

    // LEN in a required constant context, such as a constant declaration or
    // another array's length. Its argument is a designator rather than a
    // constant expression, so it is checked here instead of through the
    // signature table.
    fn check_const_len(&mut self, actuals: &[ast::Expr], pos: Pos) -> Option<Type> {
        if actuals.len() != 1 {
            self.diags.push(Diagnostic::new(
                pos,
                format!(
                    "wrong number of arguments: expected 1, found {}",
                    actuals.len()
                ),
            ));
            return None;
        }
        let ast::Expr::Name(designator) = &actuals[0] else {
            self.diags.push(Diagnostic::new(
                actuals[0].pos(),
                "argument 1 must be an array variable",
            ));
            return None;
        };
        let ty = self.check_len_designator_type(designator)?;
        if ty.is_array() {
            Some(Type::Integer)
        } else {
            self.diags.push(Diagnostic::new(
                designator.pos,
                format!("argument 1 has type {ty}, expected an array"),
            ));
            None
        }
    }

    fn eval_const_builtin(
        &self,
        builtin: Builtin,
        callee: &ast::Designator,
        actuals: &[ast::Expr],
        pos: Pos,
    ) -> Result<ConstValue, Diagnostic> {
        if builtin == Builtin::Len {
            let [ast::Expr::Name(designator)] = actuals else {
                return Err(Diagnostic::new(pos, "argument 1 must be an array variable"));
            };
            return Ok(ConstValue::Int(self.const_array_type(designator)?.len));
        }
        if builtin_signature(builtin).is_none() {
            return Err(Diagnostic::new(
                pos,
                format!("'{}' cannot be used as a value", callee.name()),
            ));
        }
        let args = actuals
            .iter()
            .map(|actual| self.eval_const(actual))
            .collect::<Result<Vec<_>, _>>()?;
        match (builtin, args.as_slice()) {
            (Builtin::Abs, [ConstValue::Int(value)]) => value
                .checked_abs()
                .map(ConstValue::Int)
                .ok_or_else(|| Diagnostic::new(pos, "constant expression overflows")),
            // Clears the sign of a negative zero, leaves an infinity alone,
            // and returns a NaN for a NaN, exactly as fabsf does at run time.
            (Builtin::Abs, [ConstValue::Real(value)]) => Ok(ConstValue::Real(value.abs())),
            (Builtin::Odd, [ConstValue::Int(value)]) => Ok(ConstValue::Bool(value % 2 != 0)),
            // The rounding is what makes FLT(MAX(INTEGER)) equal 2147483648.0
            // and therefore outside the FLOOR domain below.
            (Builtin::Flt, [ConstValue::Int(value)]) => Ok(ConstValue::Real(*value as f32)),
            (Builtin::Floor, [ConstValue::Real(value)]) => {
                floor_const(*value, actuals[0].pos()).map(ConstValue::Int)
            }
            (Builtin::Ord, [ConstValue::Bool(value)]) => Ok(ConstValue::Int(i32::from(*value))),
            // Report 10.2 calls this the ordinal number of a SET but does not
            // say how 32 elements map onto a signed INTEGER. Reinterpreting
            // the bit pattern keeps the folded and runtime forms identical
            // and matches Project Oberon, where ORD emits nothing at all.
            (Builtin::Ord, [ConstValue::Set(bits)]) => Ok(ConstValue::Int(*bits as i32)),
            (Builtin::Ord, [ConstValue::Char(c)]) => Ok(ConstValue::Int(i32::from(*c))),
            // The single-character rule again: ORD("A") is the ordinal of
            // that one character. A longer string keeps a diagnostic rather
            // than an internal invariant, because an unchecked constant walk
            // can reach here through a folded LEN index.
            (Builtin::Ord, [ConstValue::Str(bytes)]) if bytes.len() == 1 => {
                Ok(ConstValue::Int(i32::from(bytes[0])))
            }
            (Builtin::Ord, [ConstValue::Str(bytes)]) => Err(Diagnostic::new(
                actuals[0].pos(),
                format!(
                    "argument 1 has type {}, expected CHAR or BOOLEAN or SET",
                    Type::String(bytes.len())
                ),
            )),
            (Builtin::Chr, [ConstValue::Int(value)]) => {
                if (0..=255).contains(value) {
                    Ok(ConstValue::Char(*value as u8))
                } else {
                    Err(Diagnostic::new(
                        actuals[0].pos(),
                        format!("CHR argument {value} is out of range: must be between 0 and 255"),
                    ))
                }
            }
            (
                Builtin::Lsl | Builtin::Asr | Builtin::Ror,
                [ConstValue::Int(x), ConstValue::Int(n)],
            ) => {
                let count = u32::try_from(*n)
                    .ok()
                    .filter(|n| *n < 32)
                    .ok_or_else(|| shift_range_error(actuals[1].pos(), *n))?;
                Ok(ConstValue::Int(match builtin {
                    // A logical shift: bits past the top are discarded, so
                    // LSL(1, 31) is MIN(INTEGER) and not an overflow, even
                    // though the Report's gloss reads "x * 2^n".
                    Builtin::Lsl => ((*x as u32) << count) as i32,
                    Builtin::Asr => x >> count,
                    Builtin::Ror => (*x as u32).rotate_right(count) as i32,
                    _ => unreachable!("not a shift"),
                }))
            }
            _ => unreachable!("builtin call was type-checked"),
        }
    }

    pub(super) fn eval_const(&self, expr: &ast::Expr) -> Result<ConstValue, Diagnostic> {
        match expr {
            ast::Expr::Int { value, pos } => i32::try_from(*value)
                .map(ConstValue::Int)
                .map_err(|_| Diagnostic::new(*pos, "integer literal out of range")),
            ast::Expr::Real { value, .. } => Ok(ConstValue::Real(*value)),
            ast::Expr::Bool { value, .. } => Ok(ConstValue::Bool(*value)),
            ast::Expr::Nil { .. } => Ok(ConstValue::Nil),
            ast::Expr::Str { bytes, .. } => Ok(ConstValue::Str(Rc::new(bytes.clone()))),
            ast::Expr::Set { elements, .. } => {
                let mut bits = 0;
                for element in elements {
                    let low = self.const_set_element(&element.low)?;
                    bits |= match &element.high {
                        // Both endpoints are checked before the reversed-range
                        // rule applies, so {32 .. 0} is an error and not empty.
                        Some(high) => set_range_bits(low, self.const_set_element(high)?),
                        None => 1 << low,
                    };
                }
                Ok(ConstValue::Set(bits))
            }
            ast::Expr::Name(designator) => match self.qualident(designator)? {
                (Symbol::Const(value), []) => Ok(value),
                _ => Err(Diagnostic::new(
                    designator.pos,
                    format!("'{}' is not a constant", designator.name()),
                )),
            },
            ast::Expr::Apply { callee, args, pos } => {
                if !self.application_is_call(callee) {
                    return Err(Diagnostic::new(
                        *pos,
                        "constant expression contains a type guard",
                    ));
                }
                match self.resolve(callee)? {
                    Symbol::Builtin(builtin) => {
                        self.eval_const_builtin(builtin, callee, args, *pos)
                    }
                    _ => Err(Diagnostic::new(
                        *pos,
                        "constant expression contains a procedure call",
                    )),
                }
            }
            ast::Expr::TypeTest { expr, ty, pos } => {
                let target = match self.resolve(ty)? {
                    Symbol::TypeName(target) => target,
                    _ => {
                        return Err(Diagnostic::new(
                            ty.pos,
                            format!("'{}' is not a type", ty.name()),
                        ));
                    }
                };
                match (self.eval_const(expr)?, target) {
                    (ConstValue::Nil, Type::Pointer(_)) => Ok(ConstValue::Bool(false)),
                    _ => Err(Diagnostic::new(
                        *pos,
                        "type test is not a constant operation",
                    )),
                }
            }
            ast::Expr::Unary { op, expr, pos } => {
                let value = self.eval_const(expr)?;
                match (op, value) {
                    (ast::UnOp::Plus, value @ (ConstValue::Int(_) | ConstValue::Real(_))) => {
                        Ok(value)
                    }
                    (ast::UnOp::Neg, ConstValue::Int(value)) => value
                        .checked_neg()
                        .map(ConstValue::Int)
                        .ok_or_else(|| Diagnostic::new(*pos, "constant expression overflows")),
                    // Flipping the binary32 sign, so -0.0 is a value the
                    // source can write and the emitted immediate preserves.
                    (ast::UnOp::Neg, ConstValue::Real(value)) => Ok(ConstValue::Real(-value)),
                    (ast::UnOp::Neg, ConstValue::Set(bits)) => Ok(ConstValue::Set(bits ^ SET_FULL)),
                    (ast::UnOp::Not, ConstValue::Bool(value)) => Ok(ConstValue::Bool(!value)),
                    _ => unreachable!("constant expression was type-checked"),
                }
            }
            ast::Expr::Binary {
                op: ast::BinOp::And,
                lhs,
                rhs,
                ..
            } => match self.eval_const(lhs)? {
                ConstValue::Bool(false) => Ok(ConstValue::Bool(false)),
                ConstValue::Bool(true) => self.eval_const(rhs),
                _ => unreachable!("constant expression was type-checked"),
            },
            ast::Expr::Binary {
                op: ast::BinOp::Or,
                lhs,
                rhs,
                ..
            } => match self.eval_const(lhs)? {
                ConstValue::Bool(true) => Ok(ConstValue::Bool(true)),
                ConstValue::Bool(false) => self.eval_const(rhs),
                _ => unreachable!("constant expression was type-checked"),
            },
            ast::Expr::Binary {
                op: ast::BinOp::In,
                lhs,
                rhs,
                ..
            } => {
                let element = self.const_set_element(lhs)?;
                let ConstValue::Set(bits) = self.eval_const(rhs)? else {
                    unreachable!("constant expression was type-checked");
                };
                Ok(ConstValue::Bool(bits & (1 << element) != 0))
            }
            ast::Expr::Binary { op, lhs, rhs, pos } => {
                let lhs = self.eval_const(lhs)?;
                let rhs = self.eval_const(rhs)?;
                eval_const_binary(*op, lhs, rhs, *pos)
            }
        }
    }

    // Optional constant evaluation has three outcomes. A value can be folded,
    // an expression containing a variable or user procedure must run, and an
    // invalid expression made entirely from constants keeps its diagnostic.
    pub(super) fn try_eval_const(
        &self,
        expr: &ast::Expr,
    ) -> Result<Option<ConstValue>, Diagnostic> {
        match self.eval_const(expr) {
            Ok(value) => Ok(Some(value)),
            Err(diag) if self.is_const_expr(expr) => Err(diag),
            Err(_) => Ok(None),
        }
    }

    fn is_const_expr(&self, expr: &ast::Expr) -> bool {
        match expr {
            ast::Expr::Int { .. }
            | ast::Expr::Real { .. }
            | ast::Expr::Bool { .. }
            | ast::Expr::Nil { .. }
            | ast::Expr::Str { .. } => true,
            ast::Expr::Set { elements, .. } => elements.iter().all(|element| {
                self.is_const_expr(&element.low)
                    && element
                        .high
                        .as_ref()
                        .is_none_or(|high| self.is_const_expr(high))
            }),
            ast::Expr::Name(designator) => {
                matches!(self.resolve(designator), Ok(Symbol::Const(_)))
            }
            // LEN is a constant when it can see the length without running
            // anything: the argument names an array variable and every
            // selector on it is itself constant. A dynamic selector makes the
            // call nonconstant even though its result is statically known.
            ast::Expr::Apply { callee, args, .. }
                if matches!(self.resolve(callee), Ok(Symbol::Builtin(Builtin::Len))) =>
            {
                let [ast::Expr::Name(designator)] = args.as_slice() else {
                    return false;
                };
                matches!(self.len_designator_type(designator), Ok(ty) if ty.array().is_some())
            }
            ast::Expr::Apply { callee, args, .. } => {
                matches!(
                    self.resolve(callee),
                    Ok(Symbol::Builtin(builtin)) if builtin_signature(builtin).is_some()
                ) && args.iter().all(|arg| self.is_const_expr(arg))
            }
            ast::Expr::TypeTest { expr, .. } => {
                matches!(self.eval_const(expr), Ok(ConstValue::Nil))
            }
            ast::Expr::Unary { expr, .. } => self.is_const_expr(expr),
            ast::Expr::Binary { lhs, rhs, .. } => {
                self.is_const_expr(lhs) && self.is_const_expr(rhs)
            }
        }
    }

    // Type-checks a designator in the constant world before folding any
    // selector. eval_const relies on that ordering and treats an impossible
    // operand combination as an internal invariant, just as it does for every
    // other constant expression checked through check_const_expr.
    fn check_const_designator_type(&mut self, designator: &ast::Designator) -> Option<Type> {
        let (symbol, rest) = match self.qualident(designator) {
            Ok(found) => found,
            Err(diag) => {
                self.diags.push(diag);
                return None;
            }
        };
        let mut ty = match symbol {
            Symbol::Const(value) if rest.is_empty() => return Some(value.ty()),
            Symbol::Var { ty, .. } => ty,
            _ => {
                self.diags.push(Diagnostic::new(
                    designator.pos,
                    format!("'{}' cannot be used as a value", designator.name()),
                ));
                return None;
            }
        };
        for selector in rest {
            match selector {
                ast::Selector::Field(name, pos) => match self.const_field(&ty, name, *pos) {
                    Ok(field) => ty = field,
                    Err(diag) => {
                        self.diags.push(diag);
                        return None;
                    }
                },
                ast::Selector::Index(exprs, pos) => {
                    for expr in exprs {
                        let Some(elem) = ty.array_elem().cloned() else {
                            self.diags.push(Diagnostic::new(
                                *pos,
                                format!("cannot index {ty}: only an array can be indexed"),
                            ));
                            return None;
                        };
                        match self.check_const_expr(expr) {
                            Some(Type::Integer) => {}
                            Some(found) => {
                                self.diags.push(Diagnostic::new(
                                    expr.pos(),
                                    format!("array index must be INTEGER, found {found}"),
                                ));
                                return None;
                            }
                            None => return None,
                        }
                        let value = match self.eval_const(expr) {
                            Ok(ConstValue::Int(value)) => value,
                            Ok(_) => unreachable!("array index was type-checked as INTEGER"),
                            Err(diag) => {
                                self.diags.push(diag);
                                return None;
                            }
                        };
                        if let Some(array) = ty.array()
                            && !(0..array.len).contains(&value)
                        {
                            self.diags
                                .push(index_range_error(expr.pos(), value, array.len));
                            return None;
                        }
                        ty = elem;
                    }
                }
                ast::Selector::Deref(pos) => match self.const_dereference(&ty, *pos) {
                    Ok(base) => ty = base,
                    Err(diag) => {
                        self.diags.push(diag);
                        return None;
                    }
                },
                ast::Selector::Guard(_, pos) => {
                    self.diags.push(Diagnostic::new(
                        *pos,
                        "constant expression contains a type guard",
                    ));
                    return None;
                }
            }
        }
        Some(ty)
    }

    // Fixed LEN in a required constant context needs only the selected
    // array's declared type. Selectors are still type-checked, and a constant
    // index is still checked against its bound, but a dynamic INTEGER index
    // performs no work and does not prevent folding the length. A call in an
    // index is the one dynamic form this path refuses; see const_index_call.
    fn check_len_designator_type(&mut self, designator: &ast::Designator) -> Option<Type> {
        let (symbol, rest) = match self.qualident(designator) {
            Ok(found) => found,
            Err(diag) => {
                self.diags.push(diag);
                return None;
            }
        };
        let mut ty = match symbol {
            Symbol::Var { ty, .. } => ty,
            _ => {
                self.diags.push(Diagnostic::new(
                    designator.pos,
                    "argument 1 must be an array variable",
                ));
                return None;
            }
        };
        for selector in rest {
            match selector {
                ast::Selector::Field(name, pos) => match self.const_field(&ty, name, *pos) {
                    Ok(field) => ty = field,
                    Err(diag) => {
                        self.diags.push(diag);
                        return None;
                    }
                },
                ast::Selector::Deref(pos) => match self.const_dereference(&ty, *pos) {
                    Ok(base) => ty = base,
                    Err(diag) => {
                        self.diags.push(diag);
                        return None;
                    }
                },
                ast::Selector::Index(exprs, pos) => {
                    for expr in exprs {
                        let Some(elem) = ty.array_elem().cloned() else {
                            self.diags.push(Diagnostic::new(
                                *pos,
                                format!("cannot index {ty}: only an array can be indexed"),
                            ));
                            return None;
                        };
                        if let Some(diag) = self.const_index_call(expr) {
                            self.diags.push(diag);
                            return None;
                        }
                        match self.check_const_expr(expr) {
                            Some(Type::Integer) => {}
                            Some(found) => {
                                self.diags.push(Diagnostic::new(
                                    expr.pos(),
                                    format!("array index must be INTEGER, found {found}"),
                                ));
                                return None;
                            }
                            None => return None,
                        }
                        match self.try_eval_const(expr) {
                            Ok(Some(ConstValue::Int(value))) => {
                                if let Some(array) = ty.array()
                                    && !(0..array.len).contains(&value)
                                {
                                    self.diags.push(index_range_error(
                                        expr.pos(),
                                        value,
                                        array.len,
                                    ));
                                    return None;
                                }
                            }
                            Ok(None) => {}
                            Ok(Some(_)) => unreachable!("index was type-checked as INTEGER"),
                            Err(diag) => {
                                self.diags.push(diag);
                                return None;
                            }
                        }
                        ty = elem;
                    }
                }
                ast::Selector::Guard(_, pos) => {
                    self.diags.push(Diagnostic::new(
                        *pos,
                        "constant expression contains a type guard",
                    ));
                    return None;
                }
            }
        }
        Some(ty)
    }

    // The same walk without diagnostics, for deciding whether a LEN call is
    // constant and for folding it. It has to reach the same verdict as
    // check_len_designator_type on every program, or a constant declaration
    // would be checked under one rule and evaluated under another.
    fn len_designator_type(&self, designator: &ast::Designator) -> Result<Type, Diagnostic> {
        let (symbol, rest) = self.qualident(designator)?;
        let mut ty = match symbol {
            Symbol::Var { ty, .. } => ty,
            _ => {
                return Err(Diagnostic::new(
                    designator.pos,
                    "argument 1 must be an array variable",
                ));
            }
        };
        for selector in rest {
            match selector {
                ast::Selector::Field(name, pos) => ty = self.const_field(&ty, name, *pos)?,
                ast::Selector::Deref(pos) => ty = self.const_dereference(&ty, *pos)?,
                ast::Selector::Index(exprs, pos) => {
                    for expr in exprs {
                        let Some(elem) = ty.array_elem() else {
                            return Err(Diagnostic::new(
                                *pos,
                                format!("cannot index {ty}: only an array can be indexed"),
                            ));
                        };
                        if let Some(diag) = self.const_index_call(expr) {
                            return Err(diag);
                        }
                        ty = elem.clone();
                    }
                }
                ast::Selector::Guard(_, pos) => {
                    return Err(Diagnostic::new(
                        *pos,
                        "constant expression contains a type guard",
                    ));
                }
            }
        }
        Ok(ty)
    }

    // The field selector in the constant world: the same lookup and the same
    // visibility rule as Analyzer::field, with no address to compute. Both
    // constant walks use it, so `LEN` of an array field folds exactly as `LEN`
    // of an array variable does.
    fn const_field(&self, ty: &Type, name: &str, pos: Pos) -> Result<Type, Diagnostic> {
        let base = if ty.pointer().is_some() {
            self.const_dereference(ty, pos)?
        } else {
            ty.clone()
        };
        let Some(record) = base.record() else {
            return Err(Diagnostic::new(
                pos,
                format!("cannot select '{name}' from {base}: only a record has fields"),
            ));
        };
        find_field(record, name, &self.module)
            .map(|field| field.ty.clone())
            .ok_or_else(|| no_such_field(pos, name, &base))
    }

    // Report 8: a constant expression is one a mere textual scan can evaluate
    // without executing the program. The required-constant LEN walk answers
    // from the declared array type and never evaluates an index, so a call
    // written in one would be discarded rather than performed. It is rejected
    // instead, with the same message a call anywhere else in a constant
    // expression already gets. A variable index stays legal: skipping a read
    // is unobservable, and skipping a call is not.
    fn const_index_call(&self, expr: &ast::Expr) -> Option<Diagnostic> {
        match expr {
            ast::Expr::Int { .. }
            | ast::Expr::Real { .. }
            | ast::Expr::Bool { .. }
            | ast::Expr::Nil { .. }
            | ast::Expr::Str { .. } => None,
            ast::Expr::Set { elements, .. } => elements.iter().find_map(|element| {
                self.const_index_call(&element.low).or_else(|| {
                    element
                        .high
                        .as_ref()
                        .and_then(|high| self.const_index_call(high))
                })
            }),
            // A designator carries index selectors of its own, and LEN of one
            // array can be the index into another.
            ast::Expr::Name(designator) => self.const_selector_call(designator),
            ast::Expr::Apply { callee, args, pos } => {
                if !matches!(self.resolve(callee), Ok(Symbol::Builtin(_))) {
                    return Some(Diagnostic::new(
                        *pos,
                        "constant expression contains a procedure call",
                    ));
                }
                args.iter().find_map(|arg| self.const_index_call(arg))
            }
            ast::Expr::TypeTest { expr, .. } => self.const_index_call(expr),
            ast::Expr::Unary { expr, .. } => self.const_index_call(expr),
            ast::Expr::Binary { lhs, rhs, .. } => self
                .const_index_call(lhs)
                .or_else(|| self.const_index_call(rhs)),
        }
    }

    fn const_selector_call(&self, designator: &ast::Designator) -> Option<Diagnostic> {
        designator
            .selectors
            .iter()
            .find_map(|selector| match selector {
                ast::Selector::Index(exprs, _) => {
                    exprs.iter().find_map(|expr| self.const_index_call(expr))
                }
                ast::Selector::Field(..) | ast::Selector::Deref(_) => None,
                ast::Selector::Guard(_, pos) => Some(Diagnostic::new(
                    *pos,
                    "constant expression contains a type guard",
                )),
            })
    }

    fn const_dereference(&self, ty: &Type, pos: Pos) -> Result<Type, Diagnostic> {
        let Some(pointer) = ty.pointer() else {
            return Err(Diagnostic::new(
                pos,
                format!("cannot dereference {ty}: only a pointer can be dereferenced"),
            ));
        };
        pointer
            .record()
            .map(Type::Record)
            .ok_or_else(|| Diagnostic::new(pos, "pointer has an invalid base type"))
    }

    // The array a folded LEN is about. The variable itself need not be a
    // constant: a fixed length is a property of its type.
    fn const_array_type(&self, designator: &ast::Designator) -> Result<Rc<ArrayType>, Diagnostic> {
        let ty = self.len_designator_type(designator)?;
        if matches!(ty, Type::OpenArray(_)) {
            return Err(Diagnostic::new(
                designator.pos,
                "LEN of an open array is not a constant",
            ));
        }
        ty.array().cloned().ok_or_else(|| {
            Diagnostic::new(
                designator.pos,
                format!("argument 1 has type {ty}, expected an array"),
            )
        })
    }

    // The static half of the one element-domain rule; Analyzer::
    // check_set_element is the runtime half, and the two must stay in step.
    fn const_set_element(&self, expr: &ast::Expr) -> Result<i32, Diagnostic> {
        let ConstValue::Int(element) = self.eval_const(expr)? else {
            unreachable!("constant expression was type-checked");
        };
        if (0..=SET_MAX).contains(&element) {
            Ok(element)
        } else {
            Err(set_element_range_error(expr.pos(), element))
        }
    }
}
