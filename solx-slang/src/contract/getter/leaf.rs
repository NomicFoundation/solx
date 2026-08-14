//!
//! The read at the bottom of a getter's storage walk.
//!

use slang_solidity_v2::ast::StructDefinition;
use slang_solidity_v2::ast::StructMember;

use solx_mlir::Context;
use solx_mlir::Place;
use solx_mlir::Type as MlirType;
use solx_mlir::Value;

/// The read at the bottom of the getter's storage walk.
pub enum Leaf<'context> {
    /// A single value, loaded at its stored type.
    Value(MlirType<'context>),
    /// The declared indices of a struct's returnable members; skipped members leave gaps, never
    /// compaction.
    Members(Vec<usize>),
}

impl<'context> Leaf<'context> {
    /// The declared positions of `returned`, the members slang's getter type is built from.
    pub fn members(struct_definition: &StructDefinition, returned: &[StructMember]) -> Self {
        let declared = struct_definition.members();
        Self::Members(
            returned
                .iter()
                .map(|member| {
                    declared
                        .iter()
                        .position(|candidate| candidate.node_id() == member.node_id())
                        .expect("a struct lists the members it declares")
                })
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
