//!
//! Member access expressions: struct fields and the environment intrinsics.
//!

use num::BigInt;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::Type;

use solx_mlir::Place;
use solx_mlir::Type as MlirType;
use solx_mlir::Value;
use solx_utils::FunctionReferenceKind;

use crate::scope::function::FunctionScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// A struct field loads from its place; an enum member is its ordinal; an externally visible
    /// function reached through a contract instance or `this` is the pointer dispatching it; every
    /// other member access is an environment or EVM intrinsic.
    pub fn member_access(&mut self, node: &MemberAccessExpression) -> Value<'context> {
        let operand = node.operand();

        if matches!(operand.get_type(), Some(Type::Struct(_))) {
            let (place, element_type) = self.member_access_place(node);
            return place.load(element_type, self);
        }

        if let Some(Definition::EnumMember(member)) = node.member().resolve_to_definition() {
            let Some(Definition::Enum(enum_definition)) = member.enclosing_definition() else {
                unreachable!("an enum member is declared by an enum");
            };
            let ordinal = enum_definition
                .members()
                .iter()
                .position(|candidate| candidate.node_id() == member.node_id())
                .expect("an enum lists the members it declares");
            let enum_type = self.typing(node.get_type());
            return Value::constant_from_bigint(&BigInt::from(ordinal), enum_type, self);
        }

        if let Some(Type::Function(function_type)) = node.get_type()
            && FunctionReferenceKind::from(function_type.visibility())
                == FunctionReferenceKind::External
        {
            let Some(Definition::Function(function_definition)) =
                function_type.associated_definition()
            else {
                unreachable!("an external function type names the function it dispatches");
            };
            let address = self.converted(&operand, MlirType::address(self.melior, false));
            return Value::external_function_constant(
                address,
                function_definition
                    .compute_selector()
                    .expect("an externally visible function has a selector"),
                self.typing(node.get_type()),
                self,
            );
        }

        match node.member().resolve_to_built_in() {
            Some(BuiltIn::AddressBalance) => {
                let address = self.converted(&operand, MlirType::address(self.melior, false));
                Value::balance(address, self)
            }
            Some(BuiltIn::AddressCodehash) => {
                let address = self.converted(&operand, MlirType::address(self.melior, false));
                Value::code_hash(address, self)
            }
            Some(BuiltIn::AddressCode) => {
                let address = self.converted(&operand, MlirType::address(self.melior, false));
                Value::code(address, self)
            }
            Some(BuiltIn::Length) => self.expression(&operand).length(self),
            Some(BuiltIn::FunctionSelector) => self.external_selector(&operand),
            Some(BuiltIn::FunctionAddress) => {
                self.expression(&operand).external_function_address(self)
            }
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
            Some(BuiltIn::TypeEnumMin) => Value::zero(self.typing(node.get_type()), self),
            Some(BuiltIn::TypeEnumMax) => {
                let enum_type = self.typing(node.get_type());
                Value::constant_from_bigint(&BigInt::from(enum_type.enum_max()), enum_type, self)
            }
            _ => unimplemented!("unsupported member access: {}", node.member().name()),
        }
    }

    /// The address yielded by `s.field` together with the field's element MLIR type.
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

        let Some(Definition::StructMember(member)) = node.member().resolve_to_definition() else {
            unreachable!("a struct field access binds to the member it names");
        };
        let field_index = struct_definition
            .members()
            .iter()
            .position(|candidate| candidate.node_id() == member.node_id())
            .expect("a struct lists the members it declares");

        let base_value = self.expression(&base);
        let element_type = base_value.r#type().element_type(field_index as u64);
        (
            Place::from(base_value).gep_field(field_index, element_type, self),
            element_type,
        )
    }

    /// The ABI selector `.selector` yields: an operand naming a function folds to its selector
    /// constant, while a pointer value carries its own.
    pub fn external_selector(&mut self, operand: &Expression) -> Value<'context> {
        if let Expression::MemberAccessExpression(access) = operand
            && let Some(Definition::Function(function_definition)) =
                access.member().resolve_to_definition()
        {
            return Value::selector(
                function_definition
                    .compute_selector()
                    .expect("an externally visible function has a selector"),
                MlirType::selector(self.melior),
                self,
            );
        }
        self.expression(operand).external_function_selector(self)
    }
}
