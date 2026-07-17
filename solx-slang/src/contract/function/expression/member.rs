//!
//! Member access expressions: struct fields and the environment intrinsics.
//!

use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::Type;

use solx_mlir::Place;
use solx_mlir::Type as MlirType;
use solx_mlir::Value;

use crate::scope::function::FunctionScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// A struct field loads from its place; every other member access is an environment or EVM
    /// intrinsic.
    pub fn member_access(&mut self, node: &MemberAccessExpression) -> Value<'context> {
        if matches!(node.operand().get_type(), Some(Type::Struct(_))) {
            let (place, element_type) = self.member_access_place(node);
            return place.load(element_type, self);
        }
        match node.member().resolve_to_built_in() {
            Some(BuiltIn::AddressBalance) => Value::balance(self.expression(&node.operand()), self),
            Some(BuiltIn::AddressCodehash) => {
                Value::code_hash(self.expression(&node.operand()), self)
            }
            Some(BuiltIn::AddressCode) => Value::code(self.expression(&node.operand()), self),
            Some(BuiltIn::Length) => self.expression(&node.operand()).length(self),
            Some(BuiltIn::TxOrigin) => Value::tx_origin(self),
            Some(BuiltIn::TxGasPrice) => Value::tx_gas_price(self),
            Some(BuiltIn::MsgSender) => Value::msg_sender(self),
            Some(BuiltIn::MsgValue) => Value::msg_value(self),
            Some(BuiltIn::MsgSig) => Value::msg_sig(self),
            Some(BuiltIn::MsgData) => Value::msg_data(self),
            Some(BuiltIn::BlockTimestamp) => Value::block_timestamp(self),
            Some(BuiltIn::BlockNumber) => Value::block_number(self),
            Some(BuiltIn::BlockCoinbase) => Value::block_coinbase(self),
            Some(BuiltIn::BlockChainid) => Value::block_chain_id(self),
            Some(BuiltIn::BlockBasefee) => Value::block_base_fee(self),
            Some(BuiltIn::BlockGaslimit) => Value::block_gas_limit(self),
            Some(BuiltIn::BlockBlobbasefee) => Value::block_blob_base_fee(self),
            Some(BuiltIn::BlockDifficulty) => Value::block_difficulty(self),
            Some(BuiltIn::BlockPrevrandao) => Value::block_prev_randao(self),
            _ => unimplemented!("unsupported member access: {}", node.member().name()),
        }
    }

    /// The address yielded by `s.field` together with the field's element MLIR type. The field index
    /// is derived by member-name comparison until node-id resolution is verified against the corpus.
    pub fn member_access_place(
        &mut self,
        node: &MemberAccessExpression,
    ) -> (Place<'context>, MlirType<'context>) {
        let base = node.operand();
        let Some(Type::Struct(struct_type)) = base.get_type() else {
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

        let base_value = self.expression(&base);
        let element_type = base_value.r#type().element_type(field_index as u64);
        (
            Place::from(base_value).gep_field(field_index, element_type, self),
            element_type,
        )
    }
}
