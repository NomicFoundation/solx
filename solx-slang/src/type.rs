//!
//! The projection from Slang's semantic type tree onto Sol dialect types.
//!

use std::collections::HashMap;

use num_traits::sign::Signed;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionType as SlangFunctionType;
use slang_solidity_v2::ast::LiteralKind;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::StructDefinition;
use slang_solidity_v2::ast::Type;

use solx_mlir::ArraySize;
use solx_mlir::FunctionType;
use solx_mlir::Type as MlirType;

use crate::scope::source_unit::SourceUnitScope;

/// The struct types one walk has entered, by definition and data location: a recursive struct is
/// recorded before its members are walked, so a cycle back to it finds it opaque.
type EnteredStructures<'context> = HashMap<(NodeId, solx_utils::DataLocation), MlirType<'context>>;

/// The position of a type within its parent, deciding whether an identified struct still being
/// built may be embedded there before its body is known.
enum Position {
    /// The parent lays the type out inline, so it needs the struct's body.
    ByValue,
    /// The parent refers to the type without laying it out, so a cycle may break here.
    ByReference,
}

impl<'context> SourceUnitScope<'context> {
    /// Resolves a Slang semantic type to its Sol dialect MLIR type.
    ///
    /// `inherited_location` is the dialect data location to substitute when a type's Slang location
    /// is `Inherited` (struct-field-relative). Top-level callers pass `None`; `structure_members`
    /// sets it to the parent struct's location for its members.
    pub fn resolve(
        &self,
        node: &Type,
        inherited_location: Option<solx_utils::DataLocation>,
    ) -> MlirType<'context> {
        self.resolve_within(
            node,
            inherited_location,
            &mut EnteredStructures::new(),
            Position::ByValue,
        )
    }

    /// Resolves a function type's MLIR signature from the binder's type, so a callee naming no
    /// definition to look a registered signature up by resolves here.
    pub fn function_type(&self, function_type: &SlangFunctionType) -> FunctionType<'context> {
        self.function_type_within(function_type, &mut EnteredStructures::new())
    }

    /// The MLIR signature type `function` declares: its parameters and results.
    pub fn signature_type(&self, function: &FunctionDefinition) -> FunctionType<'context> {
        let Some(Type::Function(function_type)) = function.get_type() else {
            unreachable!("slang types every function definition");
        };
        self.function_type(&function_type)
    }

    /// Resolves the binder's typing of a node to its Sol dialect MLIR type.
    pub fn typing(&self, slang_type: Option<Type>) -> MlirType<'context> {
        self.resolve(
            &slang_type.expect("the binder types every expression"),
            None,
        )
    }

    /// The MLIR pointer type a `sol.gep` / `sol.map` / `sol.addr_of` yields for a value of this
    /// Slang type: mirrors `Sol_GepOp::build`'s non-ptr-ref-in-storage rule, where a
    /// reference-typed element living in `Storage` or `CallData` is its own storage pointer, so
    /// the pointer type is the element type itself.
    pub fn pointer(
        &self,
        node: &Type,
        element_type: MlirType<'context>,
        base_location: solx_utils::DataLocation,
    ) -> MlirType<'context> {
        if node.is_reference_type()
            && matches!(
                base_location,
                solx_utils::DataLocation::Storage | solx_utils::DataLocation::CallData
            )
        {
            return element_type;
        }
        MlirType::pointer(self.melior, element_type, base_location)
    }

    /// [`Self::resolve`] within one top-level resolution: `entered` holds the struct types the
    /// walk has entered, and `position` where the type sits in its parent. A recursive struct is
    /// an identified type, recorded opaque before its members are walked and returned as such
    /// where a cycle breaks: at a by-reference position naming a struct still opaque. A by-value
    /// position naming one rebuilds its body through this nested walk, so the enclosing walk's
    /// later identical body is a no-op. The walk terminates because every legal cycle carries a
    /// recursive struct entered by reference. A struct entered once returns at once, or a chain
    /// naming each successor twice would be re-walked at every level.
    fn resolve_within(
        &self,
        node: &Type,
        inherited_location: Option<solx_utils::DataLocation>,
        entered: &mut EnteredStructures<'context>,
        position: Position,
    ) -> MlirType<'context> {
        match node {
            Type::Integer(integer_type) => MlirType::integer(
                self.melior,
                integer_type.bits() as usize,
                integer_type.is_signed(),
            ),
            Type::FixedPointNumber(fixed_point_type) => MlirType::integer(
                self.melior,
                fixed_point_type.bits() as usize,
                fixed_point_type.is_signed(),
            ),
            Type::Boolean(_) => MlirType::boolean(self.melior),
            Type::Address(address) => MlirType::address(self.melior, address.is_payable()),
            Type::Literal(literal_type) => match literal_type.kind() {
                LiteralKind::Address { .. } => MlirType::address(self.melior, false),
                LiteralKind::Integer { value } => {
                    let bits = if value.is_negative() {
                        (-&value - 1u32).bits() + 1
                    } else {
                        value.bits().max(1)
                    };
                    let bits = usize::try_from(bits)
                        .expect("a literal's bit count fits the address width")
                        .next_multiple_of(solx_utils::BIT_LENGTH_BYTE)
                        .max(solx_utils::BIT_LENGTH_BYTE);
                    MlirType::integer(self.melior, bits, value.is_negative())
                }
                LiteralKind::HexInteger { bytes, .. } => {
                    let bits = bytes as usize * solx_utils::BIT_LENGTH_BYTE;
                    MlirType::unsigned(self.melior, bits)
                }
                LiteralKind::String { .. } => {
                    MlirType::string(self.melior, solx_utils::DataLocation::Memory)
                }
                LiteralKind::HexString { bytes } => MlirType::fixed_bytes(self.melior, bytes),
                LiteralKind::Rational { .. } => {
                    unreachable!("a rational literal folds into its integer-typed parent")
                }
            },
            Type::String(string_type) => {
                let location = solx_utils::DataLocation::from_slang(
                    string_type.location(),
                    inherited_location,
                );
                MlirType::string(self.melior, location)
            }
            Type::Bytes(bytes_type) => {
                let location =
                    solx_utils::DataLocation::from_slang(bytes_type.location(), inherited_location);
                MlirType::string(self.melior, location)
            }
            Type::ByteArray(byte_array_type) => {
                MlirType::fixed_bytes(self.melior, byte_array_type.width() as usize)
            }
            Type::Array(array_type) => {
                let element_type = self.resolve_within(
                    &array_type.element_type(),
                    inherited_location,
                    entered,
                    Position::ByReference,
                );
                let location =
                    solx_utils::DataLocation::from_slang(array_type.location(), inherited_location);
                MlirType::array(self.melior, ArraySize::Dynamic, element_type, location)
            }
            Type::FixedSizeArray(fixed_array_type) => {
                let element_type = self.resolve_within(
                    &fixed_array_type.element_type(),
                    inherited_location,
                    entered,
                    position,
                );
                let location = solx_utils::DataLocation::from_slang(
                    fixed_array_type.location(),
                    inherited_location,
                );
                MlirType::array(
                    self.melior,
                    ArraySize::Fixed(
                        u64::try_from(fixed_array_type.size()).expect("fixed array size fits u64"),
                    ),
                    element_type,
                    location,
                )
            }
            Type::Mapping(mapping_type) => {
                let key_type = self.resolve_within(
                    &mapping_type.key_type(),
                    Some(solx_utils::DataLocation::Storage),
                    entered,
                    Position::ByReference,
                );
                let value_type = self.resolve_within(
                    &mapping_type.value_type(),
                    Some(solx_utils::DataLocation::Storage),
                    entered,
                    Position::ByReference,
                );
                MlirType::mapping(self.melior, key_type, value_type)
            }
            Type::Struct(struct_type) => {
                let struct_location = solx_utils::DataLocation::from_slang(
                    struct_type.location(),
                    inherited_location,
                );
                let Definition::Struct(struct_definition) = struct_type.definition() else {
                    unreachable!("Slang StructType always references a Struct definition");
                };
                let key = (struct_definition.node_id(), struct_location);
                if let Some(&structure) = entered.get(&key)
                    && (!structure.structure_is_opaque()
                        || matches!(position, Position::ByReference))
                {
                    return structure;
                }
                if !struct_definition.is_recursive() {
                    let structure = MlirType::structure(
                        self.melior,
                        &self.structure_members(&struct_definition, struct_location, entered),
                        struct_location,
                    );
                    entered.insert(key, structure);
                    return structure;
                }
                // The node id keeps same-named structs of two scopes apart: the type uniquer's
                // key is not the module's symbol table.
                let structure = MlirType::identified_structure(
                    self.melior,
                    &format!(
                        "{}_{}",
                        struct_definition.name().name(),
                        struct_definition.node_id()
                    ),
                    struct_location,
                );
                entered.insert(key, structure);
                structure.set_body(&self.structure_members(
                    &struct_definition,
                    struct_location,
                    entered,
                ));
                structure
            }
            Type::Contract(inner) => self.object_type(inner.definition()),
            Type::Interface(inner) => self.object_type(inner.definition()),
            Type::Library(inner) => self.object_type(inner.definition()),
            Type::Enum(enum_type) => {
                let Definition::Enum(enum_definition) = enum_type.definition() else {
                    unreachable!("Slang EnumType always references an Enum definition");
                };
                let member_count = enum_definition.members().len();
                let max = u8::try_from(member_count - 1).expect("enum member count fits in u8");
                MlirType::enumeration(self.melior, max.into())
            }
            Type::Function(function_type) => self
                .function_type_within(function_type, entered)
                .reference(self.melior, function_type.visibility().into()),
            Type::UserDefinedValue(udvt) => {
                let target_type = udvt
                    .target_type()
                    .expect("UDVT target type resolved by semantic analysis");
                self.resolve_within(&target_type, inherited_location, entered, position)
            }
            _ => unimplemented!("unsupported Slang type"),
        }
    }

    /// [`Self::function_type`] within one top-level resolution, its parameters and results
    /// referred to without being laid out.
    fn function_type_within(
        &self,
        function_type: &SlangFunctionType,
        entered: &mut EnteredStructures<'context>,
    ) -> FunctionType<'context> {
        FunctionType {
            parameters: function_type
                .parameter_types()
                .iter()
                .map(|parameter_type| {
                    self.resolve_within(parameter_type, None, entered, Position::ByReference)
                })
                .collect(),
            results: match function_type.return_type() {
                Type::Void(_) => Vec::new(),
                Type::Tuple(tuple_type) => tuple_type
                    .types()
                    .iter()
                    .map(|element_type| {
                        self.resolve_within(element_type, None, entered, Position::ByReference)
                    })
                    .collect(),
                other => vec![self.resolve_within(&other, None, entered, Position::ByReference)],
            },
        }
    }

    /// The MLIR types of `definition`'s members, each resolved at the struct's own data
    /// location and laid out inline.
    fn structure_members(
        &self,
        definition: &StructDefinition,
        location: solx_utils::DataLocation,
        entered: &mut EnteredStructures<'context>,
    ) -> Vec<MlirType<'context>> {
        definition
            .members()
            .iter()
            .map(|member| {
                self.resolve_within(
                    &member
                        .get_type()
                        .expect("struct member type resolved by semantic analysis"),
                    Some(location),
                    entered,
                    Position::ByValue,
                )
            })
            .collect()
    }

    /// The Sol dialect type a value of an object type carries: the object identifier it is linked
    /// by. Only a contract declares the payable dispatch that lets a plain transfer reach it.
    fn object_type(&self, definition: Definition) -> MlirType<'context> {
        let (file_id, name, is_payable) = match &definition {
            Definition::Contract(node) => (node.get_file_id(), node.name(), node.is_payable()),
            Definition::Interface(node) => (node.get_file_id(), node.name(), false),
            Definition::Library(node) => (node.get_file_id(), node.name(), false),
            _ => unreachable!("slang types an object type by its own definition"),
        };
        MlirType::contract(
            self.melior,
            solx_utils::ContractName::full_path(file_id.as_str(), name.name()).as_str(),
            is_payable,
        )
    }
}
