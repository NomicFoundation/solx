//!
//! Member access expressions: struct fields and the environment intrinsics.
//!

use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::Place;
use solx_mlir::Value;

use crate::contract::function::expression::Expression;

codegen!(
    MemberAccessExpression {
        /// A struct field loads from its place; every other member access is an environment or
        /// EVM intrinsic.
        -> Value |node, scope| {
            if matches!(node.operand().get_type(), Some(SlangType::Struct(_))) {
                let (place, element_type) = Self::emit_place(node, scope);
                return place.load(element_type, scope);
            }
            match node.member().resolve_to_built_in() {
                Some(BuiltIn::AddressBalance) => {
                    let address = Expression::emit(&node.operand(), scope);
                    Value::balance(address, scope)
                }
                Some(BuiltIn::AddressCodehash) => {
                    let address = Expression::emit(&node.operand(), scope);
                    Value::code_hash(address, scope)
                }
                Some(BuiltIn::AddressCode) => {
                    let address = Expression::emit(&node.operand(), scope);
                    Value::code(address, scope)
                }
                Some(BuiltIn::Length) => {
                    Expression::emit(&node.operand(), scope).length(scope)
                }
                Some(BuiltIn::TxOrigin) => Value::tx_origin(scope),
                Some(BuiltIn::TxGasPrice) => Value::tx_gas_price(scope),
                Some(BuiltIn::MsgSender) => Value::msg_sender(scope),
                Some(BuiltIn::MsgValue) => Value::msg_value(scope),
                Some(BuiltIn::MsgSig) => Value::msg_sig(scope),
                Some(BuiltIn::MsgData) => Value::msg_data(scope),
                Some(BuiltIn::BlockTimestamp) => Value::block_timestamp(scope),
                Some(BuiltIn::BlockNumber) => Value::block_number(scope),
                Some(BuiltIn::BlockCoinbase) => Value::block_coinbase(scope),
                Some(BuiltIn::BlockChainid) => Value::block_chain_id(scope),
                Some(BuiltIn::BlockBasefee) => Value::block_base_fee(scope),
                Some(BuiltIn::BlockGaslimit) => Value::block_gas_limit(scope),
                Some(BuiltIn::BlockBlobbasefee) => Value::block_blob_base_fee(scope),
                Some(BuiltIn::BlockDifficulty) => Value::block_difficulty(scope),
                Some(BuiltIn::BlockPrevrandao) => Value::block_prev_randao(scope),
                _ => unimplemented!("unsupported member access: {}", node.member().name()),
            }
        }

        /// The address yielded by `s.field` together with the field's element MLIR type. The
        /// field index is derived by member-name comparison until node-id resolution is verified
        /// against the corpus.
        -> Place |node, scope| {
            let base = node.operand();
            let Some(SlangType::Struct(struct_type)) = base.get_type() else {
                unreachable!("a member-access place always has a struct base");
            };
            let Definition::Struct(struct_definition) = struct_type.definition() else {
                unreachable!("slang StructType always references a Struct definition");
            };

            let member_name = node.member().name();
            let field_index = struct_definition
                .members()
                .iter()
                .position(|member| member.name().name() == member_name)
                .expect("slang validates the accessed member exists");

            let base_value = Expression::emit(&base, scope);
            let element_type = base_value.r#type().element_type(field_index as u64);
            let place = Place::from(base_value).gep_field(field_index, element_type, scope);
            (place, element_type)
        }
    }
);
