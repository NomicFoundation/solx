//!
//! Yul path resolution: the Yul variables an assembly block declares and the Solidity declarations
//! it reaches through the `sol.yul_*` bridge ops.
//!

use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::Identifier;
use slang_solidity_v2::ast::Number;
use slang_solidity_v2::ast::Type;
use slang_solidity_v2::ast::YulPath;

use solx_mlir::Value;
use solx_mlir::Word;
use solx_utils::DataLocation;

use crate::contract::function::assembly::reference::YulField;
use crate::contract::function::assembly::reference::YulReference;
use crate::contract::object::Object;
use crate::scope::assembly::AssemblyScope;
use crate::scope::function::FunctionScope;
use crate::scope::source_unit::SourceUnitScope;

impl<'function, 'contract, 'source_unit, 'context>
    AssemblyScope<'function, 'contract, 'source_unit, 'context>
{
    /// What `path` denotes, resolved from the declaration it names together with the suffix that
    /// projects a field out of it: a bare name is either a Yul variable's own pointer or a Solidity
    /// local reinterpreted as one, while a constant, a library and a state variable's `.slot` /
    /// `.offset` are compile-time words. Slang gates the suffixes by neither the declaration nor
    /// its type, so the pairs Solidity rejects are spelled out here.
    pub fn reference(&mut self, path: &YulPath) -> YulReference<'context> {
        let mut identifiers = path.iter();
        let base = identifiers
            .next()
            .expect("a Yul path names at least one identifier");
        let suffix = identifiers.next().map(|member| {
            let field = member.resolve_to_built_in().unwrap_or_else(|| {
                unreachable!("{} carries no field named {}", base.name(), member.name())
            });
            YulField::from(field)
        });

        match (base.resolve_to_definition(), suffix) {
            (Some(definition), None)
                if let Some((initializer, declared_type)) =
                    FunctionScope::constant_definition(&definition) =>
            {
                let value = self.constant_value(&initializer, declared_type);
                YulReference::Word(value.yul_word(self))
            }
            (
                Some(Definition::YulVariable(declaration) | Definition::YulParameter(declaration)),
                None,
            ) => YulReference::Pointer(self.variable(declaration.node_id())),
            (Some(Definition::StateVariable(state_variable)), Some(YulField::Slot)) => {
                let symbol = SourceUnitScope::state_variable_symbol(&state_variable);
                YulReference::Word(Word::state_variable_slot(symbol.as_str(), self))
            }
            (Some(Definition::StateVariable(state_variable)), Some(YulField::Offset)) => {
                let symbol = SourceUnitScope::state_variable_symbol(&state_variable);
                YulReference::Word(Word::state_variable_offset(symbol.as_str(), self))
            }
            (Some(Definition::Variable(_) | Definition::Parameter(_)), suffix) => {
                self.local_reference(&base, suffix)
            }
            (Some(Definition::Library(library)), None) => {
                let address = Value::library_address(
                    Object::Library(library).identifier().as_str(),
                    self.function,
                );
                YulReference::Word(address.yul_word(self))
            }
            (_, suffix) => unreachable!(
                "a Yul path names a Yul variable, a Solidity local, a state variable, a constant \
                 or a library: {} carries {suffix:?}",
                base.name()
            ),
        }
    }

    /// The field of a Solidity local `base` that `suffix` projects, bridged into Yul. A bare local
    /// is its stack pointer reinterpreted as a Yul one; the suffixed forms read the field its data
    /// location carries, which is what tells `.offset` into a calldata reference from `.offset`
    /// within a storage slot.
    fn local_reference(
        &mut self,
        base: &Identifier,
        suffix: Option<YulField>,
    ) -> YulReference<'context> {
        if self.in_function() {
            unimplemented!(
                "a Yul function is isolated from the frame around it, so it cannot reach the \
                 Solidity variable {}",
                base.name()
            );
        }
        let (place, element_type) = self.function.identifier_place(base);
        match (suffix, element_type.data_location()) {
            (None, _) => YulReference::Pointer(place.yul_pointer(self)),
            (Some(YulField::Slot), DataLocation::Storage | DataLocation::Transient) => {
                YulReference::Pointer(place.yul_storage_slot(self))
            }
            (Some(YulField::Offset), DataLocation::Storage | DataLocation::Transient) => {
                YulReference::Word(place.yul_storage_offset(self))
            }
            (Some(YulField::Offset), DataLocation::CallData) => {
                YulReference::Pointer(place.yul_calldata_offset(self))
            }
            (Some(YulField::Length), DataLocation::CallData) => {
                YulReference::Pointer(place.yul_calldata_length(self))
            }
            (Some(YulField::Selector), DataLocation::Stack) => {
                YulReference::Pointer(place.yul_selector(self))
            }
            (Some(YulField::Address), DataLocation::Stack) => {
                YulReference::Pointer(place.yul_function_address(self))
            }
            (suffix, location) => unreachable!(
                "{} lives in {location:?}, which carries no {suffix:?}",
                base.name()
            ),
        }
    }

    /// The value a Solidity constant's initializer folds to at `declared_type`, which is what makes
    /// a narrow or signed constant reach Yul sign-extended and cleaned up.
    ///
    /// Only the shapes that fold reach here: anything else would emit its ops into the
    /// `sol.inline_asm` body, where an allocation bumps the free-memory pointer under the block's
    /// own memory arithmetic. `bytes32 constant B = A;` naming a string constant is the shape that
    /// reaches it without a call in sight - the identifier lands on `sol.string_lit`, a `MemAlloc`
    /// op, unless the string is folded at the bytes-like type here. That is the set Solidity
    /// rejects too, admitting "only direct number constants and references to such constants".
    fn constant_value(
        &mut self,
        node: &Expression,
        declared_type: Option<Type>,
    ) -> Value<'context> {
        let declared_type = self.function.typing(declared_type);
        if let Expression::StringExpression(literal) = node
            && declared_type.is_bytes_like()
        {
            return Value::left_aligned_bytes(literal.value(), declared_type, self.function);
        }

        let slang_type = node.get_type();
        let value = if let Some(Type::Literal(literal_type)) = &slang_type
            && let Some(Number::Integer(number)) = Number::from_literal_kind(&literal_type.kind())
        {
            let literal_type = self.function.typing(slang_type);
            Value::constant_from_bigint(&number, literal_type, self.function)
        } else {
            match node {
                Expression::TrueKeyword(_) => self.function.boolean_literal(true),
                Expression::FalseKeyword(_) => self.function.boolean_literal(false),
                Expression::Identifier(identifier) => {
                    let referenced = identifier
                        .resolve_to_definition()
                        .as_ref()
                        .and_then(FunctionScope::constant_definition);
                    let Some((initializer, referenced_type)) = referenced else {
                        unimplemented!("inline assembly reads only direct number constants");
                    };
                    self.constant_value(&initializer, referenced_type)
                }
                _ => unimplemented!("inline assembly reads only direct number constants"),
            }
        };
        value.convert(declared_type, self.function)
    }
}
