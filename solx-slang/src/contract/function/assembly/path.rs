//!
//! Yul path resolution: the Yul variables an assembly block declares and the Solidity declarations
//! it reaches through the `sol.yul_*` bridge ops.
//!

use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::Identifier;
use slang_solidity_v2::ast::Type;
use slang_solidity_v2::ast::YulPath;

use solx_mlir::Word;
use solx_utils::DataLocation;

use crate::contract::function::assembly::reference::YulField;
use crate::contract::function::assembly::reference::YulReference;
use crate::scope::assembly::AssemblyScope;
use crate::scope::function::FunctionScope;
use crate::scope::source_unit::SourceUnitScope;

impl<'function, 'contract, 'source_unit, 'context>
    AssemblyScope<'function, 'contract, 'source_unit, 'context>
{
    /// What `path` denotes. A bare name is either a Yul variable's own pointer or a Solidity local
    /// reinterpreted as one; a suffixed name projects the field of the Solidity declaration the
    /// suffix asks for, which for a state variable is a compile-time word rather than a pointer.
    pub fn reference(&mut self, path: &YulPath) -> YulReference<'context> {
        let mut identifiers = path.iter();
        let base = identifiers
            .next()
            .expect("a Yul path names at least one identifier");
        let suffix = identifiers.next().map(|member| {
            YulField::from(
                member
                    .resolve_to_built_in()
                    .expect("a suffixed Yul path ends in a built-in field"),
            )
        });

        let definition = base.resolve_to_definition();
        if let Some(definition) = &definition
            && let Some((initializer, declared_type)) =
                FunctionScope::constant_definition(definition)
        {
            return self.constant_reference(&initializer, declared_type);
        }
        match definition {
            Some(Definition::YulVariable(declaration) | Definition::YulParameter(declaration)) => {
                YulReference::Pointer(self.variable(declaration.node_id()))
            }
            Some(Definition::StateVariable(state_variable)) => {
                let symbol = SourceUnitScope::state_variable_symbol(&state_variable);
                YulReference::Word(match suffix {
                    Some(YulField::Slot) => Word::state_variable_slot(symbol.as_str(), self),
                    Some(YulField::Offset) => Word::state_variable_offset(symbol.as_str(), self),
                    _ => unreachable!(
                        "assembly reaches a state variable only through .slot / .offset"
                    ),
                })
            }
            Some(Definition::Variable(_) | Definition::Parameter(_)) => {
                self.local_reference(&base, suffix)
            }
            _ => unreachable!(
                "a Yul path names a variable or a state variable: {}",
                base.name()
            ),
        }
    }

    /// The field of a Solidity local `base` that `suffix` projects, bridged into Yul. A bare local
    /// is its stack pointer reinterpreted as a Yul one; `.offset` reads the calldata offset of a
    /// calldata reference and the byte offset within a slot of a storage one.
    fn local_reference(
        &mut self,
        base: &Identifier,
        suffix: Option<YulField>,
    ) -> YulReference<'context> {
        let (place, element_type) = self.function.identifier_place(base);
        match suffix {
            Some(YulField::Slot) => YulReference::Pointer(place.yul_storage_slot(self)),
            Some(YulField::Offset) => {
                if element_type.data_location() == DataLocation::CallData {
                    YulReference::Pointer(place.yul_calldata_offset(self))
                } else {
                    YulReference::Word(place.yul_storage_offset(self))
                }
            }
            Some(YulField::Length) => YulReference::Pointer(place.yul_calldata_length(self)),
            Some(YulField::Selector) => YulReference::Pointer(place.yul_selector(self)),
            Some(YulField::Address) => YulReference::Pointer(place.yul_function_address(self)),
            None => YulReference::Pointer(place.yul_pointer(self)),
        }
    }

    /// The word a Solidity constant's initializer denotes. The initializer is folded at the
    /// constant's declared type before being reinterpreted, so a narrow or signed constant reaches
    /// Yul sign-extended and cleaned up rather than as the literal's own type.
    fn constant_reference(
        &mut self,
        initializer: &Expression,
        declared_type: Option<Type>,
    ) -> YulReference<'context> {
        let declared_type = self.function.typing(declared_type);
        let value = self.function.converted(initializer, declared_type);
        YulReference::Word(value.yul_word(self))
    }
}
