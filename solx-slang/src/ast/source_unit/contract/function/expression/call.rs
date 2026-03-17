//!
//! Function call and member access expression lowering.
//!

use melior::ir::Block;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::RegionLike;
use melior::ir::Value;
use slang_solidity::backend::ir::ast::ArgumentsDeclaration;
use slang_solidity::backend::ir::ast::Expression;
use slang_solidity::backend::ir::ast::PositionalArguments;

use solx_mlir::ICmpPredicate;
use solx_utils::AddressSpace;

use crate::ast::source_unit::contract::function::expression::ExpressionEmitter;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Emits a function call expression.
    ///
    /// Resolves the callee by name and argument count, handling void and
    /// value-returning functions correctly.
    pub(super) fn emit_function_call(
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
        // TODO: detect built-in send/transfer via Slang's `Typing::BuiltIn` once
        // exposed in the semi-public API.
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
        // TODO: detect casts via Slang's binder once exposed in the semi-public
        // API, instead of matching on a hardcoded set of type names.
        let is_type_conversion = matches!(
            callee_name.as_str(),
            "payable" | "address" | "uint256" | "uint8" | "int256" | "bool"
        ) && matches!(
            callee,
            Expression::PayableKeyword | Expression::Identifier(_)
        );
        if is_type_conversion && positional_arguments.len() == 1 {
            let first = positional_arguments
                .iter()
                .next()
                .expect("len checked to be 1 above");
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
                .emit_sol_call(mlir_name, &argument_values, &[i256], &current_block)?
                .expect("function call always produces one result");
            Ok((result, current_block))
        } else {
            self.state
                .emit_sol_call(mlir_name, &argument_values, &[], &current_block)?;
            let zero = self.state.emit_sol_constant(0, &current_block);
            Ok((zero, current_block))
        }
    }

    /// Emits a member access expression (e.g. `tx.origin`, `msg.sender`).
    ///
    // TODO: detect built-in member accesses (e.g. `tx.origin`, `msg.sender`)
    // via Slang's `Typing::BuiltIn` variants once exposed in the semi-public
    // API, instead of matching on string names.
    pub(super) fn emit_member_access(
        &self,
        operand: &Expression,
        member: &str,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        if let Expression::Identifier(id) = operand {
            let object = id.name();
            let intrinsic = match (object.as_str(), member) {
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
                _ => anyhow::bail!("unsupported member access: {object}.{member}"),
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
    pub(super) fn emit_address_call(
        &self,
        address_expr: &Expression,
        arguments: &PositionalArguments,
        revert_on_fail: bool,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (address, block) = self.emit(address_expr, block)?;

        let (value, block) = if arguments.len() == 1 {
            let first = arguments.iter().next().expect("len checked to be 1 above");
            self.emit(&first, block)?
        } else {
            (self.state.emit_sol_constant(0, &block), block)
        };

        let gas = self
            .state
            .emit_sol_constant(Self::TRANSFER_GAS_STIPEND, &block);
        let zero = self.state.emit_sol_constant(0, &block);
        let null_pointer =
            self.state
                .emit_inttoptr(zero, self.state.pointer(AddressSpace::Heap), &block);

        // call(gas, addr, value, argsOffset, argsLen, retOffset, retLen)
        let result = self
            .emit_intrinsic_call(
                solx_mlir::ops::EVM_CALL,
                &[gas, address, value, null_pointer, zero, null_pointer, zero],
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
            let revert_zero = self.state.emit_sol_constant(0, &revert_ref);
            let heap_pointer_type = self.state.pointer(AddressSpace::Heap);
            let revert_pointer =
                self.state
                    .emit_inttoptr(revert_zero, heap_pointer_type, &revert_ref);
            self.emit_intrinsic_call(
                solx_mlir::ops::EVM_REVERT,
                &[revert_pointer, revert_zero],
                false,
                &revert_ref,
            )?;
            revert_ref.append_operation(melior::dialect::llvm::unreachable(self.state.location()));

            block.append_operation(self.state.llvm_cond_br(
                is_zero_i1,
                &revert_ref,
                &cont_ref,
                &[],
                &[],
            ));

            let zero2 = self.state.emit_sol_constant(0, &cont_ref);
            Ok((zero2, cont_ref))
        } else {
            Ok((result, block))
        }
    }
}
