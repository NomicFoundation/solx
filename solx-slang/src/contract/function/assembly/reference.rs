//!
//! Yul path resolution: the Yul variables an assembly block declares and the Solidity declarations
//! it reaches through the `sol.yul_*` bridge ops.
//!

use ruint::aliases::U256;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::Identifier;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::StateVariableMutability;
use slang_solidity_v2::ast::Type;
use slang_solidity_v2::ast::YulPath;

use solx_mlir::Slot;
use solx_mlir::Word;
use solx_mlir::YulReference;
use solx_utils::DataLocation;

use crate::scope::assembly::AssemblyScope;

impl<'function, 'contract, 'source_unit, 'context>
    AssemblyScope<'function, 'contract, 'source_unit, 'context>
{
    /// Binds a Yul variable: allocates its slot, stores `value` into it, and records it under its
    /// declaring identifier. The value is materialized before the slot, which is the order the
    /// declaration reads in.
    pub fn bind(&mut self, declaration: NodeId, value: Word<'context>) -> Slot<'context> {
        let slot = Slot::alloca(self);
        slot.store(value, self);
        self.variables.insert(declaration, slot);
        slot
    }

    /// Binds a Yul variable whose slot precedes the zero stored into it: a function's return
    /// variable, which Yul initializes at entry rather than at a declaration site.
    pub fn bind_zero(&mut self, declaration: NodeId) -> Slot<'context> {
        let slot = Slot::alloca(self);
        let zero = self.word_zero();
        slot.store(zero, self);
        self.variables.insert(declaration, slot);
        slot
    }

    /// The `yul.constant 0` a Yul variable defaults to.
    pub fn word_zero(&mut self) -> Word<'context> {
        Word::constant(&U256::ZERO, self)
    }

    /// What `path` denotes. A bare name is either a Yul variable's own slot or a Solidity local
    /// reinterpreted as one; a suffixed name projects the field of the Solidity declaration the
    /// suffix asks for, which for a state variable is a compile-time word rather than a slot.
    pub fn yul_reference(&mut self, path: &YulPath) -> YulReference<'context> {
        let mut identifiers = path.iter();
        let base = identifiers
            .next()
            .expect("a Yul path names at least one identifier");
        let suffix = identifiers.next().map(|member| {
            let built_in = member
                .resolve_to_built_in()
                .expect("a suffixed Yul path ends in a built-in field");
            YulField::of(built_in).expect("a Yul field suffix is one of the five Solidity allows")
        });

        match base.resolve_to_definition() {
            Some(Definition::YulVariable(declaration) | Definition::YulParameter(declaration)) => {
                YulReference::Slot(self.variable(declaration.node_id()))
            }
            Some(Definition::Constant(constant)) => self.constant_word(
                &constant
                    .value()
                    .expect("a file-level constant is initialized"),
                constant.get_type(),
            ),
            Some(Definition::StateVariable(state_variable))
                if matches!(
                    state_variable.attributes().mutability(),
                    StateVariableMutability::Constant
                ) =>
            {
                self.constant_word(
                    &state_variable
                        .value()
                        .expect("a constant state variable is initialized"),
                    state_variable.get_type(),
                )
            }
            Some(Definition::StateVariable(state_variable)) => {
                let symbol = self
                    .function
                    .contract
                    .storage_layout
                    .get(&state_variable.node_id())
                    .expect("state variable is registered in the storage layout")
                    .name
                    .clone();
                YulReference::Word(match suffix {
                    Some(YulField::Slot) => Word::state_var_slot(symbol.as_str(), self),
                    Some(YulField::Offset) => Word::state_var_offset(symbol.as_str(), self),
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
    /// is its stack slot reinterpreted as a Yul one; `.offset` reads the calldata offset of a
    /// calldata reference and the byte offset within a slot of a storage one.
    fn local_reference(
        &mut self,
        base: &Identifier,
        suffix: Option<YulField>,
    ) -> YulReference<'context> {
        let (place, element_type) = self.function.identifier_place(base);
        match suffix {
            None => YulReference::Slot(place.yul_slot(self)),
            Some(YulField::Slot) => YulReference::Slot(place.yul_storage_slot(self)),
            Some(YulField::Offset) => {
                if element_type.data_location() == DataLocation::CallData {
                    YulReference::Slot(place.yul_calldata_offset(self))
                } else {
                    YulReference::Word(place.yul_storage_offset(self))
                }
            }
            Some(YulField::Length) => YulReference::Slot(place.yul_calldata_length(self)),
            Some(YulField::Selector) => YulReference::Slot(place.yul_selector(self)),
            Some(YulField::Address) => YulReference::Slot(place.yul_function_address(self)),
        }
    }

    /// The word a Solidity constant's initializer denotes. The initializer is folded at the
    /// constant's declared type before being reinterpreted, so a narrow or signed constant reaches
    /// Yul sign-extended and cleaned up rather than as the literal's own type.
    fn constant_word(
        &mut self,
        initializer: &Expression,
        declared_type: Option<Type>,
    ) -> YulReference<'context> {
        let declared_type = self.function.typing(declared_type);
        let value = self.function.converted(initializer, declared_type);
        YulReference::Word(value.yul_word(self))
    }
}

/// The field a suffixed Yul path projects out of a Solidity declaration.
#[derive(Clone, Copy)]
enum YulField {
    /// `.slot` of a storage reference or state variable.
    Slot,
    /// `.offset` of a storage or calldata reference.
    Offset,
    /// `.length` of a calldata reference.
    Length,
    /// `.selector` of an external function pointer.
    Selector,
    /// `.address` of an external function pointer.
    Address,
}

impl YulField {
    /// The field `built_in` names, absent for a built-in that is not a Yul path suffix.
    fn of(built_in: BuiltIn) -> Option<Self> {
        match built_in {
            BuiltIn::YulSlot => Some(Self::Slot),
            BuiltIn::YulOffset => Some(Self::Offset),
            BuiltIn::YulLengthField => Some(Self::Length),
            BuiltIn::YulSelector => Some(Self::Selector),
            BuiltIn::YulAddressField => Some(Self::Address),
            _ => None,
        }
    }
}
