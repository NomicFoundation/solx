//!
//! A returnable member of a struct getter.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::StructDefinition;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::Context;
use solx_mlir::LocationPolicy;
use solx_mlir::Pointer;
use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;

/// A returnable member of a struct getter. A scalar member loads its value; a reference member is
/// returned as its reinterpreted address, the two told apart by whether `stored_type` equals
/// `result_type`.
pub struct Member<'context> {
    /// The `sol.gep` index of the member within the struct place.
    pub index: u64,
    /// The member's type as laid out in storage.
    pub stored_type: Type<'context>,
    /// The type the member is returned as: equal to `stored_type` for a scalar, the memory
    /// reference type for a reference member.
    pub result_type: Type<'context>,
}

impl<'context> Member<'context> {
    /// The returnable members of `struct_definition`, or `None` when none is returnable. Mappings
    /// and arrays are skipped; a nested struct returns whole as its memory ABI tuple.
    pub fn layout(
        struct_definition: &StructDefinition,
        struct_mlir_type: Type<'context>,
        context: &Context<'context>,
    ) -> Option<Vec<Self>> {
        let mut members = Vec::new();
        for (member_index, member) in struct_definition.members().iter().enumerate() {
            let member_slang = member.get_type().expect("slang validated");
            let stored_type = AstType::new(struct_mlir_type)
                .element_type(member_index)
                .into_mlir();
            let result_type = match &member_slang {
                SlangType::Mapping(_) | SlangType::Array(_) | SlangType::FixedSizeArray(_) => {
                    continue;
                }
                SlangType::String(_) | SlangType::Bytes(_) => {
                    AstType::string(context.mlir_context, solx_utils::DataLocation::Memory)
                        .into_mlir()
                }
                SlangType::Struct(_) => {
                    AstType::resolve(&member_slang, LocationPolicy::ForceMemory, context)
                }
                _ => stored_type,
            };
            members.push(Self {
                index: member_index as u64,
                stored_type,
                result_type,
            });
        }
        if members.is_empty() {
            return None;
        }
        Some(members)
    }

    /// Loads this member from the struct place `base`.
    pub fn load_from<'block>(
        &self,
        base: Value<'context, 'block>,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let index = AstValue::constant(
            self.index as i64,
            AstType::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_X64),
            context,
            block,
        );
        let address = Pointer::new(base)
            .gep(index, AstType::new(self.stored_type), false, context, block)
            .into_mlir();
        if self.stored_type == self.result_type {
            Pointer::new(address)
                .load(AstType::new(self.result_type), context, block)
                .into_mlir()
        } else {
            AstValue::new(address)
                .cast(AstType::new(self.result_type), context, block)
                .into_mlir()
        }
    }
}
