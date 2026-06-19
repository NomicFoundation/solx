//!
//! Public state-variable getter ABI-signature resolution.
//!

use melior::ir::Type;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::StateVariableDefinition;
use slang_solidity_v2::ast::Type as SlangType;
use solx_utils::DataLocation;

use crate::ast::LocationPolicy;
use crate::ast::Type as AstType;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::getter::StructGetterLayout;

impl<'state, 'context, 'block> ExpressionContext<'state, 'context, 'block> {
    /// The ABI signature of a `public` state variable's synthesised getter:
    /// the key/index parameter types and the returned value types.
    ///
    /// A scalar variable `T public x` is `() -> (T)`; a mapping `mapping(K => V)`
    /// is `(K) -> (V)`; an array is `(uint256) -> (element)`; a struct is
    /// `() -> (flattened returnable members)` (sharing the synthesised getter's
    /// member layout, so the call decodes exactly what the getter returns).
    /// Single-level only — a nested or reference-typed key / value / element
    /// returns `None`, a LOUD residual at the (already-classified) call site.
    pub fn getter_signature(
        &self,
        state_variable: &StateVariableDefinition,
    ) -> Option<(Vec<Type<'context>>, Vec<Type<'context>>)> {
        let declared_type = state_variable.get_type()?;
        let builder = &self.state.builder;
        match &declared_type {
            SlangType::Mapping(mapping_type) => {
                let key = mapping_type.key_type();
                let value = mapping_type.value_type();
                if key.is_reference_type() || value.is_reference_type() {
                    return None;
                }
                Some((
                    vec![AstType::resolve(
                        &key,
                        LocationPolicy::Declared(None),
                        builder,
                    )],
                    vec![AstType::resolve(
                        &value,
                        LocationPolicy::Declared(None),
                        builder,
                    )],
                ))
            }
            SlangType::Array(array_type) => {
                let element = array_type.element_type();
                if element.is_reference_type() {
                    return None;
                }
                Some((
                    vec![
                        AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD)
                            .into_mlir(),
                    ],
                    vec![AstType::resolve(
                        &element,
                        LocationPolicy::Declared(None),
                        builder,
                    )],
                ))
            }
            SlangType::FixedSizeArray(array_type) => {
                let element = array_type.element_type();
                if element.is_reference_type() {
                    return None;
                }
                Some((
                    vec![
                        AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD)
                            .into_mlir(),
                    ],
                    vec![AstType::resolve(
                        &element,
                        LocationPolicy::Declared(None),
                        builder,
                    )],
                ))
            }
            SlangType::Struct(struct_type) => {
                let Definition::Struct(struct_definition) = struct_type.definition() else {
                    return None;
                };
                let struct_mlir_type = AstType::resolve(
                    &declared_type,
                    LocationPolicy::Declared(Some(DataLocation::Storage)),
                    builder,
                );
                let plan = struct_definition.struct_getter_layout(struct_mlir_type, builder)?;
                let return_types = plan
                    .iter()
                    .map(|(_, _, result_type)| *result_type)
                    .collect();
                Some((Vec::new(), return_types))
            }
            other if !other.is_reference_type() => Some((
                Vec::new(),
                vec![AstType::resolve(
                    other,
                    LocationPolicy::Declared(None),
                    builder,
                )],
            )),
            _ => None,
        }
    }
}
