//!
//! The external ABI signature of a `public` state variable's synthesised getter.
//!

use melior::ir::Type;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::StateVariableDefinition;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::Context;

use crate::ast::LocationPolicy;
use crate::ast::Type as AstType;
use crate::ast::contract::getter::keyed_signature::KeyedSignature;
use crate::ast::contract::getter::member::Member;

/// The external ABI signature of a `public` state variable's synthesised getter, shared by the
/// call-position getter, the function-pointer value, and the published method identifiers.
pub trait Signature {
    /// `(parameter_types, return_types)`: scalar `() -> (T)`, mapping `(K) -> (V)`, array
    /// `(uint256) -> (element)`, struct `() -> (members)`; `None` with no flattenable result.
    fn getter_signature<'context>(
        &self,
        context: &Context<'context>,
    ) -> Option<(Vec<Type<'context>>, Vec<Type<'context>>)>;

    /// The multi-level keyed signature, shared with the getter body so the two never disagree;
    /// `None` for a non-keyed type or a leaf with no flattenable getter.
    fn keyed_signature<'context>(
        &self,
        location: solx_utils::DataLocation,
        context: &Context<'context>,
    ) -> Option<KeyedSignature<'context>>;
}

impl Signature for StateVariableDefinition {
    fn getter_signature<'context>(
        &self,
        context: &Context<'context>,
    ) -> Option<(Vec<Type<'context>>, Vec<Type<'context>>)> {
        self.get_type()
            .and_then(|declared_type| match &declared_type {
                SlangType::Mapping(_) | SlangType::Array(_) | SlangType::FixedSizeArray(_) => self
                    .keyed_signature(solx_utils::DataLocation::Storage, context)
                    .map(|signature| (signature.input_types, signature.result_types)),
                SlangType::Struct(struct_type) => {
                    let Definition::Struct(struct_definition) = struct_type.definition() else {
                        return None;
                    };
                    let struct_mlir_type = AstType::resolve(
                        &declared_type,
                        LocationPolicy::Declared(Some(solx_utils::DataLocation::Storage)),
                        context,
                    );
                    let members = Member::layout(&struct_definition, struct_mlir_type, context)?;
                    let return_types = members.iter().map(|member| member.result_type).collect();
                    Some((Vec::new(), return_types))
                }
                other if other.is_reference_type() => Some((
                    Vec::new(),
                    vec![AstType::resolve(
                        other,
                        LocationPolicy::ForceMemory,
                        context,
                    )],
                )),
                other => Some((
                    Vec::new(),
                    vec![AstType::resolve(
                        other,
                        LocationPolicy::Declared(None),
                        context,
                    )],
                )),
            })
    }

    fn keyed_signature<'context>(
        &self,
        location: solx_utils::DataLocation,
        context: &Context<'context>,
    ) -> Option<KeyedSignature<'context>> {
        let mut input_types: Vec<Type<'context>> = Vec::new();
        let mut terminal = self.get_type()?;
        loop {
            match &terminal {
                SlangType::Mapping(mapping_type) => {
                    let key = mapping_type.key_type();
                    let key_type = if key.is_reference_type() {
                        AstType::string(context.mlir_context, solx_utils::DataLocation::Memory)
                            .into_mlir()
                    } else {
                        AstType::resolve(&key, LocationPolicy::Declared(Some(location)), context)
                    };
                    input_types.push(key_type);
                    terminal = mapping_type.value_type();
                }
                SlangType::Array(array_type) => {
                    input_types.push(
                        AstType::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_FIELD)
                            .into_mlir(),
                    );
                    terminal = array_type.element_type();
                }
                SlangType::FixedSizeArray(array_type) => {
                    input_types.push(
                        AstType::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_FIELD)
                            .into_mlir(),
                    );
                    terminal = array_type.element_type();
                }
                _ => break,
            }
        }
        if input_types.is_empty() {
            return None;
        }
        let terminal_is_reference = matches!(&terminal, SlangType::String(_) | SlangType::Bytes(_));
        let result_type = if terminal_is_reference {
            AstType::resolve(&terminal, LocationPolicy::ForceMemory, context)
        } else {
            AstType::resolve(&terminal, LocationPolicy::Declared(Some(location)), context)
        };
        let members = match &terminal {
            SlangType::Struct(struct_type) => {
                let Definition::Struct(struct_definition) = struct_type.definition() else {
                    return None;
                };
                Some(Member::layout(&struct_definition, result_type, context)?)
            }
            SlangType::String(_) | SlangType::Bytes(_) => None,
            _ => None,
        };
        let result_types: Vec<Type<'context>> = match &members {
            Some(members) => members.iter().map(|member| member.result_type).collect(),
            None => vec![result_type],
        };
        Some(KeyedSignature {
            input_types,
            result_types,
            members,
            terminal_is_reference,
        })
    }
}
