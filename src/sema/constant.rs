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
            ast::Expr::Bool { .. } => Some(Type::Boolean),
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
            ast::Expr::Apply { callee, args, pos } => self.check_const_call(callee, args, *pos),
            ast::Expr::Unary { op, expr, pos } => {
                let found = self.check_const_expr(expr)?;
                match (op, &found) {
                    (ast::UnOp::Plus, Type::Integer) | (ast::UnOp::Neg, Type::Integer) => {
                        Some(found)
                    }
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
                        if lhs == rhs && lhs.scalar().is_some() {
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
                        check_order_types(*pos, *op, lhs, rhs, &mut self.diags)?;
                        Some(Type::Boolean)
                    }
                }
            }
        }
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
            Ok(Symbol::Proc { ty, .. }) => ty.procedure().cloned(),
            Ok(_) => match self.designator_type(callee) {
                Ok(ty) => match ty.procedure().cloned() {
                    Some(procedure) => Some(procedure),
                    None => {
                        self.diags.push(Diagnostic::new(
                            pos,
                            format!("'{}' has type {ty} and is not a procedure", callee.name()),
                        ));
                        None
                    }
                },
                Err(diag) => {
                    self.diags.push(diag);
                    None
                }
            },
            Err(diag) => match self.designator_type(callee) {
                Ok(ty) => ty.procedure().cloned(),
                Err(_) => {
                    self.diags.push(diag);
                    None
                }
            },
        };
        let Some(procedure) = proc else {
            for actual in actuals {
                let _ = self.check_const_expr(actual);
            }
            return None;
        };
        if actuals.len() != procedure.params.len() {
            self.diags.push(Diagnostic::new(
                pos,
                format!(
                    "wrong number of arguments: expected {}, found {}",
                    procedure.params.len(),
                    actuals.len()
                ),
            ));
        }
        for (i, actual) in actuals.iter().enumerate() {
            let found = self.check_const_expr(actual);
            if let (Some(found), Some(formal)) = (found, procedure.params.get(i))
                // The same identical-type rule as the executable path for
                // reference formals, so the constant precheck cannot accept
                // what lowering rejects.
                && if formal.var {
                    found != formal.ty
                } else {
                    assign_kind(&formal.ty, &found).is_none()
                }
            {
                self.diags.push(Diagnostic::new(
                    actual.pos(),
                    format!(
                        "argument {} has type {found}, expected {}",
                        i + 1,
                        formal.ty
                    ),
                ));
            }
        }
        match procedure.ret.clone() {
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
            if !expected.contains(&found) {
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

    fn eval_const_builtin(
        &self,
        builtin: Builtin,
        callee: &ast::Designator,
        actuals: &[ast::Expr],
        pos: Pos,
    ) -> Result<ConstValue, Diagnostic> {
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
            (Builtin::Odd, [ConstValue::Int(value)]) => Ok(ConstValue::Bool(value % 2 != 0)),
            (Builtin::Ord, [ConstValue::Bool(value)]) => Ok(ConstValue::Int(i32::from(*value))),
            // Report 10.2 calls this the ordinal number of a SET but does not
            // say how 32 elements map onto a signed INTEGER. Reinterpreting
            // the bit pattern keeps the folded and runtime forms identical
            // and matches Project Oberon, where ORD emits nothing at all.
            (Builtin::Ord, [ConstValue::Set(bits)]) => Ok(ConstValue::Int(*bits as i32)),
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
            ast::Expr::Bool { value, .. } => Ok(ConstValue::Bool(*value)),
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
            ast::Expr::Apply { callee, args, pos } => match self.resolve(callee) {
                Ok(Symbol::Builtin(builtin)) => {
                    self.eval_const_builtin(builtin, callee, args, *pos)
                }
                Ok(_) | Err(_) => Err(Diagnostic::new(
                    *pos,
                    "constant expression contains a procedure call",
                )),
            },
            ast::Expr::Unary { op, expr, pos } => {
                let value = self.eval_const(expr)?;
                match (op, value) {
                    (ast::UnOp::Plus, value @ ConstValue::Int(_)) => Ok(value),
                    (ast::UnOp::Neg, ConstValue::Int(value)) => value
                        .checked_neg()
                        .map(ConstValue::Int)
                        .ok_or_else(|| Diagnostic::new(*pos, "constant expression overflows")),
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
            ast::Expr::Int { .. } | ast::Expr::Bool { .. } => true,
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
            ast::Expr::Apply { callee, args, .. } => {
                matches!(
                    self.resolve(callee),
                    Ok(Symbol::Builtin(builtin)) if builtin_signature(builtin).is_some()
                ) && args.iter().all(|arg| self.is_const_expr(arg))
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
            }
        }
        Some(ty)
    }

    // The field selector in the constant world, with the same answer as
    // Analyzer::field.
    fn const_field(&self, ty: &Type, name: &str, pos: Pos) -> Result<Type, Diagnostic> {
        Err(Diagnostic::new(
            pos,
            format!("cannot select '{name}' from {ty}: only a record has fields"),
        ))
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
