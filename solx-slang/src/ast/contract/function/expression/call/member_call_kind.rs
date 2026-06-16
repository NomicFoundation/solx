//!
//! The kind of a member-access call.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::Type as SlangType;

use crate::ast::BlockAnd;
use crate::ast::Materialize;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::library_visibility::LibraryVisibility;

/// The kind of a member-access call `x.f(...)` whose member resolves to a
/// definition (a function or state variable). A built-in member resolves to a
/// `BuiltIn` instead, handled by `ExpressionContext::emit_built_in_member_access`.
pub enum MemberCallKind {
    /// `super.f(...)` / a base-qualified call up the C3 linearisation.
    Super,
    /// A library call `L.f(...)`, by [`LibraryVisibility`] (external delegatecalls,
    /// internal inlines).
    Library(LibraryVisibility),
    /// A call through a member-resolved function pointer.
    FunctionPointer,
    /// `this.f(...)` — a self external call.
    SelfExternal,
    /// `instance.f(...)` — an external call on another instance.
    ExternalInstance,
    /// `this.x` — a self getter.
    SelfGetter,
    /// `instance.x` — an external getter.
    ExternalGetter,
}

impl MemberCallKind {
    /// Resolves a member-access callee `x.f(...)` to its kind.
    pub fn new<'state, 'context, 'block>(
        context: &ExpressionContext<'state, 'context, 'block>,
        access: &MemberAccessExpression,
    ) -> Self {
        let operand = access.operand();
        // `super.f()` or a recorded base redirect dispatches up the C3 chain.
        if matches!(operand, Expression::SuperKeyword(_))
            || context.state.super_redirect.contains_key(&access.node_id())
        {
            return Self::Super;
        }
        // `L.f()`: a selector-bearing (external/public) library fn delegatecalls;
        // an internal one inlines.
        if let Expression::Identifier(identifier) = &operand
            && matches!(
                identifier.resolve_to_definition(),
                Some(Definition::Library(_))
            )
        {
            let visibility = match access.member().resolve_to_definition() {
                Some(Definition::Function(function)) if function.compute_selector().is_some() => {
                    LibraryVisibility::External
                }
                _ => LibraryVisibility::Internal,
            };
            return Self::Library(visibility);
        }
        let is_this = matches!(operand, Expression::ThisKeyword(_));
        match access.member().resolve_to_definition() {
            Some(Definition::Function(function)) => {
                if is_this {
                    Self::SelfExternal
                } else if function.compute_selector().is_none() {
                    // using-for on an internal (no-selector) library fn: operand is `self`.
                    Self::Library(LibraryVisibility::Internal)
                } else if matches!(
                    function.enclosing_definition(),
                    Some(Definition::Library(_))
                ) {
                    // using-for / `L.f` onto an external library fn: a delegatecall.
                    Self::Library(LibraryVisibility::External)
                } else {
                    Self::ExternalInstance
                }
            }
            Some(Definition::StateVariable(_)) => {
                if is_this {
                    Self::SelfGetter
                } else if matches!(&operand, Expression::Identifier(identifier)
                    if matches!(identifier.resolve_to_definition(), Some(Definition::Contract(_))))
                    && matches!(access.get_type(), Some(SlangType::Function(_)))
                {
                    // `C.x(args)`: a function-pointer state variable read, then called.
                    Self::FunctionPointer
                } else {
                    Self::ExternalGetter
                }
            }
            // `s.f(...)` through a function-pointer struct field.
            Some(Definition::StructMember(_))
                if matches!(access.get_type(), Some(SlangType::Function(_))) =>
            {
                Self::FunctionPointer
            }
            other => unimplemented!(
                "unsupported member call: {:?}",
                other.map(|definition| definition.node_id())
            ),
        }
    }

    /// Whether a member-access operand `x` in `x.f(...)` is a namespace qualifier
    /// — a library or import alias (`L.f` / `M.f`), which is not a value — rather
    /// than a `using for` receiver, which becomes the implicit `self` argument.
    pub fn is_namespace_qualifier(operand: &Expression) -> bool {
        let Expression::Identifier(identifier) = operand else {
            return false;
        };
        matches!(
            identifier.resolve_to_definition(),
            Some(Definition::Library(_) | Definition::Import(_) | Definition::ImportedSymbol(_))
        )
    }

    /// Lowers this kind to its result values (a getter / call may yield zero or more).
    pub fn emit<'state, 'context, 'block>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        access: &MemberAccessExpression,
        call_value: Option<Value<'context, 'block>>,
        arguments: &ArgumentsDeclaration,
        block: BlockRef<'context, 'block>,
    ) -> (Vec<Value<'context, 'block>>, BlockRef<'context, 'block>) {
        // `L.f(...)` / using-for `x.f(...)` onto an external library function is
        // the only member call that accepts named arguments; ordering against the
        // explicit parameters collapses the positional and named forms.
        if let Self::Library(LibraryVisibility::External) = self {
            let (library_name, library_function, self_receiver) =
                ExpressionContext::resolve_external_library(access);
            let parameter_ids: Vec<NodeId> = library_function
                .parameters()
                .iter()
                .map(|parameter| parameter.node_id())
                .collect();
            // A using-for receiver is the implicit `self` first parameter, so the
            // named arguments name only the parameters after it.
            let explicit_parameter_ids = if self_receiver.is_some() {
                &parameter_ids[1..]
            } else {
                &parameter_ids[..]
            };
            let argument_expressions = arguments.ordered_by(explicit_parameter_ids);
            return self.emit_library_external_call(
                context,
                &library_name,
                &library_function,
                &argument_expressions,
                self_receiver.as_ref(),
                block,
            );
        }

        // Every other member-call shape takes positional arguments only.
        let ArgumentsDeclaration::PositionalArguments(arguments) = arguments else {
            unimplemented!("named arguments on this member call are not yet supported");
        };
        match self {
            Self::SelfExternal
            | Self::ExternalInstance
            | Self::SelfGetter
            | Self::ExternalGetter => {
                self.emit_external(context, access, call_value, arguments, block)
            }
            Self::Library(LibraryVisibility::Internal) => {
                let Some(Definition::Function(library_function)) =
                    access.member().resolve_to_definition()
                else {
                    unreachable!("a library call resolves to a function");
                };
                self.emit_library_call(context, access, &library_function, arguments, block)
            }
            Self::Super => {
                // `super.f` / `Base.f`: call the C3-resolved redirect target as an
                // internal function.
                let target_id = context
                    .state
                    .super_redirect
                    .get(&access.node_id())
                    .copied()
                    .expect("a super/base call has a recorded redirect target");
                let argument_expressions: Vec<Expression> = arguments.iter().collect();
                let function = context.state.resolve_function(target_id);
                let BlockAnd {
                    value: argument_values,
                    block: current_block,
                } = argument_expressions.materialize(&function.parameter_types, context, block);
                let results =
                    function.call(&argument_values, &context.state.builder, &current_block);
                (results, current_block)
            }
            Self::FunctionPointer => {
                // `s.f` through a function-pointer field: the indirect-call path on
                // the loaded `func_ref`.
                let callee = Expression::MemberAccessExpression(access.clone());
                let function_slang_type = access.get_type().expect("slang validated");
                context.emit_indirect_call_results(
                    &callee,
                    &function_slang_type,
                    arguments,
                    call_value,
                    block,
                )
            }
            Self::Library(LibraryVisibility::External) => {
                unreachable!("an external library call is handled above")
            }
        }
    }
}
