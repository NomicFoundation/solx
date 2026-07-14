//!
//! The `codegen!` node-projection macro and the `dispatch!` variant-table macro.
//!

/// Declares Slang AST node lowerings: each declaration stamps one crate-owned namespace type named
/// for the node, carrying the node's ENTIRE projection: nothing about a node lives outside its
/// declaration. The mode arrow fixes a method's name and return shape: `-> Value` is `emit`
/// yielding one [`solx_mlir::Value`], `-> Values` is `emit_values` yielding a value list,
/// `-> Effect` is `emit` yielding nothing, and `-> Place` is `emit_place` yielding the node's
/// address with its element type.
///
/// A hand body `Node -> Mode |node, scope| { .. }` binds the node and the enclosing function's
/// emission scope; a multi-node head `A | B -> Mode` stamps the same body onto several nodes.
/// A node with several projections declares them in one block, where plain `fn` items pass through
/// into the node's impl verbatim:
///
/// ```text
/// codegen!(
///     Identifier {
///         -> Value |node, scope| { .. }
///         -> Place |node, scope| { .. }
///         pub fn text(node: &ast::Identifier) -> String { .. }
///     }
/// );
/// ```
///
/// Operator families are declarative tables. `Node(OperatorEnum) -> binary { Variant => method }`
/// coerces both operands to the binder's result type and applies the row's [`solx_mlir::Value`]
/// method; `Node -> binary(method)` is the single-operator form. `-> compare { Variant => Predicate }`
/// maps operators onto `sol.cmp` predicates over [`solx_mlir::Value::compare_coerced`].
/// `-> compound { Variant => method }` lowers `lhs op= rhs` through the left operand's place; the
/// right operand is fully emitted before the place's old value is read, matching solc's IR
/// evaluation order, and plain `=`, built into the arm, never a row, stores it without the
/// read-modify load. A row binds several operator variants as `A | B => method`, and a `(checked)`
/// suffix, accepted after any `binary` or `compound` method, including the single-operator form,
/// threads the scope's checked-arithmetic flag; it is the vocabulary's only keyword, so any
/// other suffix fails to expand. The family arms resolve their operands through the
/// [`crate::contract::function::expression`] dispatchers.
///
/// `codegen!(@result_type Label, expression, scope)` is the shared binder-result-type projection
/// hand bodies may call: it resolves `expression.get_type()` to an MLIR type, with the first
/// argument serving only as the expect-message label.
macro_rules! codegen {
    () => {};

    (@method [$(#[$meta:meta])*] $node:ident, Value, $n:ident, $c:ident, $body:block) => {
        /// Emits the node's value.
        $(#[$meta])*
        pub fn emit<'context>(
            $n: &::slang_solidity_v2::ast::$node,
            $c: &mut $crate::scope::FunctionScope<'_, '_, 'context>,
        ) -> ::solx_mlir::Value<'context> $body
    };
    (@method [$(#[$meta:meta])*] $node:ident, Values, $n:ident, $c:ident, $body:block) => {
        /// Emits the node's values in declaration order.
        $(#[$meta])*
        pub fn emit_values<'context>(
            $n: &::slang_solidity_v2::ast::$node,
            $c: &mut $crate::scope::FunctionScope<'_, '_, 'context>,
        ) -> ::std::vec::Vec<::solx_mlir::Value<'context>> $body
    };
    (@method [$(#[$meta:meta])*] $node:ident, Effect, $n:ident, $c:ident, $body:block) => {
        /// Emits the node for its effects on the current block and environment.
        $(#[$meta])*
        pub fn emit<'context>(
            $n: &::slang_solidity_v2::ast::$node,
            $c: &mut $crate::scope::FunctionScope<'_, '_, 'context>,
        ) $body
    };
    (@method [$(#[$meta:meta])*] $node:ident, Place, $n:ident, $c:ident, $body:block) => {
        /// Emits the address the node resolves to together with its element MLIR type, without
        /// the trailing `sol.load`, serving both the read path and the assignment lvalue path.
        $(#[$meta])*
        pub fn emit_place<'context>(
            $n: &::slang_solidity_v2::ast::$node,
            $c: &mut $crate::scope::FunctionScope<'_, '_, 'context>,
        ) -> (::solx_mlir::Place<'context>, ::solx_mlir::Type<'context>) $body
    };

    (@block $node:ident {}) => {};
    (
        @block $node:ident {
            $(#[$meta:meta])*
            -> $mode:ident |$n:ident, $c:ident| $body:block
            $($rest:tt)*
        }
    ) => {
        codegen!(@method [$(#[$meta])*] $node, $mode, $n, $c, $body);
        codegen!(@block $node { $($rest)* });
    };
    (@block $node:ident { $item:item $($rest:tt)* }) => {
        $item
        codegen!(@block $node { $($rest)* });
    };

    (@apply $lhs:ident.$method:ident($rhs:ident) checked, $scope:ident) => {
        $lhs.$method($rhs, $scope.checked(), $scope)
    };
    (@apply $lhs:ident.$method:ident($rhs:ident), $scope:ident) => {
        $lhs.$method($rhs, $scope)
    };

    (@result_type $label:ident, $binding:expr, $scope:ident) => {
        $crate::r#type::Type::resolve(
            &$binding
                .get_type()
                .expect(concat!("binder types every ", stringify!($label))),
            None,
            $scope,
        )
    };

    (@forbid_equal Equal) => {
        compile_error!("plain `=` is built into the compound arm; do not declare an Equal row");
    };
    (@forbid_equal $variant:ident) => {};

    (@declare $node:ident { $($body:tt)* }) => {
        /// The frontend gives each Slang AST node its own projection.
        pub struct $node;

        impl $node {
            $($body)*
        }
    };

    (@binary_impl [$(#[$meta:meta])*] $node:ident { $($tail:tt)+ }) => {
        codegen!(@declare $node {
            /// Coerces both operands to the binder's result type and applies the operator.
            $(#[$meta])*
            pub fn emit<'context>(
                node: &::slang_solidity_v2::ast::$node,
                scope: &mut $crate::scope::FunctionScope<'_, '_, 'context>,
            ) -> ::solx_mlir::Value<'context> {
                let result_type = codegen!(@result_type $node, node, scope);
                let lhs = $crate::contract::function::expression::Expression::emit(&node.left_operand(), scope)
                    .coerce(result_type, scope);
                let rhs = $crate::contract::function::expression::Expression::emit(&node.right_operand(), scope)
                    .coerce(result_type, scope);
                codegen!(@binary_tail(lhs, rhs, scope, node) $($tail)+)
            }
        });
    };
    (
        @binary_tail($lhs:ident, $rhs:ident, $scope:ident, $node:ident)
        $operator:ident {
            $($($variant:ident)|+ => $method:ident $(($checked:ident))?),+ $(,)?
        }
    ) => {
        match $node.operator() {
            $($(::slang_solidity_v2::ast::$operator::$variant(_))|+ =>
                codegen!(@apply $lhs.$method($rhs) $($checked)?, $scope),)+
        }
    };
    (
        @binary_tail($lhs:ident, $rhs:ident, $scope:ident, $node:ident)
        $method:ident $(($checked:ident))?
    ) => {
        codegen!(@apply $lhs.$method($rhs) $($checked)?, $scope)
    };

    (
        $(#[$meta:meta])*
        $node:ident($operator:ident) -> binary { $($rows:tt)+ }
        $($rest:tt)*
    ) => {
        codegen!(@binary_impl [$(#[$meta])*] $node { $operator { $($rows)+ } });
        codegen!($($rest)*);
    };

    (
        $(#[$meta:meta])*
        $node:ident -> binary($method:ident $(($checked:ident))?)
        $($rest:tt)*
    ) => {
        codegen!(@binary_impl [$(#[$meta])*] $node { $method $(($checked))? });
        codegen!($($rest)*);
    };

    (
        $(#[$meta:meta])*
        $node:ident($operator:ident) -> compare {
            $($($variant:ident)|+ => $predicate:ident),+ $(,)?
        }
        $($rest:tt)*
    ) => {
        codegen!(@declare $node {
            /// Compares the operands under the operator's predicate.
            $(#[$meta])*
            pub fn emit<'context>(
                node: &::slang_solidity_v2::ast::$node,
                scope: &mut $crate::scope::FunctionScope<'_, '_, 'context>,
            ) -> ::solx_mlir::Value<'context> {
                let lhs = $crate::contract::function::expression::Expression::emit(&node.left_operand(), scope);
                let rhs = $crate::contract::function::expression::Expression::emit(&node.right_operand(), scope);
                let predicate = match node.operator() {
                    $($(::slang_solidity_v2::ast::$operator::$variant(_))|+ =>
                        ::solx_mlir::CmpPredicate::$predicate,)+
                };
                lhs.compare_coerced(rhs, predicate, scope)
            }
        });
        codegen!($($rest)*);
    };

    (
        $(#[$meta:meta])*
        $node:ident($operator:ident) -> compound {
            $($($variant:ident)|+ => $method:ident $(($checked:ident))?),+ $(,)?
        }
        $($rest:tt)*
    ) => {
        $($(codegen!(@forbid_equal $variant);)+)+

        codegen!(@declare $node {
            /// Lowers `lhs op= rhs` through the left operand's place; plain `=` stores the right
            /// operand without the read-modify load.
            $(#[$meta])*
            pub fn emit<'context>(
                node: &::slang_solidity_v2::ast::$node,
                scope: &mut $crate::scope::FunctionScope<'_, '_, 'context>,
            ) -> ::solx_mlir::Value<'context> {
                let (place, element_type) =
                    $crate::contract::function::expression::Expression::emit_place(&node.left_operand(), scope);
                if place.r#type() == element_type {
                    unimplemented!(
                        "assignment through a reference-typed place in storage or calldata is not yet supported"
                    );
                }
                let rhs = $crate::contract::function::expression::Expression::emit(&node.right_operand(), scope)
                    .coerce(element_type, scope);
                let stored = match node.operator() {
                    ::slang_solidity_v2::ast::$operator::Equal(_) => rhs,
                    $($(::slang_solidity_v2::ast::$operator::$variant(_))|+ => {
                        let loaded = place.load(element_type, scope);
                        codegen!(@apply loaded.$method(rhs) $($checked)?, scope)
                    })+
                };
                place.store(stored, scope);
                stored
            }
        });
        codegen!($($rest)*);
    };

    (
        $(#[$meta:meta])*
        $first:ident | $($node:ident)|+ -> $mode:ident |$n:ident, $c:ident| $body:block
        $($rest:tt)*
    ) => {
        codegen!($(#[$meta])* $first -> $mode |$n, $c| $body);
        codegen!($(#[$meta])* $($node)|+ -> $mode |$n, $c| $body $($rest)*);
    };

    (
        $(#[$meta:meta])*
        $node:ident -> $mode:ident |$n:ident, $c:ident| $body:block
        $($rest:tt)*
    ) => {
        codegen!(@declare $node {
            codegen!(@method [$(#[$meta])*] $node, $mode, $n, $c, $body);
        });
        codegen!($($rest)*);
    };

    (
        $node:ident { $($block:tt)* }
        $($rest:tt)*
    ) => {
        codegen!(@declare $node {
            codegen!(@block $node { $($block)* });
        });
        codegen!($($rest)*);
    };
}

/// Declares a dispatcher: a crate-owned type routing a Slang enum's variants onto their lowerings
/// as a table, one row per variant, each expanding to `Enum::Variant(inner)` calling the variant
/// type's method for the mode arrow. The `else` block splices verbatim match arms for the variants
/// whose lowering is not table-shaped, keeping the match exhaustive so a new upstream variant
/// fails at compile time. A dispatcher with several surfaces declares them in one block, where
/// plain `fn` items pass through into the dispatcher's impl verbatim:
///
/// ```text
/// dispatch!(
///     Expression(Expression) {
///         -> Value |node, scope| { Identifier, .. } else { .. }
///         -> Place |node, scope| { Identifier, .. } else { .. }
///         pub fn step(..) -> .. { .. }
///     }
/// );
/// ```
macro_rules! dispatch {
    () => {};

    (@declare $name:ident { $($body:tt)* }) => {
        /// The frontend routes each Slang enum through one dispatch table.
        pub struct $name;

        impl $name {
            $($body)*
        }
    };

    (@arm Value, $variant:ident, $inner:ident, $c:ident) => {
        $variant::emit($inner, $c)
    };
    (@arm Values, $variant:ident, $inner:ident, $c:ident) => {
        $variant::emit_values($inner, $c)
    };
    (@arm Effect, $variant:ident, $inner:ident, $c:ident) => {
        $variant::emit($inner, $c)
    };
    (@arm Place, $variant:ident, $inner:ident, $c:ident) => {
        $variant::emit_place($inner, $c)
    };

    (@dispatch $enum_:ident, $n:ident, $c:ident, $mode:ident, [$($variant:ident),+] [$($tail:tt)*]) => {
        match $n {
            $(::slang_solidity_v2::ast::$enum_::$variant(inner) =>
                dispatch!(@arm $mode, $variant, inner, $c),)+
            $($tail)*
        }
    };

    (@method [$(#[$meta:meta])*] $enum_:ident, Value, $n:ident, $c:ident, [$($variant:ident),+] [$($tail:tt)*]) => {
        /// Dispatches the node to its variant's lowering, yielding its value.
        $(#[$meta])*
        pub fn emit<'context>(
            $n: &::slang_solidity_v2::ast::$enum_,
            $c: &mut $crate::scope::FunctionScope<'_, '_, 'context>,
        ) -> ::solx_mlir::Value<'context> {
            dispatch!(@dispatch $enum_, $n, $c, Value, [$($variant),+] [$($tail)*])
        }
    };
    (@method [$(#[$meta:meta])*] $enum_:ident, Values, $n:ident, $c:ident, [$($variant:ident),+] [$($tail:tt)*]) => {
        /// Dispatches the node to its variant's lowering, yielding its values.
        $(#[$meta])*
        pub fn emit_values<'context>(
            $n: &::slang_solidity_v2::ast::$enum_,
            $c: &mut $crate::scope::FunctionScope<'_, '_, 'context>,
        ) -> ::std::vec::Vec<::solx_mlir::Value<'context>> {
            dispatch!(@dispatch $enum_, $n, $c, Values, [$($variant),+] [$($tail)*])
        }
    };
    (@method [$(#[$meta:meta])*] $enum_:ident, Effect, $n:ident, $c:ident, [$($variant:ident),+] [$($tail:tt)*]) => {
        /// Dispatches the node to its variant's lowering for its effects.
        $(#[$meta])*
        pub fn emit<'context>(
            $n: &::slang_solidity_v2::ast::$enum_,
            $c: &mut $crate::scope::FunctionScope<'_, '_, 'context>,
        ) {
            dispatch!(@dispatch $enum_, $n, $c, Effect, [$($variant),+] [$($tail)*])
        }
    };
    (@method [$(#[$meta:meta])*] $enum_:ident, Place, $n:ident, $c:ident, [$($variant:ident),+] [$($tail:tt)*]) => {
        /// Dispatches the node to its variant's place lowering.
        $(#[$meta])*
        pub fn emit_place<'context>(
            $n: &::slang_solidity_v2::ast::$enum_,
            $c: &mut $crate::scope::FunctionScope<'_, '_, 'context>,
        ) -> (::solx_mlir::Place<'context>, ::solx_mlir::Type<'context>) {
            dispatch!(@dispatch $enum_, $n, $c, Place, [$($variant),+] [$($tail)*])
        }
    };

    (@block $enum_:ident {}) => {};
    (
        @block $enum_:ident {
            $(#[$meta:meta])*
            -> $mode:ident |$n:ident, $c:ident| { $($variant:ident),+ $(,)? }
            else { $($tail:tt)+ }
            $($rest:tt)*
        }
    ) => {
        dispatch!(@method [$(#[$meta])*] $enum_, $mode, $n, $c, [$($variant),+] [$($tail)+]);
        dispatch!(@block $enum_ { $($rest)* });
    };
    (
        @block $enum_:ident {
            $(#[$meta:meta])*
            -> $mode:ident |$n:ident, $c:ident| { $($variant:ident),+ $(,)? }
            $($rest:tt)*
        }
    ) => {
        dispatch!(@method [$(#[$meta])*] $enum_, $mode, $n, $c, [$($variant),+] []);
        dispatch!(@block $enum_ { $($rest)* });
    };
    (@block $enum_:ident { $item:item $($rest:tt)* }) => {
        $item
        dispatch!(@block $enum_ { $($rest)* });
    };

    (
        $(#[$meta:meta])*
        $name:ident($enum_:ident) -> $mode:ident |$n:ident, $c:ident| { $($variant:ident),+ $(,)? }
        $(else { $($tail:tt)+ })?
    ) => {
        dispatch!(@declare $name {
            dispatch!(@method [$(#[$meta])*] $enum_, $mode, $n, $c, [$($variant),+] [$($($tail)+)?]);
        });
    };

    (
        $name:ident($enum_:ident) { $($block:tt)* }
    ) => {
        dispatch!(@declare $name {
            dispatch!(@block $enum_ { $($block)* });
        });
    };
}
