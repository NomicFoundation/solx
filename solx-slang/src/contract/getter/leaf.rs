//!
//! The read at the bottom of a getter's storage walk.
//!

use slang_solidity_v2::ast::StructDefinition;
use slang_solidity_v2::ast::Type;

use solx_mlir::Context;
use solx_mlir::Place;
use solx_mlir::Type as MlirType;
use solx_mlir::Value;
use solx_utils::FunctionReferenceKind;

/// The read at the bottom of the getter's storage walk.
pub enum Leaf<'context> {
    /// A single value, loaded at its stored type.
    Value(MlirType<'context>),
    /// The declared indices of a struct's returnable members; skipped members leave gaps, never
    /// compaction.
    Members(Vec<usize>),
}

impl<'context> Leaf<'context> {
    /// The returnable members of a struct leaf: mappings and arrays are skipped, and a
    /// function-typed member is returnable only when external.
    pub fn members(struct_definition: &StructDefinition) -> Self {
        Self::Members(
            struct_definition
                .members()
                .iter()
                .enumerate()
                .filter(|(_, member)| {
                    match member
                        .get_type()
                        .expect("struct member type resolved by semantic analysis")
                    {
                        Type::Mapping(_) | Type::Array(_) | Type::FixedSizeArray(_) => false,
                        Type::Function(function_type) => {
                            FunctionReferenceKind::from(function_type.visibility())
                                == FunctionReferenceKind::External
                        }
                        _ => true,
                    }
                })
                .map(|(index, _)| index)
                .collect(),
        )
    }

    /// Reads the leaf place into the getter's return values, each converted to its result type.
    pub fn read(
        &self,
        place: Place<'context>,
        results: &[MlirType<'context>],
        context: &Context<'context>,
    ) -> Vec<Value<'context>> {
        match self {
            Self::Value(stored_type) => {
                vec![
                    place
                        .load(*stored_type, context)
                        .convert(results[0], context),
                ]
            }
            Self::Members(indices) => indices
                .iter()
                .zip(results)
                .map(|(&index, &result_type)| {
                    let element_type = place.r#type().element_type(index as u64);
                    place
                        .gep_field(index, element_type, context)
                        .load(element_type, context)
                        .convert(result_type, context)
                })
                .collect(),
        }
    }
}
