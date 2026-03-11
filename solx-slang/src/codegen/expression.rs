//!
//! Expression lowering to MLIR SSA values.
//!

use melior::dialect::llvm;
use melior::ir::Block;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Identifier;
use melior::ir::Region;
use melior::ir::RegionLike;
use melior::ir::Value;
use melior::ir::operation::OperationBuilder;

use slang_solidity::backend::ir::ir2_flat_contracts::ArgumentsDeclaration;
use slang_solidity::backend::ir::ir2_flat_contracts::Expression;

use solx_mlir::Environment;
use solx_mlir::ICmpPredicate;
use solx_mlir::ops;
use solx_utils::AddressSpace;

use crate::codegen::MlirContext;

/// Lowers Solidity expressions to MLIR SSA values.
pub struct ExpressionEmitter<'a, 'c, 'b> {
    /// The shared MLIR context.
    state: &'a MlirContext<'c>,
    /// Variable environment.
    env: &'a Environment<'c, 'b>,
    /// The function region for creating new blocks.
    region: &'a Region<'c>,
}

impl<'a, 'c, 'b> ExpressionEmitter<'a, 'c, 'b> {
    /// Creates a new expression emitter.
    pub fn new(
        state: &'a MlirContext<'c>,
        env: &'a Environment<'c, 'b>,
        region: &'a Region<'c>,
    ) -> Self {
        Self { state, env, region }
    }

    /// Emits MLIR for an expression, appending operations to `block`.
    ///
    /// Returns the SSA value produced and the continuation block (which may
    /// differ from the input block for short-circuit operators).
    pub fn emit(
        &self,
        expr: &Expression,
        block: BlockRef<'c, 'b>,
    ) -> anyhow::Result<(Value<'c, 'b>, BlockRef<'c, 'b>)> {
        match expr {
            Expression::DecimalNumberExpression(decimal) => {
                let text = decimal.literal.text.as_str();
                let v = if let Ok(value) = text.parse::<i64>() {
                    self.emit_i256_constant(value, &block)
                } else {
                    self.state.emit_i256_from_decimal_str(text, &block)?
                };
                Ok((v, block))
            }
            Expression::HexNumberExpression(hex) => {
                let text = hex.literal.text.as_str();
                let stripped = text
                    .strip_prefix("0x")
                    .or(text.strip_prefix("0X"))
                    .unwrap_or(text);
                let v = if let Ok(value) = i64::from_str_radix(stripped, 16) {
                    self.emit_i256_constant(value, &block)
                } else {
                    self.state.emit_i256_from_hex_str(stripped, &block)?
                };
                Ok((v, block))
            }
            Expression::TrueKeyword => {
                let v = self.emit_i256_constant(1, &block);
                Ok((v, block))
            }
            Expression::FalseKeyword => {
                let v = self.emit_i256_constant(0, &block);
                Ok((v, block))
            }
            Expression::Identifier(terminal) => {
                let name = terminal.text.as_str();
                if let Some(ptr) = self.env.get_variable(name) {
                    let v = self.emit_load(ptr, &block)?;
                    Ok((v, block))
                } else if let Some(slot) = self.state.state_variable_slot(name) {
                    let v = self.emit_storage_load(slot, &block)?;
                    Ok((v, block))
                } else {
                    anyhow::bail!("undefined variable: {name}")
                }
            }
            Expression::AssignmentExpression(assign) => {
                self.emit_assignment(assign, block)
            }
            Expression::AdditiveExpression(expr) => {
                self.emit_binary_op(&expr.left_operand, &expr.right_operand, &expr.operator.text, block)
            }
            Expression::MultiplicativeExpression(expr) => {
                self.emit_binary_op(&expr.left_operand, &expr.right_operand, &expr.operator.text, block)
            }
            Expression::EqualityExpression(expr) => {
                self.emit_icmp(&expr.left_operand, &expr.right_operand, &expr.operator.text, block)
            }
            Expression::InequalityExpression(expr) => {
                self.emit_icmp(&expr.left_operand, &expr.right_operand, &expr.operator.text, block)
            }
            Expression::AndExpression(expr) => {
                self.emit_and(&expr.left_operand, &expr.right_operand, block)
            }
            Expression::OrExpression(expr) => {
                self.emit_or(&expr.left_operand, &expr.right_operand, block)
            }
            Expression::PostfixExpression(expr) => {
                self.emit_postfix(&expr.operand, &expr.operator.text, block)
            }
            Expression::PrefixExpression(expr) => {
                self.emit_prefix(&expr.operator.text, &expr.operand, block)
            }
            Expression::BitwiseAndExpression(expr) => {
                let (lhs, block) = self.emit(&expr.left_operand, block)?;
                let (rhs, block) = self.emit(&expr.right_operand, block)?;
                let v = self.emit_llvm_op(ops::AND, lhs, rhs, &block);
                Ok((v, block))
            }
            Expression::BitwiseOrExpression(expr) => {
                let (lhs, block) = self.emit(&expr.left_operand, block)?;
                let (rhs, block) = self.emit(&expr.right_operand, block)?;
                let v = self.emit_llvm_op(ops::OR, lhs, rhs, &block);
                Ok((v, block))
            }
            Expression::BitwiseXorExpression(expr) => {
                let (lhs, block) = self.emit(&expr.left_operand, block)?;
                let (rhs, block) = self.emit(&expr.right_operand, block)?;
                let v = self.emit_llvm_op(ops::XOR, lhs, rhs, &block);
                Ok((v, block))
            }
            Expression::ShiftExpression(expr) => {
                let (lhs, block) = self.emit(&expr.left_operand, block)?;
                let (rhs, block) = self.emit(&expr.right_operand, block)?;
                let op = match expr.operator.text.as_str() {
                    "<<" => ops::SHL,
                    ">>" => ops::LSHR,
                    _ => anyhow::bail!("unsupported shift operator: {}", expr.operator.text),
                };
                let v = self.emit_llvm_op(op, lhs, rhs, &block);
                Ok((v, block))
            }
            Expression::FunctionCallExpression(call) => {
                self.emit_function_call(&call.operand, &call.arguments, block)
            }
            Expression::MemberAccessExpression(access) => {
                self.emit_member_access(&access.operand, access.member.text.as_str(), block)
            }
            _ => anyhow::bail!("unsupported expression: {expr:?}"),
        }
    }

    /// Emits an assignment expression (`=`, `+=`, `-=`, `*=`).
    fn emit_assignment(
        &self,
        assign: &slang_solidity::backend::ir::ir2_flat_contracts::AssignmentExpression,
        block: BlockRef<'c, 'b>,
    ) -> anyhow::Result<(Value<'c, 'b>, BlockRef<'c, 'b>)> {
        let Expression::Identifier(terminal) = &assign.left_operand else {
            anyhow::bail!("unsupported assignment target: {:?}", assign.left_operand);
        };
        let name = terminal.text.as_str();

        // Determine whether this is a local variable or a state variable.
        let local_ptr = self.env.get_variable(name);
        let storage_slot = self.state.state_variable_slot(name);
        if local_ptr.is_none() && storage_slot.is_none() {
            anyhow::bail!("undefined variable: {name}");
        }

        let op = assign.operator.text.as_str();
        let (value, block) = if op == "=" {
            self.emit(&assign.right_operand, block)?
        } else {
            let old = if let Some(ptr) = local_ptr {
                self.emit_load(ptr, &block)?
            } else {
                self.emit_storage_load(storage_slot.unwrap(), &block)?
            };
            let (rhs, block) = self.emit(&assign.right_operand, block)?;
            let arith_op = match op {
                "+=" => ops::ADD,
                "-=" => ops::SUB,
                "*=" => ops::MUL,
                "/=" => ops::UDIV,
                "%=" => ops::UREM,
                _ => anyhow::bail!("unsupported assignment operator: {op}"),
            };
            let result = self.emit_llvm_op(arith_op, old, rhs, &block);
            (result, block)
        };

        if let Some(ptr) = local_ptr {
            self.emit_store(value, ptr, &block)?;
        } else {
            self.emit_storage_store(storage_slot.unwrap(), value, &block)?;
        }
        Ok((value, block))
    }

    /// Emits an `llvm.mlir.constant` producing an `i256` value.
    pub fn emit_i256_constant(
        &self,
        value: i64,
        block: &BlockRef<'c, 'b>,
    ) -> Value<'c, 'b> {
        self.state.emit_i256_constant(value, block)
    }

    /// Emits an `llvm.load` from a pointer.
    fn emit_load(
        &self,
        ptr: Value<'c, 'b>,
        block: &BlockRef<'c, 'b>,
    ) -> anyhow::Result<Value<'c, 'b>> {
        self.state.emit_load(ptr, self.state.i256(), block)
    }

    /// Emits an `llvm.store` to a pointer.
    pub fn emit_store(
        &self,
        value: Value<'c, 'b>,
        ptr: Value<'c, 'b>,
        block: &BlockRef<'c, 'b>,
    ) -> anyhow::Result<()> {
        self.state.emit_store(value, ptr, block)
    }

    /// Emits a storage load (`inttoptr` slot to `ptr addrspace(5)`, then `llvm.load`).
    fn emit_storage_load(
        &self,
        slot: u64,
        block: &BlockRef<'c, 'b>,
    ) -> anyhow::Result<Value<'c, 'b>> {
        let i256 = self.state.i256();
        let storage_ptr_type = self.state.ptr(AddressSpace::Storage);
        let slot_val = self.state.emit_i256_from_u64(slot, block);
        let ptr = self.state.emit_inttoptr(slot_val, storage_ptr_type, block);
        self.state.emit_load(ptr, i256, block)
    }

    /// Emits a storage store (`inttoptr` slot to `ptr addrspace(5)`, then `llvm.store`).
    fn emit_storage_store(
        &self,
        slot: u64,
        value: Value<'c, 'b>,
        block: &BlockRef<'c, 'b>,
    ) -> anyhow::Result<()> {
        let storage_ptr_type = self.state.ptr(AddressSpace::Storage);
        let slot_val = self.state.emit_i256_from_u64(slot, block);
        let ptr = self.state.emit_inttoptr(slot_val, storage_ptr_type, block);
        self.state.emit_store(value, ptr, block)
    }

    /// Emits an `llvm.alloca` for a local variable, returning the pointer.
    pub fn emit_alloca(&self, block: &BlockRef<'c, 'b>) -> Value<'c, 'b> {
        let i256 = self.state.i256();
        let location = self.state.location();
        let context = self.state.context();
        let ptr_type = self.state.ptr(AddressSpace::Stack);

        let one = self.emit_i256_constant(1, block);
        block
            .append_operation(
                OperationBuilder::new(ops::ALLOCA, location)
                    .add_operands(&[one])
                    .add_attributes(&[(
                        Identifier::new(context, "elem_type"),
                        melior::ir::attribute::TypeAttribute::new(i256).into(),
                    )])
                    .add_results(&[ptr_type])
                    .build()
                    .expect("valid llvm.alloca"),
            )
            .result(0)
            .expect("alloca has one result")
            .into()
    }

    /// Emits a binary arithmetic LLVM operation.
    fn emit_binary_op(
        &self,
        left: &Expression,
        right: &Expression,
        operator: &str,
        block: BlockRef<'c, 'b>,
    ) -> anyhow::Result<(Value<'c, 'b>, BlockRef<'c, 'b>)> {
        let (lhs, block) = self.emit(left, block)?;
        let (rhs, block) = self.emit(right, block)?;
        let op = match operator {
            "+" => ops::ADD,
            "-" => ops::SUB,
            "*" => ops::MUL,
            "/" => ops::UDIV,
            "%" => ops::UREM,
            _ => anyhow::bail!("unsupported binary operator: {operator}"),
        };
        let v = self.emit_llvm_op(op, lhs, rhs, &block);
        Ok((v, block))
    }

    /// Emits a generic two-operand LLVM operation.
    fn emit_llvm_op(
        &self,
        op_name: &str,
        lhs: Value<'c, 'b>,
        rhs: Value<'c, 'b>,
        block: &BlockRef<'c, 'b>,
    ) -> Value<'c, 'b> {
        self.state.emit_llvm_op(op_name, lhs, rhs, self.state.i256(), block)
    }

    /// Returns whether an expression has a signed integer type.
    fn is_signed_expr(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Identifier(t) => self.env.is_signed(t.text.as_str()),
            Expression::PrefixExpression(p) if p.operator.text == "-" => true,
            _ => false,
        }
    }

    /// Emits an `llvm.icmp` comparison, zero-extended to `i256`.
    ///
    /// Uses signed predicates when either operand is a signed integer type.
    fn emit_icmp(
        &self,
        left: &Expression,
        right: &Expression,
        operator: &str,
        block: BlockRef<'c, 'b>,
    ) -> anyhow::Result<(Value<'c, 'b>, BlockRef<'c, 'b>)> {
        let signed = self.is_signed_expr(left) || self.is_signed_expr(right);
        let (lhs, block) = self.emit(left, block)?;
        let (rhs, block) = self.emit(right, block)?;

        let predicate = match (operator, signed) {
            ("==", _) => ICmpPredicate::Eq,
            ("!=", _) => ICmpPredicate::Ne,
            (">", false) => ICmpPredicate::Ugt,
            (">", true) => ICmpPredicate::Sgt,
            (">=", false) => ICmpPredicate::Uge,
            (">=", true) => ICmpPredicate::Sge,
            ("<", false) => ICmpPredicate::Ult,
            ("<", true) => ICmpPredicate::Slt,
            ("<=", false) => ICmpPredicate::Ule,
            ("<=", true) => ICmpPredicate::Sle,
            _ => anyhow::bail!("unsupported comparison operator: {operator}"),
        };

        let cmp = self.state.emit_icmp(lhs, rhs, predicate, &block);
        let v = self.state.emit_zext_to_i256(cmp, &block);
        Ok((v, block))
    }

    /// Emits short-circuit `&&` using control flow.
    ///
    /// Result is always a canonical boolean (0 or 1).
    fn emit_and(
        &self,
        left: &Expression,
        right: &Expression,
        block: BlockRef<'c, 'b>,
    ) -> anyhow::Result<(Value<'c, 'b>, BlockRef<'c, 'b>)> {
        let (lhs, block) = self.emit(left, block)?;
        let i256 = self.state.i256();
        let location = self.state.location();

        let lhs_bool = self.emit_is_nonzero(lhs, &block);

        let rhs_block = self.region.append_block(Block::new(&[]));
        let merge_block = self.region.append_block(Block::new(&[(i256, location)]));

        let zero = self.emit_i256_constant(0, &block);
        block.append_operation(
            self.state.llvm_cond_br(lhs_bool, &rhs_block, &merge_block, &[], &[zero]),
        );

        let (rhs, rhs_block) = self.emit(right, rhs_block)?;
        let rhs_bool = self.emit_is_nonzero(rhs, &rhs_block);
        let rhs_normalized = self.state.emit_zext_to_i256(rhs_bool, &rhs_block);
        rhs_block.append_operation(self.state.llvm_br(&merge_block, &[rhs_normalized]));

        let result = merge_block.argument(0)?.into();
        Ok((result, merge_block))
    }

    /// Emits short-circuit `||` using control flow.
    ///
    /// Result is always a canonical boolean (0 or 1).
    fn emit_or(
        &self,
        left: &Expression,
        right: &Expression,
        block: BlockRef<'c, 'b>,
    ) -> anyhow::Result<(Value<'c, 'b>, BlockRef<'c, 'b>)> {
        let (lhs, block) = self.emit(left, block)?;
        let i256 = self.state.i256();
        let location = self.state.location();

        let lhs_bool = self.emit_is_nonzero(lhs, &block);
        let one = self.emit_i256_constant(1, &block);

        let rhs_block = self.region.append_block(Block::new(&[]));
        let merge_block = self.region.append_block(Block::new(&[(i256, location)]));

        block.append_operation(
            self.state.llvm_cond_br(lhs_bool, &merge_block, &rhs_block, &[one], &[]),
        );

        let (rhs, rhs_block) = self.emit(right, rhs_block)?;
        let rhs_bool = self.emit_is_nonzero(rhs, &rhs_block);
        let rhs_normalized = self.state.emit_zext_to_i256(rhs_bool, &rhs_block);
        rhs_block.append_operation(self.state.llvm_br(&merge_block, &[rhs_normalized]));

        let result = merge_block.argument(0)?.into();
        Ok((result, merge_block))
    }

    /// Emits postfix `++` or `--` (returns the old value).
    fn emit_postfix(
        &self,
        operand: &Expression,
        operator: &str,
        block: BlockRef<'c, 'b>,
    ) -> anyhow::Result<(Value<'c, 'b>, BlockRef<'c, 'b>)> {
        let Expression::Identifier(terminal) = operand else {
            anyhow::bail!("unsupported postfix operand: {operand:?}");
        };
        let name = terminal.text.as_str();
        let ptr = self.env.get_variable(name).ok_or_else(|| {
            anyhow::anyhow!("undefined variable: {name}")
        })?;
        let old = self.emit_load(ptr, &block)?;
        let one = self.emit_i256_constant(1, &block);
        let op = match operator {
            "++" => ops::ADD,
            "--" => ops::SUB,
            _ => anyhow::bail!("unsupported postfix operator: {operator}"),
        };
        let new = self.emit_llvm_op(op, old, one, &block);
        self.emit_store(new, ptr, &block)?;
        Ok((old, block))
    }

    /// Emits prefix `!` or `-`.
    fn emit_prefix(
        &self,
        operator: &str,
        operand: &Expression,
        block: BlockRef<'c, 'b>,
    ) -> anyhow::Result<(Value<'c, 'b>, BlockRef<'c, 'b>)> {
        let (val, block) = self.emit(operand, block)?;
        match operator {
            "!" => {
                let zero = self.emit_i256_constant(0, &block);
                let cmp = self.state.emit_icmp(val, zero, ICmpPredicate::Eq, &block);
                let v = self.state.emit_zext_to_i256(cmp, &block);
                Ok((v, block))
            }
            "-" => {
                let zero = self.emit_i256_constant(0, &block);
                let v = self.emit_llvm_op(ops::SUB, zero, val, &block);
                Ok((v, block))
            }
            _ => anyhow::bail!("unsupported prefix operator: {operator}"),
        }
    }

    /// Emits an `icmp ne 0` producing `i1` from an `i256`.
    pub fn emit_is_nonzero(
        &self,
        value: Value<'c, 'b>,
        block: &BlockRef<'c, 'b>,
    ) -> Value<'c, 'b> {
        let zero = self.emit_i256_constant(0, block);
        self.state.emit_icmp(value, zero, ICmpPredicate::Ne, block)
    }

    /// Emits a function call expression.
    ///
    /// Resolves the callee by name and argument count, handling void and
    /// value-returning functions correctly.
    fn emit_function_call(
        &self,
        callee: &Expression,
        arguments: &ArgumentsDeclaration,
        block: BlockRef<'c, 'b>,
    ) -> anyhow::Result<(Value<'c, 'b>, BlockRef<'c, 'b>)> {
        let ArgumentsDeclaration::PositionalArguments(args) = arguments else {
            anyhow::bail!("only positional arguments supported");
        };

        // Handle member function calls: recipient.send(1), recipient.transfer(1).
        if let Expression::MemberAccessExpression(member) = callee {
            let method = member.member.text.as_str();
            match method {
                "send" | "transfer" => {
                    return self.emit_address_call(
                        &member.operand,
                        args,
                        method == "transfer",
                        block,
                    );
                }
                _ => {}
            }
        }

        // Handle type-conversion calls: payable(x), uint256(x), etc.
        let is_type_conversion = match callee {
            Expression::PayableKeyword => true,
            Expression::Identifier(t) => matches!(
                t.text.as_str(),
                "payable" | "address" | "uint256" | "uint8" | "int256" | "bool"
            ),
            _ => false,
        };
        if is_type_conversion && args.len() == 1 {
            return self.emit(&args[0], block);
        }

        let callee_name = match callee {
            Expression::Identifier(terminal) => terminal.text.as_str(),
            Expression::MemberAccessExpression(member) => member.member.text.as_str(),
            _ => anyhow::bail!("unsupported callee expression: {callee:?}"),
        };

        let mut arg_values = Vec::new();
        let mut current_block = block;

        for arg in args {
            let (val, blk) = self.emit(arg, current_block)?;
            arg_values.push(val);
            current_block = blk;
        }

        let (mlir_name, has_returns) =
            self.state.resolve_function(callee_name, arg_values.len())?;

        if has_returns {
            let i256 = self.state.i256();
            let result = self
                .state
                .emit_call(mlir_name, &arg_values, &[i256], &current_block)?
                .expect("function call has result");
            Ok((result, current_block))
        } else {
            self.state
                .emit_call(mlir_name, &arg_values, &[], &current_block)?;
            let zero = self.emit_i256_constant(0, &current_block);
            Ok((zero, current_block))
        }
    }

    /// Emits a member access expression (e.g. `tx.origin`, `msg.sender`).
    fn emit_member_access(
        &self,
        operand: &Expression,
        member: &str,
        block: BlockRef<'c, 'b>,
    ) -> anyhow::Result<(Value<'c, 'b>, BlockRef<'c, 'b>)> {
        if let Expression::Identifier(terminal) = operand {
            let obj = terminal.text.as_str();
            let intrinsic = match (obj, member) {
                ("tx", "origin") => ops::EVM_ORIGIN,
                ("tx", "gasprice") => ops::EVM_GASPRICE,
                ("msg", "sender") => ops::EVM_CALLER,
                ("msg", "value") => ops::EVM_CALLVALUE,
                ("block", "timestamp") => ops::EVM_TIMESTAMP,
                ("block", "number") => ops::EVM_NUMBER,
                ("block", "coinbase") => ops::EVM_COINBASE,
                ("block", "chainid") => ops::EVM_CHAINID,
                ("block", "basefee") => ops::EVM_BASEFEE,
                ("block", "gaslimit") => ops::EVM_GASLIMIT,
                _ => anyhow::bail!("unsupported member access: {obj}.{member}"),
            };
            let v = self
                .emit_intrinsic_call(intrinsic, &[], true, &block)?
                .expect("intrinsic has result");
            return Ok((v, block));
        }
        anyhow::bail!("unsupported member access on non-identifier operand")
    }

    /// Emits `address.send(value)` or `address.transfer(value)` as `llvm.evm.call`.
    ///
    /// Both emit an EVM CALL with 2300 gas stipend. Transfer additionally
    /// reverts when the call fails by checking the return value and branching
    /// to a revert block.
    fn emit_address_call(
        &self,
        address_expr: &Expression,
        args: &[Expression],
        revert_on_fail: bool,
        block: BlockRef<'c, 'b>,
    ) -> anyhow::Result<(Value<'c, 'b>, BlockRef<'c, 'b>)> {
        let (address, block) = self.emit(address_expr, block)?;

        let (value, block) = if args.len() == 1 {
            self.emit(&args[0], block)?
        } else {
            (self.emit_i256_constant(0, &block), block)
        };

        let gas = self.emit_i256_constant(2300, &block);
        let zero = self.emit_i256_constant(0, &block);
        let null_ptr = self.emit_inttoptr(zero, self.state.ptr(AddressSpace::Heap), &block);

        // call(gas, addr, value, argsOffset, argsLen, retOffset, retLen)
        let result = self
            .emit_intrinsic_call(
                ops::EVM_CALL,
                &[gas, address, value, null_ptr, zero, null_ptr, zero],
                true,
                &block,
            )?
            .expect("evm.call has result");

        if revert_on_fail {
            // transfer: revert if call failed (result == 0).
            let is_zero_i1 = self.state.emit_icmp(
                result,
                zero,
                ICmpPredicate::Eq,
                &block,
            );

            // Insert blocks into region to get BlockRefs (avoids linear walk).
            let revert_ref = self.region.append_block(Block::new(&[]));
            let cont_ref = self.region.append_block(Block::new(&[]));

            // Populate revert block.
            let revert_zero = self.emit_i256_constant(0, &revert_ref);
            let heap_ptr_type = self.state.ptr(AddressSpace::Heap);
            let revert_ptr = self.emit_inttoptr(revert_zero, heap_ptr_type, &revert_ref);
            self.emit_intrinsic_call(
                ops::EVM_REVERT,
                &[revert_ptr, revert_zero],
                false,
                &revert_ref,
            )?;
            revert_ref.append_operation(llvm::unreachable(self.state.location()));

            block.append_operation(
                self.state
                    .llvm_cond_br(is_zero_i1, &revert_ref, &cont_ref, &[], &[]),
            );

            let zero2 = self.emit_i256_constant(0, &cont_ref);
            Ok((zero2, cont_ref))
        } else {
            Ok((result, block))
        }
    }

    /// Emits an `llvm.inttoptr` cast.
    fn emit_inttoptr(
        &self,
        value: Value<'c, 'b>,
        ptr_type: melior::ir::Type<'c>,
        block: &BlockRef<'c, 'b>,
    ) -> Value<'c, 'b> {
        self.state.emit_inttoptr(value, ptr_type, block)
    }

    /// Emits a call to an EVM intrinsic function.
    ///
    /// Returns `Some(value)` when `has_result` is true, `None` for void calls.
    fn emit_intrinsic_call(
        &self,
        name: &str,
        operands: &[Value<'c, 'b>],
        has_result: bool,
        block: &BlockRef<'c, 'b>,
    ) -> anyhow::Result<Option<Value<'c, 'b>>> {
        let result_types: Vec<melior::ir::Type<'c>> = if has_result {
            vec![self.state.i256()]
        } else {
            vec![]
        };
        self.state.emit_call(name, operands, &result_types, block)
    }
}
