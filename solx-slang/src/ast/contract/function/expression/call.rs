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

use solx_mlir::ICmpPredicate;

use crate::ast::contract::function::expression::ExpressionEmitter;

/// Lowers function call and member access expressions to MLIR.
pub struct CallEmitter<'emitter, 'state, 'context, 'block> {
    /// The parent expression emitter for recursive subexpression emission.
    expression_emitter: &'emitter ExpressionEmitter<'state, 'context, 'block>,
}

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Solidity built-in `require()` function name.
    const REQUIRE: &'static str = "require";

    /// Creates a new call emitter.
    pub fn new(expression_emitter: &'emitter ExpressionEmitter<'state, 'context, 'block>) -> Self {
        Self { expression_emitter }
    }

    /// Emits a function call expression.
    ///
    /// Resolves the callee by name and argument count, handling type
    /// conversions, built-in functions, and user-defined calls.
    pub fn emit_function_call(
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
            Expression::Identifier(identifier) => identifier.name(),
            Expression::PayableKeyword => "payable".to_owned(),
            _ => anyhow::bail!("unsupported callee expression"),
        };

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
            let (value, block) = self.expression_emitter.emit(&first, block)?;
            return match callee_name.as_str() {
                "bool" => {
                    // bool(x) → x != 0, zero-extended to i256.
                    let zero = self
                        .expression_emitter
                        .state
                        .builder()
                        .emit_sol_constant(0, &block);
                    let cmp = self.expression_emitter.state.builder().emit_icmp(
                        value,
                        zero,
                        ICmpPredicate::Ne,
                        &block,
                    );
                    let result = self
                        .expression_emitter
                        .state
                        .builder()
                        .emit_zext_to_i256(cmp, &block);
                    Ok((result, block))
                }
                "uint8" => {
                    let mask = self
                        .expression_emitter
                        .state
                        .builder()
                        .emit_sol_constant(0xFF, &block);
                    let result = self.expression_emitter.emit_llvm_operation(
                        solx_mlir::Builder::AND,
                        value,
                        mask,
                        &block,
                    )?;
                    Ok((result, block))
                }
                "address" | "payable" => {
                    // Truncate to 160 bits: value & ((1 << 160) - 1).
                    let one = self
                        .expression_emitter
                        .state
                        .builder()
                        .emit_sol_constant(1, &block);
                    let bit_width = self
                        .expression_emitter
                        .state
                        .builder()
                        .emit_sol_constant(solx_utils::BIT_LENGTH_ETH_ADDRESS as i64, &block);
                    let shifted = self.expression_emitter.emit_llvm_operation(
                        solx_mlir::Builder::SHL,
                        one,
                        bit_width,
                        &block,
                    )?;
                    let mask = self.expression_emitter.emit_llvm_operation(
                        solx_mlir::Builder::SUB,
                        shifted,
                        one,
                        &block,
                    )?;
                    let result = self.expression_emitter.emit_llvm_operation(
                        solx_mlir::Builder::AND,
                        value,
                        mask,
                        &block,
                    )?;
                    Ok((result, block))
                }
                // Word-sized types need no truncation.
                "uint256" | "int256" => Ok((value, block)),
                _ => Ok((value, block)),
            };
        }

        // Handle require() built-in.
        if callee_name == Self::REQUIRE
            && (positional_arguments.len() == 1 || positional_arguments.len() == 2)
        {
            // TODO: encode revert reason from second argument
            let first = positional_arguments
                .iter()
                .next()
                .expect("length checked above");
            return self.emit_require(&first, block);
        }

        let mut argument_values = Vec::new();
        let mut current_block = block;

        for argument in positional_arguments.iter() {
            let (value, next_block) = self.expression_emitter.emit(&argument, current_block)?;
            argument_values.push(value);
            current_block = next_block;
        }

        let (mlir_name, return_count) = self
            .expression_emitter
            .state
            .resolve_function(&callee_name, argument_values.len())?;

        if return_count > 0 {
            let i256 = self.expression_emitter.state.i256();
            let result_types: Vec<melior::ir::Type<'context>> =
                (0..return_count).map(|_| i256).collect();
            let result = self
                .expression_emitter
                .state
                .builder()
                .emit_sol_call(mlir_name, &argument_values, &result_types, &current_block)?
                .expect("function call always produces at least one result");
            Ok((result, current_block))
        } else {
            // TODO: return None for void calls instead of fabricating zero
            self.expression_emitter.state.builder().emit_sol_call(
                mlir_name,
                &argument_values,
                &[],
                &current_block,
            )?;
            let zero = self
                .expression_emitter
                .state
                .builder()
                .emit_sol_constant(0, &current_block);
            Ok((zero, current_block))
        }
    }

    /// Emits a member access expression (e.g. `tx.origin`, `msg.sender`).
    ///
    // TODO: detect built-in member accesses (e.g. `tx.origin`, `msg.sender`)
    // via Slang's `Typing::BuiltIn` variants once exposed in the semi-public
    // API, instead of matching on string names.
    pub fn emit_member_access(
        &self,
        operand: &Expression,
        member: &str,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        if let Expression::Identifier(identifier) = operand {
            let object = identifier.name();
            let intrinsic = match (object.as_str(), member) {
                ("tx", "origin") => solx_mlir::Builder::EVM_ORIGIN,
                ("tx", "gasprice") => solx_mlir::Builder::EVM_GASPRICE,
                ("msg", "sender") => solx_mlir::Builder::EVM_CALLER,
                ("msg", "value") => solx_mlir::Builder::EVM_CALLVALUE,
                ("block", "timestamp") => solx_mlir::Builder::EVM_TIMESTAMP,
                ("block", "number") => solx_mlir::Builder::EVM_NUMBER,
                ("block", "coinbase") => solx_mlir::Builder::EVM_COINBASE,
                ("block", "chainid") => solx_mlir::Builder::EVM_CHAINID,
                ("block", "basefee") => solx_mlir::Builder::EVM_BASEFEE,
                ("block", "gaslimit") => solx_mlir::Builder::EVM_GASLIMIT,
                _ => anyhow::bail!("unsupported member access: {object}.{member}"),
            };
            let value = self
                .expression_emitter
                .emit_intrinsic_call(intrinsic, &[], true, &block)?
                .expect("intrinsic always produces one result");
            return Ok((value, block));
        }
        anyhow::bail!("unsupported member access on non-identifier operand")
    }

    /// Emits a `require(condition)` built-in: revert with empty data when false.
    fn emit_require(
        &self,
        condition: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (condition_value, block) = self.expression_emitter.emit(condition, block)?;
        let condition_boolean = self
            .expression_emitter
            .emit_is_nonzero(condition_value, &block);

        let revert_block = self.expression_emitter.region.append_block(Block::new(&[]));
        let continue_block = self.expression_emitter.region.append_block(Block::new(&[]));

        block.append_operation(self.expression_emitter.state.builder().llvm_cond_br(
            condition_boolean,
            &continue_block,
            &revert_block,
            &[],
            &[],
        ));

        // Revert with empty error data via sol.revert + unreachable terminator.
        self.expression_emitter
            .state
            .builder()
            .emit_sol_revert(&revert_block);
        revert_block.append_operation(melior::dialect::llvm::unreachable(
            self.expression_emitter.state.location(),
        ));

        let result = self
            .expression_emitter
            .state
            .builder()
            .emit_sol_constant(0, &continue_block);
        Ok((result, continue_block))
    }
}
