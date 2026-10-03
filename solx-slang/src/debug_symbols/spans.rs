//!
//! The ranges of a source unit's nonterminal nodes.
//!

use std::collections::BTreeSet;
use std::ops::Range;

use slang_solidity_v2::ast;
use slang_solidity_v2::ast::NodeLocation;
use slang_solidity_v2::ast::SourceUnit;
use slang_solidity_v2::ast::visitor::Visitor;

/// The `[offset, length]` of every node visited that covers text, ordered and deduplicated.
#[derive(Default)]
struct Spans(BTreeSet<[usize; 2]>);

impl Spans {
    /// Records a node's range, if it covers any text. A modifier without attributes has an empty
    /// attribute node at offset 0.
    fn record(&mut self, range: Option<Range<usize>>) {
        if let Some(range) = range.filter(|range| !range.is_empty()) {
            self.0.insert(super::offset_length(&range));
        }
    }
}

/// Collects the spans of every sequence and non-empty collection node in `unit`. Choices are
/// skipped: each spans its variant, which is either recorded itself or a terminal.
pub fn spans(unit: &SourceUnit) -> Vec<[usize; 2]> {
    let mut spans = Spans::default();
    ast::visitor::accept_source_unit(unit, &mut spans);
    spans.0.into_iter().collect()
}

macro_rules! record_nonterminals {
    ($($enter:ident: $node:ident,)*) => {
        impl Visitor for Spans {
            $(
                fn $enter(&mut self, node: &ast::$node) -> bool {
                    self.record(node.calculate_text_range());
                    true
                }
            )*
        }
    };
}

// `ast` exports semantic types named `AddressType`, `FunctionType` and `MappingType`, so those three
// nodes go by their struct names.
record_nonterminals! {
    enter_abicoder_pragma: AbicoderPragma,
    enter_additive_expression: AdditiveExpression,
    enter_address_type: AddressTypeStruct,
    enter_and_expression: AndExpression,
    enter_array_expression: ArrayExpression,
    enter_array_type_name: ArrayTypeName,
    enter_assembly_statement: AssemblyStatement,
    enter_assignment_expression: AssignmentExpression,
    enter_bitwise_and_expression: BitwiseAndExpression,
    enter_bitwise_or_expression: BitwiseOrExpression,
    enter_bitwise_xor_expression: BitwiseXorExpression,
    enter_block: Block,
    enter_break_statement: BreakStatement,
    enter_call_options_expression: CallOptionsExpression,
    enter_catch_clause: CatchClause,
    enter_conditional_expression: ConditionalExpression,
    enter_constant_definition: ConstantDefinition,
    enter_continue_statement: ContinueStatement,
    enter_contract_definition: ContractDefinition,
    enter_decimal_number_expression: DecimalNumberExpression,
    enter_do_while_statement: DoWhileStatement,
    enter_emit_statement: EmitStatement,
    enter_enum_definition: EnumDefinition,
    enter_equality_expression: EqualityExpression,
    enter_error_definition: ErrorDefinition,
    enter_event_definition: EventDefinition,
    enter_experimental_pragma: ExperimentalPragma,
    enter_exponentiation_expression: ExponentiationExpression,
    enter_expression_statement: ExpressionStatement,
    enter_for_statement: ForStatement,
    enter_function_attributes: FunctionAttributes,
    enter_function_call_expression: FunctionCallExpression,
    enter_function_definition: FunctionDefinition,
    enter_function_type: FunctionTypeStruct,
    enter_function_type_attributes: FunctionTypeAttributes,
    enter_hex_number_expression: HexNumberExpression,
    enter_if_statement: IfStatement,
    enter_import_deconstruction: ImportDeconstruction,
    enter_import_deconstruction_symbol: ImportDeconstructionSymbol,
    enter_index_access_expression: IndexAccessExpression,
    enter_inequality_expression: InequalityExpression,
    enter_inheritance_type: InheritanceType,
    enter_interface_definition: InterfaceDefinition,
    enter_library_definition: LibraryDefinition,
    enter_mapping_type: MappingTypeStruct,
    enter_member_access_expression: MemberAccessExpression,
    enter_modifier_invocation: ModifierInvocation,
    enter_multi_typed_declaration: MultiTypedDeclaration,
    enter_multi_typed_declaration_element: MultiTypedDeclarationElement,
    enter_multiplicative_expression: MultiplicativeExpression,
    enter_named_argument: NamedArgument,
    enter_new_expression: NewExpression,
    enter_or_expression: OrExpression,
    enter_parameter: Parameter,
    enter_path_import: PathImport,
    enter_postfix_expression: PostfixExpression,
    enter_pragma_directive: PragmaDirective,
    enter_prefix_expression: PrefixExpression,
    enter_return_statement: ReturnStatement,
    enter_revert_statement: RevertStatement,
    enter_shift_expression: ShiftExpression,
    enter_single_typed_declaration: SingleTypedDeclaration,
    enter_source_unit: SourceUnit,
    enter_state_variable_attributes: StateVariableAttributes,
    enter_state_variable_definition: StateVariableDefinition,
    enter_struct_definition: StructDefinition,
    enter_struct_member: StructMember,
    enter_try_statement: TryStatement,
    enter_tuple_expression: TupleExpression,
    enter_tuple_value: TupleValue,
    enter_type_expression: TypeExpression,
    enter_unchecked_block: UncheckedBlock,
    enter_user_defined_value_type_definition: UserDefinedValueTypeDefinition,
    enter_using_deconstruction: UsingDeconstruction,
    enter_using_deconstruction_symbol: UsingDeconstructionSymbol,
    enter_using_directive: UsingDirective,
    enter_variable_declaration: VariableDeclaration,
    enter_variable_declaration_statement: VariableDeclarationStatement,
    enter_version_pragma: VersionPragma,
    enter_version_pragma_comparator: VersionPragmaComparator,
    enter_while_statement: WhileStatement,
    enter_yul_block: YulBlock,
    enter_yul_break_statement: YulBreakStatement,
    enter_yul_continue_statement: YulContinueStatement,
    enter_yul_default_case: YulDefaultCase,
    enter_yul_for_statement: YulForStatement,
    enter_yul_function_call_expression: YulFunctionCallExpression,
    enter_yul_function_definition: YulFunctionDefinition,
    enter_yul_if_statement: YulIfStatement,
    enter_yul_leave_statement: YulLeaveStatement,
    enter_yul_switch_statement: YulSwitchStatement,
    enter_yul_value_case: YulValueCase,
    enter_yul_variable_assignment_statement: YulVariableAssignmentStatement,
    enter_yul_variable_declaration_statement: YulVariableDeclarationStatement,
    enter_yul_variable_declaration_value: YulVariableDeclarationValue,
    enter_array_values: ArrayValues,
    enter_call_options: CallOptions,
    enter_catch_clauses: CatchClauses,
    enter_contract_members: ContractMembers,
    enter_enum_members: EnumMembers,
    enter_hex_string_literals: HexStringLiterals,
    enter_identifier_path: IdentifierPath,
    enter_import_deconstruction_symbols: ImportDeconstructionSymbols,
    enter_inheritance_types: InheritanceTypes,
    enter_interface_members: InterfaceMembers,
    enter_library_members: LibraryMembers,
    enter_modifier_invocations: ModifierInvocations,
    enter_multi_typed_declaration_elements: MultiTypedDeclarationElements,
    enter_named_arguments: NamedArguments,
    enter_override_paths: OverridePaths,
    enter_parameters: Parameters,
    enter_positional_arguments: PositionalArguments,
    enter_source_unit_members: SourceUnitMembers,
    enter_statements: Statements,
    enter_string_literals: StringLiterals,
    enter_struct_members: StructMembers,
    enter_tuple_values: TupleValues,
    enter_unicode_string_literals: UnicodeStringLiterals,
    enter_using_deconstruction_symbols: UsingDeconstructionSymbols,
    enter_version_pragma_expression_set: VersionPragmaExpressionSet,
    enter_version_pragma_expression_sets: VersionPragmaExpressionSets,
    enter_version_pragma_specifier: VersionPragmaSpecifier,
    enter_yul_arguments: YulArguments,
    enter_yul_parameters: YulParameters,
    enter_yul_path: YulPath,
    enter_yul_paths: YulPaths,
    enter_yul_statements: YulStatements,
    enter_yul_value_cases: YulValueCases,
    enter_yul_variable_names: YulVariableNames,
}
