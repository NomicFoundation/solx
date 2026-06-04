//!
//! Assignable locations (lvalues) and their loads and stores.
//!
//! Shared by assignment, compound assignment, and the increment/decrement
//! operators — every read-modify-write of a named location resolves through
//! here.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::Identifier;

use crate::ast::contract::function::storage_slot::StorageSlot;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

/// A resolved assignable location.
pub enum Lvalue<'context, 'block> {
    /// A stack slot — a local variable or parameter.
    Stack(Value<'context, 'block>, Type<'context>),
    /// A value-typed state variable storage slot.
    Storage(StorageSlot, Type<'context>),
    /// A materialized element address — a struct field or array element,
    /// already resolved to a pointer (the address-producing ops have run).
    Pointer(Value<'context, 'block>, Type<'context>),
    /// A reference-typed state variable (storage struct / array / `bytes` /
    /// `string`). The storage reference is materialized lazily at load/store;
    /// a store copies the whole aggregate in (`sol.copy`) rather than writing a
    /// scalar with `sol.store`.
    StorageReference(StorageSlot, Type<'context>),
}

impl<'context, 'block> Lvalue<'context, 'block> {
    /// The declared element type of the location.
    pub fn element_type(&self) -> Type<'context> {
        match self {
            Self::Stack(_, element_type)
            | Self::Storage(_, element_type)
            | Self::Pointer(_, element_type)
            | Self::StorageReference(_, element_type) => *element_type,
        }
    }
}

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Resolves an expression to the location it denotes for assignment,
    /// emitting any address-producing ops (`sol.gep` / `sol.map`) into `block`.
    ///
    /// Identifier targets (locals, parameters, value- and reference-typed state
    /// variables), struct-field member accesses, and array / mapping index
    /// accesses are supported.
    pub fn resolve_lvalue(
        &self,
        expression: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Lvalue<'context, 'block>, BlockRef<'context, 'block>)> {
        match expression {
            Expression::Identifier(identifier) => {
                Ok((self.resolve_identifier_lvalue(identifier), block))
            }
            Expression::MemberAccessExpression(access) => {
                let (address, element_type, block) = self
                    .emit_struct_field_address(access, block)?
                    .expect("a member-access lvalue addresses a struct field");
                Ok((Lvalue::Pointer(address, element_type), block))
            }
            Expression::IndexAccessExpression(index_access) => {
                let (address, element_type, block) =
                    self.emit_index_access_address(index_access, block)?;
                Ok((Lvalue::Pointer(address, element_type), block))
            }
            _ => unimplemented!("lvalue: {:?}", std::mem::discriminant(expression)),
        }
    }

    /// Resolves an identifier to the stack slot or storage slot it names.
    fn resolve_identifier_lvalue(&self, identifier: &Identifier) -> Lvalue<'context, 'block> {
        match identifier.resolve_to_definition() {
            Some(Definition::Variable(_) | Definition::Parameter(_)) => {
                let (pointer, element_type) =
                    self.environment.variable_with_type(&identifier.name());
                Lvalue::Stack(pointer, element_type)
            }
            Some(Definition::StateVariable(state_variable)) => {
                let declared_type = state_variable
                    .get_type()
                    .expect("binder types every state variable");
                let slot = self
                    .storage_layout
                    .get(&state_variable.node_id())
                    .expect("every state variable has a storage slot")
                    .clone();
                let element_type =
                    TypeConversion::resolve_slang_type(&declared_type, None, &self.state.builder);
                if declared_type.is_reference_type() {
                    Lvalue::StorageReference(slot, element_type)
                } else {
                    Lvalue::Storage(slot, element_type)
                }
            }
            Some(_) => unimplemented!("lvalue binding kind: {}", identifier.name()),
            None => unreachable!("slang resolves every identifier reference"),
        }
    }

    /// Loads the value currently held at an lvalue.
    pub fn emit_lvalue_load(
        &self,
        lvalue: &Lvalue<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<Value<'context, 'block>> {
        match lvalue {
            Lvalue::Stack(pointer, element_type) => {
                self.state
                    .builder
                    .emit_sol_load(*pointer, *element_type, block)
            }
            Lvalue::Storage(slot, element_type) => {
                self.emit_storage_load(slot, *element_type, block)
            }
            Lvalue::Pointer(pointer, element_type) => {
                self.state
                    .builder
                    .emit_sol_load(*pointer, *element_type, block)
            }
            Lvalue::StorageReference(slot, element_type) => Ok(self
                .state
                .builder
                .emit_sol_addr_of(&slot.name, *element_type, block)),
        }
    }

    /// Stores a value to an lvalue.
    pub fn emit_lvalue_store(
        &self,
        lvalue: &Lvalue<'context, 'block>,
        value: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) {
        match lvalue {
            Lvalue::Stack(pointer, _) => self.state.builder.emit_sol_store(value, *pointer, block),
            Lvalue::Storage(slot, element_type) => {
                self.emit_storage_store(slot, value, *element_type, block);
            }
            Lvalue::Pointer(pointer, _) => {
                self.state.builder.emit_sol_store(value, *pointer, block);
            }
            Lvalue::StorageReference(slot, element_type) => {
                let reference =
                    self.state
                        .builder
                        .emit_sol_addr_of(&slot.name, *element_type, block);
                self.state.builder.emit_sol_copy(value, reference, block);
            }
        }
    }
}
