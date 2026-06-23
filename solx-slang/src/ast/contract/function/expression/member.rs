//!
//! Member access expression emission: `base.member` — namespace-qualified reads, struct fields, and built-ins.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Value as MlirValue;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::Type as SlangType;
use solx_mlir::ods::sol::BalanceOperation;
use solx_mlir::ods::sol::BaseFeeOperation;
use solx_mlir::ods::sol::BlobBaseFeeOperation;
use solx_mlir::ods::sol::BlockNumberOperation;
use solx_mlir::ods::sol::CallValueOperation;
use solx_mlir::ods::sol::CallerOperation;
use solx_mlir::ods::sol::ChainIdOperation;
use solx_mlir::ods::sol::CodeHashOperation;
use solx_mlir::ods::sol::CodeOperation;
use solx_mlir::ods::sol::CoinbaseOperation;
use solx_mlir::ods::sol::DifficultyOperation;
use solx_mlir::ods::sol::GasLimitOperation;
use solx_mlir::ods::sol::GasPriceOperation;
use solx_mlir::ods::sol::GetCallDataOperation;
use solx_mlir::ods::sol::OriginOperation;
use solx_mlir::ods::sol::PrevRandaoOperation;
use solx_mlir::ods::sol::SigOperation;
use solx_mlir::ods::sol::TimestampOperation;
use solx_utils::DataLocation;

use crate::ast::BlockAnd;
use crate::ast::EmitExpression;
use crate::ast::EmitPlace;
use crate::ast::Place;
use crate::ast::Pointer;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
use crate::ast::contract::function::expression::ExpressionContext;

impl<'context: 'block, 'block> EmitPlace<'context, 'block> for MemberAccessExpression {
    /// Emits the address `s.field` denotes with the field's element type (`sol.gep`), without the load. Struct base only.
    fn emit_place<'state>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Place<'context, 'block>> {
        let base = self.operand();
        let Some(SlangType::Struct(struct_type)) = base.get_type() else {
            unreachable!("a struct-field address is only emitted for a struct base");
        };
        let Definition::Struct(struct_definition) = struct_type.definition() else {
            unreachable!("slang StructType always references a Struct definition");
        };

        // Locate the accessed field by node-id identity — slang exposes fields as an ordered list
        // with no direct index lookup, and the binder resolves the access (no name comparison).
        let Some(Definition::StructMember(member_definition)) =
            self.member().resolve_to_definition()
        else {
            unreachable!("slang resolves a struct field access to its StructMember definition");
        };
        let member_id = member_definition.node_id();
        let field_index = struct_definition
            .members()
            .iter()
            .position(|member| member.node_id() == member_id)
            .expect("slang validated");

        let BlockAnd {
            value: base_value,
            block,
        } = base.emit(context, block);
        let builder = &context.state.builder;

        let index_value = AstValue::constant(
            field_index as i64,
            AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_X64),
            builder,
            &block,
        );
        let element_type = base_value.r#type().element_type(field_index);
        let address = base_value
            .into_pointer()
            .gep(index_value, element_type, builder, &block)
            .into_mlir();
        BlockAnd {
            value: Place {
                address,
                element_type: element_type.into_mlir(),
            },
            block,
        }
    }
}

expression_emit!(MemberAccessExpression; |node, context, block| {
    // `address.balance`/`codehash`/`code`, `arr.length`: a unary intrinsic
    // over the receiver value.
    match node.member().resolve_to_built_in() {
        Some(BuiltIn::AddressBalance) => {
            let BlockAnd {
                value: address,
                block,
            } = node.operand().emit(context, block);
            let builder = &context.state.builder;
            let value: MlirValue<'context, 'block> = mlir_op!(
                builder,
                &block,
                BalanceOperation
                    .cont_addr(address.into_mlir())
                    .out(AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD))
            );
            return BlockAnd {
                block,
                value: value.into(),
            };
        }
        Some(BuiltIn::AddressCodehash) => {
            let BlockAnd {
                value: address,
                block,
            } = node.operand().emit(context, block);
            let builder = &context.state.builder;
            let value: MlirValue<'context, 'block> = mlir_op!(
                builder,
                &block,
                CodeHashOperation
                    .cont_addr(address.into_mlir())
                    .out(AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD))
            );
            return BlockAnd {
                block,
                value: value.into(),
            };
        }
        Some(BuiltIn::AddressCode) => {
            let BlockAnd {
                value: address,
                block,
            } = node.operand().emit(context, block);
            let builder = &context.state.builder;
            let value: MlirValue<'context, 'block> = mlir_op!(
                builder,
                &block,
                CodeOperation
                    .cont_addr(address.into_mlir())
                    .out(AstType::string(builder.context, DataLocation::Memory))
            );
            return BlockAnd {
                block,
                value: value.into(),
            };
        }
        Some(BuiltIn::Length) => {
            let BlockAnd {
                value: operand,
                block,
            } = node.operand().emit(context, block);
            return BlockAnd {
                value: operand.length(&context.state.builder, &block),
                block,
            };
        }
        _ => {}
    }
    // EVM environment globals (`block`/`tx`/`msg`): nullary `sol.*` intrinsics, built then appended once.
    let builder = &context.state.builder;
    let environment_op = match node.member().resolve_to_built_in() {
        Some(BuiltIn::TxOrigin) => {
            Some(mlir_op_build!(builder, OriginOperation.addr(AstType::address(builder.context, false))))
        }
        Some(BuiltIn::TxGasPrice) => Some(mlir_op_build!(
            builder,
            GasPriceOperation.val(AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD))
        )),
        Some(BuiltIn::MsgSender) => {
            Some(mlir_op_build!(builder, CallerOperation.addr(AstType::address(builder.context, false))))
        }
        Some(BuiltIn::MsgValue) => Some(mlir_op_build!(
            builder,
            CallValueOperation.val(AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD))
        )),
        Some(BuiltIn::BlockTimestamp) => Some(mlir_op_build!(
            builder,
            TimestampOperation.val(AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD))
        )),
        Some(BuiltIn::BlockNumber) => Some(mlir_op_build!(
            builder,
            BlockNumberOperation.val(AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD))
        )),
        Some(BuiltIn::BlockCoinbase) => {
            Some(mlir_op_build!(builder, CoinbaseOperation.addr(AstType::address(builder.context, false))))
        }
        Some(BuiltIn::BlockChainid) => Some(mlir_op_build!(
            builder,
            ChainIdOperation.val(AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD))
        )),
        Some(BuiltIn::BlockBasefee) => Some(mlir_op_build!(
            builder,
            BaseFeeOperation.val(AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD))
        )),
        Some(BuiltIn::BlockGaslimit) => Some(mlir_op_build!(
            builder,
            GasLimitOperation.val(AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD))
        )),
        Some(BuiltIn::BlockBlobbasefee) => Some(mlir_op_build!(
            builder,
            BlobBaseFeeOperation.val(AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD))
        )),
        Some(BuiltIn::BlockDifficulty) => Some(mlir_op_build!(
            builder,
            DifficultyOperation.val(AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD))
        )),
        Some(BuiltIn::BlockPrevrandao) => Some(mlir_op_build!(
            builder,
            PrevRandaoOperation.val(AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD))
        )),
        Some(BuiltIn::MsgSig) => Some(mlir_op_build!(
            builder,
            SigOperation.val(AstType::fixed_bytes(builder.context, 4))
        )),
        Some(BuiltIn::MsgData) => Some(mlir_op_build!(
            builder,
            GetCallDataOperation.addr(AstType::string(builder.context, DataLocation::CallData))
        )),
        _ => None,
    };
    if let Some(operation) = environment_op {
        let value: MlirValue<'context, 'block> = block
            .append_operation(operation)
            .result(0)
            .expect("an environment global produces one result")
            .into();
        return BlockAnd {
            block,
            value: value.into(),
        };
    }
    // A struct-typed base is a field read (`s.field`); anything else is unsupported.
    if matches!(node.operand().get_type(), Some(SlangType::Struct(_))) {
        // Address the field (`sol.gep`) and `sol.load` it.
        let BlockAnd {
            value: Place {
                address,
                element_type,
            },
            block,
        } = node.emit_place(context, block);
        let value = Pointer::new(address).load(
            AstType::new(element_type),
            &context.state.builder,
            &block,
        );
        BlockAnd { block, value }
    } else {
        unimplemented!("unsupported member access: {}", node.member().name());
    }
});
