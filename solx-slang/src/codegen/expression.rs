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

use slang_solidity::backend::ir::ast::ArgumentsDeclaration;
use slang_solidity::backend::ir::ast::Expression;
use slang_solidity::backend::ir::ast::PositionalArguments;

use solx_mlir::Environment;
use solx_mlir::ICmpPredicate;
use solx_utils::AddressSpace;

use crate::codegen::MlirContext;

/// Gas stipend for `address.transfer()` and `address.send()` calls.
const TRANSFER_GAS_STIPEND: i64 = 2300;

/// Lowers Solidity expressions to MLIR SSA values.
pub struct ExpressionEmitter<'state, 'context, 'block> {
    /// The shared MLIR context.
    state: &'state MlirContext<'context>,
    /// Variable environment.
    environment: &'state Environment<'context, 'block>,
    /// The function region for creating new blocks.
    region: &'state Region<'context>,
}

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Creates a new expression emitter.
    pub(crate) fn new(
        state: &'state MlirContext<'context>,
        environment: &'state Environment<'context, 'block>,
        region: &'state Region<'context>,
    ) -> Self {
        Self {
            state,
            environment,
            region,
        }
    }

    /// Emits MLIR for an expression, appending operations to `block`.
    ///
    /// Returns the SSA value produced and the continuation block (which may
    /// differ from the input block for short-circuit operators).
    pub(crate) fn emit(
        &self,
        expr: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        match expr {
            Expression::DecimalNumberExpression(decimal) => {
                let literal = decimal.literal();
                let text = literal.text.as_str();
                let value = if let Ok(value) = text.parse::<i64>() {
                    self.emit_i256_constant(value, &block)
                } else {
                    self.state.emit_i256_from_decimal_str(text, &block)?
                };
                Ok((value, block))
            }
            Expression::HexNumberExpression(hex) => {
                let literal = hex.literal();
                let text = literal.text.as_str();
                let stripped = text
                    .strip_prefix("0x")
                    .or(text.strip_prefix("0X"))
                    .unwrap_or(text);
                let value = if let Ok(parsed) = i64::from_str_radix(stripped, 16) {
                    self.emit_i256_constant(parsed, &block)
                } else {
                    self.state.emit_i256_from_hex_str(stripped, &block)?
                };
                Ok((value, block))
            }
            Expression::TrueKeyword => {
                let value = self.emit_i256_constant(1, &block);
                Ok((value, block))
            }
            Expression::FalseKeyword => {
                let value = self.emit_i256_constant(0, &block);
                Ok((value, block))
            }
            Expression::Identifier(identifier) => {
                let name = identifier.name();
                if let Some(ptr) = self.environment.get_variable(&name) {
                    let value = self.emit_load(ptr, &block)?;
                    Ok((value, block))
                } else if let Some(slot) = self.state.state_variable_slot(&name) {
                    let value = self.emit_storage_load(slot, &block)?;
                    Ok((value, block))
                } else {
                    anyhow::bail!("undefined variable: {name}")
                }
            }
            Expression::AssignmentExpression(assign) => self.emit_assignment(assign, block),
            Expression::AdditiveExpression(expr) => {
                let left = expr.left_operand();
                let right = expr.right_operand();
                let operator = expr.operator();
                self.emit_binary_op(&left, &right, &operator.text, block)
            }
            Expression::MultiplicativeExpression(expr) => {
                let left = expr.left_operand();
                let right = expr.right_operand();
                let operator = expr.operator();
                self.emit_binary_op(&left, &right, &operator.text, block)
            }
            Expression::EqualityExpression(expr) => {
                let left = expr.left_operand();
                let right = expr.right_operand();
                let operator = expr.operator();
                self.emit_icmp(&left, &right, &operator.text, block)
            }
            Expression::InequalityExpression(expr) => {
                let left = expr.left_operand();
                let right = expr.right_operand();
                let operator = expr.operator();
                self.emit_icmp(&left, &right, &operator.text, block)
            }
            Expression::AndExpression(expr) => {
                let left = expr.left_operand();
                let right = expr.right_operand();
                self.emit_and(&left, &right, block)
            }
            Expression::OrExpression(expr) => {
                let left = expr.left_operand();
                let right = expr.right_operand();
                self.emit_or(&left, &right, block)
            }
            Expression::PostfixExpression(expr) => {
                let operand = expr.operand();
                let operator = expr.operator();
                self.emit_postfix(&operand, &operator.text, block)
            }
            Expression::PrefixExpression(expr) => {
                let operator = expr.operator();
                let operand = expr.operand();
                self.emit_prefix(&operator.text, &operand, block)
            }
            Expression::BitwiseAndExpression(expr) => {
                let left = expr.left_operand();
                let right = expr.right_operand();
                self.emit_binary_op(&left, &right, "&", block)
            }
            Expression::BitwiseOrExpression(expr) => {
                let left = expr.left_operand();
                let right = expr.right_operand();
                self.emit_binary_op(&left, &right, "|", block)
            }
            Expression::BitwiseXorExpression(expr) => {
                let left = expr.left_operand();
                let right = expr.right_operand();
                self.emit_binary_op(&left, &right, "^", block)
            }
            Expression::ShiftExpression(expr) => {
                let left = expr.left_operand();
                let right = expr.right_operand();
                let operator = expr.operator();
                let (lhs, block) = self.emit(&left, block)?;
                let (rhs, block) = self.emit(&right, block)?;
                let op = match operator.text.as_str() {
                    "<<" => solx_mlir::ops::SHL,
                    ">>" => solx_mlir::ops::LSHR,
                    _ => anyhow::bail!("unsupported shift operator: {}", operator.text),
                };
                let value = self.emit_llvm_op(op, lhs, rhs, &block)?;
                Ok((value, block))
            }
            Expression::FunctionCallExpression(call) => {
                let callee = call.operand();
                let arguments = call.arguments();
                self.emit_function_call(&callee, &arguments, block)
            }
            Expression::MemberAccessExpression(access) => {
                let operand = access.operand();
                let member = access.member().name();
                self.emit_member_access(&operand, &member, block)
            }
            _ => anyhow::bail!("unsupported expression: {:?}", std::mem::discriminant(expr)),
        }
    }

    /// Emits an `llvm.mlir.constant` producing an `i256` value.
    pub(crate) fn emit_i256_constant(
        &self,
        value: i64,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        self.state.emit_i256_constant(value, block)
    }

    /// Emits an `llvm.store` to a pointer.
    pub(crate) fn emit_store(
        &self,
        value: Value<'context, 'block>,
        ptr: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) {
        self.state.emit_store(value, ptr, block);
    }

    /// Emits an `llvm.alloca` for a local variable, returning the pointer.
    pub(crate) fn emit_alloca(
        &self,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let i256 = self.state.i256();
        let location = self.state.location();
        let context = self.state.context();
        let ptr_type = self.state.ptr(AddressSpace::Stack);

        let one = self.emit_i256_constant(1, block);
        block
            .append_operation(
                OperationBuilder::new(solx_mlir::ops::ALLOCA, location)
                    .add_operands(&[one])
                    .add_attributes(&[(
                        Identifier::new(context, "elem_type"),
                        melior::ir::attribute::TypeAttribute::new(i256).into(),
                    )])
                    .add_results(&[ptr_type])
                    .build()
                    .expect("llvm.alloca operation is well-formed"),
            )
            .result(0)
            .expect("alloca always produces one result")
            .into()
    }

    /// Emits an `icmp ne 0` producing `i1` from an `i256`.
    pub(crate) fn emit_is_nonzero(
        &self,
        value: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let zero = self.emit_i256_constant(0, block);
        self.state.emit_icmp(value, zero, ICmpPredicate::Ne, block)
    }

    /// Emits an assignment expression (`=`, `+=`, `-=`, `*=`).
    fn emit_assignment(
        &self,
        assign: &slang_solidity::backend::ir::ast::AssignmentExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let left = assign.left_operand();
        let Expression::Identifier(identifier) = &left else {
            anyhow::bail!("unsupported assignment target");
        };
        let name = identifier.name();

        // Determine whether this is a local variable or a state variable.
        let local_ptr = self.environment.get_variable(&name);
        let storage_slot = self.state.state_variable_slot(&name);
        if local_ptr.is_none() && storage_slot.is_none() {
            anyhow::bail!("undefined variable: {name}");
        }

        let operator = assign.operator();
        let op = operator.text.as_str();
        let right = assign.right_operand();
        let (value, block) = if op == "=" {
            self.emit(&right, block)?
        } else {
            let old = if let Some(ptr) = local_ptr {
                self.emit_load(ptr, &block)?
            } else {
                let slot = storage_slot.ok_or_else(|| {
                    anyhow::anyhow!("state variable '{name}' has no assigned storage slot")
                })?;
                self.emit_storage_load(slot, &block)?
            };
            let (rhs, block) = self.emit(&right, block)?;
            let arith_op = match op {
                "+=" => solx_mlir::ops::ADD,
                "-=" => solx_mlir::ops::SUB,
                "*=" => solx_mlir::ops::MUL,
                "/=" => solx_mlir::ops::UDIV,
                "%=" => solx_mlir::ops::UREM,
                _ => anyhow::bail!("unsupported assignment operator: {op}"),
            };
            let result = self.emit_llvm_op(arith_op, old, rhs, &block)?;
            (result, block)
        };

        if let Some(ptr) = local_ptr {
            self.emit_store(value, ptr, &block);
        } else {
            let slot = storage_slot.ok_or_else(|| {
                anyhow::anyhow!("state variable '{name}' has no assigned storage slot")
            })?;
            self.emit_storage_store(slot, value, &block);
        }
        Ok((value, block))
    }

    /// Emits an `llvm.load` from a pointer.
    fn emit_load(
        &self,
        ptr: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<Value<'context, 'block>> {
        self.state.emit_load(ptr, self.state.i256(), block)
    }

    /// Emits a storage load (`inttoptr` slot to `ptr addrspace(5)`, then `llvm.load`).
    fn emit_storage_load(
        &self,
        slot: u64,
        block: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<Value<'context, 'block>> {
        let i256 = self.state.i256();
        let storage_ptr_type = self.state.ptr(AddressSpace::Storage);
        let slot_value = self.state.emit_i256_from_u64(slot, block);
        let ptr = self
            .state
            .emit_inttoptr(slot_value, storage_ptr_type, block);
        self.state.emit_load(ptr, i256, block)
    }

    /// Emits a storage store (`inttoptr` slot to `ptr addrspace(5)`, then `llvm.store`).
    fn emit_storage_store(
        &self,
        slot: u64,
        value: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) {
        let storage_ptr_type = self.state.ptr(AddressSpace::Storage);
        let slot_value = self.state.emit_i256_from_u64(slot, block);
        let ptr = self
            .state
            .emit_inttoptr(slot_value, storage_ptr_type, block);
        self.state.emit_store(value, ptr, block);
    }

    /// Emits a binary arithmetic LLVM operation.
    fn emit_binary_op(
        &self,
        left: &Expression,
        right: &Expression,
        operator: &str,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (lhs, block) = self.emit(left, block)?;
        let (rhs, block) = self.emit(right, block)?;
        let op = match operator {
            "+" => solx_mlir::ops::ADD,
            "-" => solx_mlir::ops::SUB,
            "*" => solx_mlir::ops::MUL,
            "/" => solx_mlir::ops::UDIV,
            "%" => solx_mlir::ops::UREM,
            "&" => solx_mlir::ops::AND,
            "|" => solx_mlir::ops::OR,
            "^" => solx_mlir::ops::XOR,
            _ => anyhow::bail!("unsupported binary operator: {operator}"),
        };
        let value = self.emit_llvm_op(op, lhs, rhs, &block)?;
        Ok((value, block))
    }

    /// Emits a generic two-operand LLVM operation.
    fn emit_llvm_op(
        &self,
        op_name: &str,
        lhs: Value<'context, 'block>,
        rhs: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<Value<'context, 'block>> {
        self.state
            .emit_llvm_op(op_name, lhs, rhs, self.state.i256(), block)
    }

    /// Returns whether an expression has a signed integer type.
    fn is_signed_expr(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Identifier(id) => self.environment.is_signed(&id.name()),
            Expression::PrefixExpression(p) if p.operator().text == "-" => true,
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
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
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
        let value = self.state.emit_zext_to_i256(cmp, &block);
        Ok((value, block))
    }

    /// Emits short-circuit `&&` using control flow.
    ///
    /// Result is always a canonical boolean (0 or 1).
    fn emit_and(
        &self,
        left: &Expression,
        right: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (lhs, block) = self.emit(left, block)?;
        let i256 = self.state.i256();
        let location = self.state.location();

        let lhs_bool = self.emit_is_nonzero(lhs, &block);

        let rhs_block = self.region.append_block(Block::new(&[]));
        let merge_block = self.region.append_block(Block::new(&[(i256, location)]));

        let zero = self.emit_i256_constant(0, &block);
        block.append_operation(self.state.llvm_cond_br(
            lhs_bool,
            &rhs_block,
            &merge_block,
            &[],
            &[zero],
        ));

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
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (lhs, block) = self.emit(left, block)?;
        let i256 = self.state.i256();
        let location = self.state.location();

        let lhs_bool = self.emit_is_nonzero(lhs, &block);
        let one = self.emit_i256_constant(1, &block);

        let rhs_block = self.region.append_block(Block::new(&[]));
        let merge_block = self.region.append_block(Block::new(&[(i256, location)]));

        block.append_operation(self.state.llvm_cond_br(
            lhs_bool,
            &merge_block,
            &rhs_block,
            &[one],
            &[],
        ));

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
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let Expression::Identifier(identifier) = operand else {
            anyhow::bail!("unsupported postfix operand");
        };
        let name = identifier.name();
        let ptr = self
            .environment
            .get_variable(&name)
            .ok_or_else(|| anyhow::anyhow!("undefined variable: {name}"))?;
        let old = self.emit_load(ptr, &block)?;
        let one = self.emit_i256_constant(1, &block);
        let op = match operator {
            "++" => solx_mlir::ops::ADD,
            "--" => solx_mlir::ops::SUB,
            _ => anyhow::bail!("unsupported postfix operator: {operator}"),
        };
        let new = self.emit_llvm_op(op, old, one, &block)?;
        self.emit_store(new, ptr, &block);
        Ok((old, block))
    }

    /// Emits prefix `!` or `-`.
    fn emit_prefix(
        &self,
        operator: &str,
        operand: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (value, block) = self.emit(operand, block)?;
        match operator {
            "!" => {
                let zero = self.emit_i256_constant(0, &block);
                let cmp = self.state.emit_icmp(value, zero, ICmpPredicate::Eq, &block);
                let result = self.state.emit_zext_to_i256(cmp, &block);
                Ok((result, block))
            }
            "-" => {
                let zero = self.emit_i256_constant(0, &block);
                let result = self.emit_llvm_op(solx_mlir::ops::SUB, zero, value, &block)?;
                Ok((result, block))
            }
            _ => anyhow::bail!("unsupported prefix operator: {operator}"),
        }
    }

    /// Emits a function call expression.
    ///
    /// Resolves the callee by name and argument count, handling void and
    /// value-returning functions correctly.
    fn emit_function_call(
        &self,
        callee: &Expression,
        arguments: &ArgumentsDeclaration,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let ArgumentsDeclaration::PositionalArguments(positional_arguments) = arguments else {
            anyhow::bail!("only positional arguments supported");
        };

        // Resolve callee name once for all downstream checks.
        let callee_name = match callee {
            Expression::Identifier(id) => id.name(),
            Expression::MemberAccessExpression(member) => member.member().name(),
            Expression::PayableKeyword => "payable".to_owned(),
            _ => anyhow::bail!("unsupported callee expression"),
        };

        // Handle member function calls: recipient.send(1), recipient.transfer(1).
        if let Expression::MemberAccessExpression(member) = callee {
            match callee_name.as_str() {
                "send" | "transfer" => {
                    let operand = member.operand();
                    return self.emit_address_call(
                        &operand,
                        positional_arguments,
                        callee_name == "transfer",
                        block,
                    );
                }
                _ => {}
            }
        }

        // Handle type-conversion calls: payable(x), uint256(x), etc.
        let is_type_conversion = matches!(
            callee_name.as_str(),
            "payable" | "address" | "uint256" | "uint8" | "int256" | "bool"
        ) && matches!(
            callee,
            Expression::PayableKeyword | Expression::Identifier(_)
        );
        if is_type_conversion && positional_arguments.len() == 1 {
            let first = positional_arguments.iter().next().unwrap();
            return self.emit(&first, block);
        }

        let mut argument_values = Vec::new();
        let mut current_block = block;

        for argument in positional_arguments.iter() {
            let (value, next_block) = self.emit(&argument, current_block)?;
            argument_values.push(value);
            current_block = next_block;
        }

        let (mlir_name, has_returns) = self
            .state
            .resolve_function(&callee_name, argument_values.len())?;

        if has_returns {
            let i256 = self.state.i256();
            let result = self
                .state
                .emit_call(mlir_name, &argument_values, &[i256], &current_block)?
                .expect("function call always produces one result");
            Ok((result, current_block))
        } else {
            self.state
                .emit_call(mlir_name, &argument_values, &[], &current_block)?;
            let zero = self.emit_i256_constant(0, &current_block);
            Ok((zero, current_block))
        }
    }

    /// Emits a member access expression (e.g. `tx.origin`, `msg.sender`).
    fn emit_member_access(
        &self,
        operand: &Expression,
        member: &str,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        if let Expression::Identifier(id) = operand {
            let obj = id.name();
            let intrinsic = match (obj.as_str(), member) {
                ("tx", "origin") => solx_mlir::ops::EVM_ORIGIN,
                ("tx", "gasprice") => solx_mlir::ops::EVM_GASPRICE,
                ("msg", "sender") => solx_mlir::ops::EVM_CALLER,
                ("msg", "value") => solx_mlir::ops::EVM_CALLVALUE,
                ("block", "timestamp") => solx_mlir::ops::EVM_TIMESTAMP,
                ("block", "number") => solx_mlir::ops::EVM_NUMBER,
                ("block", "coinbase") => solx_mlir::ops::EVM_COINBASE,
                ("block", "chainid") => solx_mlir::ops::EVM_CHAINID,
                ("block", "basefee") => solx_mlir::ops::EVM_BASEFEE,
                ("block", "gaslimit") => solx_mlir::ops::EVM_GASLIMIT,
                _ => anyhow::bail!("unsupported member access: {obj}.{member}"),
            };
            let value = self
                .emit_intrinsic_call(intrinsic, &[], true, &block)?
                .expect("intrinsic always produces one result");
            return Ok((value, block));
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
        arguments: &PositionalArguments,
        revert_on_fail: bool,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (address, block) = self.emit(address_expr, block)?;

        let (value, block) = if arguments.len() == 1 {
            let first = arguments.iter().next().unwrap();
            self.emit(&first, block)?
        } else {
            (self.emit_i256_constant(0, &block), block)
        };

        let gas = self.emit_i256_constant(TRANSFER_GAS_STIPEND, &block);
        let zero = self.emit_i256_constant(0, &block);
        let null_ptr = self.emit_inttoptr(zero, self.state.ptr(AddressSpace::Heap), &block);

        // call(gas, addr, value, argsOffset, argsLen, retOffset, retLen)
        let result = self
            .emit_intrinsic_call(
                solx_mlir::ops::EVM_CALL,
                &[gas, address, value, null_ptr, zero, null_ptr, zero],
                true,
                &block,
            )?
            .expect("evm.call always produces one result");

        if revert_on_fail {
            // transfer: revert if call failed (result == 0).
            let is_zero_i1 = self
                .state
                .emit_icmp(result, zero, ICmpPredicate::Eq, &block);

            // Insert blocks into region to get BlockRefs (avoids linear walk).
            let revert_ref = self.region.append_block(Block::new(&[]));
            let cont_ref = self.region.append_block(Block::new(&[]));

            // Populate revert block.
            let revert_zero = self.emit_i256_constant(0, &revert_ref);
            let heap_ptr_type = self.state.ptr(AddressSpace::Heap);
            let revert_ptr = self.emit_inttoptr(revert_zero, heap_ptr_type, &revert_ref);
            self.emit_intrinsic_call(
                solx_mlir::ops::EVM_REVERT,
                &[revert_ptr, revert_zero],
                false,
                &revert_ref,
            )?;
            revert_ref.append_operation(llvm::unreachable(self.state.location()));

            block.append_operation(self.state.llvm_cond_br(
                is_zero_i1,
                &revert_ref,
                &cont_ref,
                &[],
                &[],
            ));

            let zero2 = self.emit_i256_constant(0, &cont_ref);
            Ok((zero2, cont_ref))
        } else {
            Ok((result, block))
        }
    }

    /// Emits an `llvm.inttoptr` cast.
    fn emit_inttoptr(
        &self,
        value: Value<'context, 'block>,
        ptr_type: melior::ir::Type<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        self.state.emit_inttoptr(value, ptr_type, block)
    }

    /// Emits a call to an EVM intrinsic function.
    ///
    /// Returns `Some(value)` when `has_result` is true, `None` for void calls.
    fn emit_intrinsic_call(
        &self,
        name: &str,
        operands: &[Value<'context, 'block>],
        has_result: bool,
        block: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<Value<'context, 'block>>> {
        let result_types: Vec<melior::ir::Type<'context>> = if has_result {
            vec![self.state.i256()]
        } else {
            vec![]
        };
        self.state.emit_call(name, operands, &result_types, block)
    }
}
