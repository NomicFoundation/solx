//!
//! ODS op-construction macros.
//!
//! `mlir_op_build!` / `mlir_op!` / `mlir_region_op!` collapse the ceremony of an ODS-generated op
//! builder (the `(context, unknown_location)` head and `.build().into()` tail) so a site states only
//! the op name and its setters.
//!

/// Builds an inlined dialect op and yields it as an `Operation`, without appending.
macro_rules! mlir_op_build {
    ($context:expr, $operation:ident $(.$method:ident($($argument:expr),* $(,)?))*) => {
        $operation::builder($context.melior, $context.location())
            $(.$method($($crate::IntoOds::into_ods($argument)),*))*
            .build()
            .into()
    };
}

/// Builds an inlined dialect op ([`mlir_op_build!`]), appends it to `$block`, and
/// returns its single result value. The `expect` message is derived from the op.
/// Omitting `$block` appends at the `current_block()` cursor. A `; ()` tail marks the value-less
/// form of a `$block` site: the op is appended for its effect and yields `()`.
macro_rules! mlir_op {
    ($context:expr, $operation:ident $(.$method:ident($($argument:expr),* $(,)?))*) => {
        mlir_op!($context, $context.current_block(), $operation $(.$method($($argument),*))*)
    };
    ($context:expr, $block:expr, $operation:ident $(.$method:ident($($argument:expr),* $(,)?))* ; ()) => {
        $block.append_operation(mlir_op_build!($context, $operation $(.$method($($argument),*))*));
    };
    ($context:expr, $block:expr, $operation:ident $(.$method:ident($($argument:expr),* $(,)?))*) => {
        $block
            .append_operation(mlir_op_build!($context, $operation $(.$method($($argument),*))*))
            .result(0)
            .expect(concat!(stringify!($operation), " produces one result"))
    };
}

/// Appends a region-bearing control-flow op (`sol.if`/`for`/`while`/`do`) and hands back each
/// region's fresh entry block for the caller to emit into and terminate. A trailing `; empty name…`
/// clause sets a region the op's shape requires but this method leaves bodiless — an `if` with no
/// `else` — and it is not handed back.
macro_rules! mlir_region_op {
    (
        $context:expr, $block:expr, $operation:ident
        $(.$method:ident($($argument:expr),* $(,)?))*
        ; $($region:ident),+
        $(; empty $($empty_region:ident),+)?
        $(,)?
    ) => {{
        $(
            let $region = {
                let region = melior::ir::Region::new();
                melior::ir::RegionLike::append_block(&region, melior::ir::Block::new(&[]));
                region
            };
        )+
        $($(
            let $empty_region = melior::ir::Region::new();
        )+)?
        let operation = melior::ir::BlockLike::append_operation(
            $block,
            $operation::builder($context.melior, $context.location())
                $(.$method($($crate::IntoOds::into_ods($argument)),*))*
                $(.$region($region))+
                $($(.$empty_region($empty_region))+)?
                .build()
                .into(),
        );
        let mut regions = (0usize..).map(|index| {
            melior::ir::operation::OperationLike::region(&operation, index)
                .expect(concat!(stringify!($operation), " region index in range"))
        });
        (
            $(
                $crate::Block::from(
                    melior::ir::RegionLike::first_block(
                        &regions.next().expect(concat!("missing ", stringify!($region))),
                    )
                    .expect(concat!(stringify!($region), " has an entry block")),
                )
            ),+
        )
    }};
}

/// A Sol dialect attribute enum built by a `solxCreate*Attr` FFI constructor: the `#[repr(u32)]`
/// enum plus its `attribute()` builder. `From`/other impls, where present, live alongside the call.
macro_rules! sol_dialect_attribute {
    (
        $(#[$enum_meta:meta])*
        $name:ident => $ffi:path {
            $($(#[$variant_meta:meta])* $variant:ident = $value:expr),+ $(,)?
        }
    ) => {
        $(#[$enum_meta])*
        #[repr(u32)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name {
            $($(#[$variant_meta])* $variant = $value),+
        }

        impl $name {
            /// Builds the corresponding Sol dialect attribute in `context`.
            pub fn attribute(self, context: &melior::Context) -> melior::ir::Attribute<'_> {
                unsafe { melior::ir::Attribute::from_raw($ffi(context.to_raw(), self as u32)) }
            }
        }
    };
}

/// A Sol comparison-predicate enum encoded as an `i64` `IntegerAttribute`: the `#[repr(i64)]` enum
/// plus its `attribute()` builder. `From`/other impls, where present, live alongside the call.
macro_rules! sol_predicate_attribute {
    (
        $(#[$enum_meta:meta])*
        $name:ident {
            $($(#[$variant_meta:meta])* $variant:ident = $value:expr),+ $(,)?
        }
    ) => {
        $(#[$enum_meta])*
        #[repr(i64)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name {
            $($(#[$variant_meta])* $variant = $value),+
        }

        impl $name {
            /// The `i64` `IntegerAttribute` this predicate's operand demands.
            pub fn attribute(
                self,
                context: &melior::Context,
            ) -> melior::ir::attribute::IntegerAttribute<'_> {
                melior::ir::attribute::IntegerAttribute::new(
                    melior::ir::r#type::IntegerType::new(context, solx_utils::BIT_LENGTH_X64 as u32)
                        .into(),
                    self as i64,
                )
            }
        }
    };
}

/// Converts an op-builder setter argument to the type the ODS setter expects.
///
/// A local trait rather than `From`/`Into`: the orphan rule forbids implementing `From` for the
/// foreign melior setter types the domain conversions target, so the macros route every argument
/// through it. The reflexive impl is the identity for an argument already of the setter's type.
pub trait IntoOds<T> {
    /// Converts `self` into the setter's argument type.
    fn into_ods(self) -> T;
}

impl<T> IntoOds<T> for T {
    fn into_ods(self) -> T {
        self
    }
}

impl<'slice, T, const N: usize> IntoOds<&'slice [T]> for &'slice [T; N] {
    fn into_ods(self) -> &'slice [T] {
        self
    }
}

/// Declares Sol dialect op-wrapper methods on their entity homes as pure data: one ODS operation
/// per declaration.
///
/// A declaration names the receiver, the method and its typed parameters, the disposition, the
/// operation, and the builder setter chain. Every setter argument is a parameter, the receiver
/// `self`, or a closed keyword from the `@arg` rules; keywords are call-shaped, so a bare
/// identifier is always a parameter.
///
/// A `base | flagged (…) … { … } flagged .setter ;` declaration stamps a pair of methods off one
/// chain: `base` omits the unit-flag setter and `flagged` appends `.setter(unit_flag)`, so a binary
/// mode is two named methods rather than one method taking a `bool`.
///
/// Dispositions: `-> value` / `-> place` append at the `current_block()` cursor and wrap the single
/// result; `-> value nop_if_same(param)` short-circuits when the receiver already has that type;
/// `-> value checked(CheckedOp)` threads a `checked: bool` selector that builds `CheckedOp` in place
/// of the declared operation off the same chain; an arrowless declaration is value-less and appends
/// to the receiver block for a `Block` method, or at the `current_block()` cursor for a `Value` /
/// `Place`. A `Block` declaration listing region names after `;` opens a region-bearing op and
/// returns each region's entry block, or the sole block when one region is named. Every argument is
/// routed through [`IntoOds`] to the setter's type.
macro_rules! sol_ops {
    () => {};

    (@ty i64) => { i64 };
    (@ty str) => { &str };
    (@ty bytes) => { &[u8] };
    (@ty value) => { $crate::Value<'context> };
    (@ty values) => { &[$crate::Value<'context>] };
    (@ty ty) => { $crate::Type<'context> };
    (@ty types) => { &[$crate::Type<'context>] };
    (@ty function) => { &$crate::Function<'context> };
    (@ty predicate) => { $crate::CmpPredicate };
    (@ty selector) => { u32 };
    (@ty optional_str) => { ::core::option::Option<&str> };
    (@ty optional_value) => { ::core::option::Option<$crate::Value<'context>> };

    (@arg [$context:ident] [$receiver:tt] self) => { $receiver.inner };
    (@arg [$context:ident] [$receiver:tt] self_ty) => { $receiver.r#type() };
    (@arg [$context:ident] [$receiver:tt] gep_of($element:ident)) => {
        $receiver.r#type().gep_result_type($element)
    };
    (@arg [$context:ident] [$receiver:tt] field()) => {
        $crate::Type::field($context.melior)
    };
    (@arg [$context:ident] [$receiver:tt] address()) => {
        $crate::Type::address($context.melior, false)
    };
    (@arg [$context:ident] [$receiver:tt] boolean()) => {
        $crate::Type::boolean($context.melior)
    };
    (@arg [$context:ident] [$receiver:tt] memory()) => {
        $crate::Type::string($context.melior, solx_utils::DataLocation::Memory)
    };
    (@arg [$context:ident] [$receiver:tt] calldata()) => {
        $crate::Type::string($context.melior, solx_utils::DataLocation::CallData)
    };
    (@arg [$context:ident] [$receiver:tt] fixed_bytes($width:expr)) => {
        $crate::Type::fixed_bytes($context.melior, $width)
    };
    (@arg [$context:ident] [$receiver:tt] selector()) => {
        $crate::Type::selector($context.melior)
    };
    (@arg [$context:ident] [$receiver:tt] ptr($pointee:ident, stack)) => {
        $crate::Type::pointer($context.melior, $pointee, solx_utils::DataLocation::Stack)
    };
    (@arg [$context:ident] [$receiver:tt] signature($callee:ident)) => {
        $crate::Type::new($callee.function_type.to_mlir($context.melior).into())
    };
    (@arg [$context:ident] [$receiver:tt] int_attr($value:ident, $result_type:ident)) => {
        ::melior::ir::Attribute::from(::melior::ir::attribute::IntegerAttribute::new(
            $result_type.into_mlir(),
            $value,
        ))
    };
    (@arg [$context:ident] [$receiver:tt] str_attr($text:expr)) => {
        ::melior::ir::attribute::StringAttribute::new($context.melior, $text)
    };
    (@arg [$context:ident] [$receiver:tt] bytes_attr($bytes:ident)) => {
        ::melior::ir::attribute::StringAttribute::from_bytes($context.melior, $bytes)
    };
    (@arg [$context:ident] [$receiver:tt] symbol_attr($name:expr)) => {
        ::melior::ir::attribute::FlatSymbolRefAttribute::new($context.melior, $name)
    };
    (@arg [$context:ident] [$receiver:tt] predicate_attr($predicate:ident)) => {
        ::melior::ir::Attribute::from($predicate.attribute($context.melior))
    };
    (@arg [$context:ident] [$receiver:tt] selector_attr($selector:ident)) => {
        $crate::Type::selector_attribute($selector, $context.melior)
    };
    (@arg [$context:ident] [$receiver:tt] ty_attr($($inner:tt)*)) => {
        ::melior::ir::attribute::TypeAttribute::new(
            sol_ops!(@arg [$context] [$receiver] $($inner)*).into_mlir(),
        )
    };
    (@arg [$context:ident] [$receiver:tt] count_attr($topics:ident)) => {
        ::melior::ir::attribute::IntegerAttribute::new(
            ::melior::ir::r#type::IntegerType::new(
                $context.melior,
                solx_utils::BIT_LENGTH_BYTE as u32,
            )
            .into(),
            i8::try_from($topics.len())
                .expect("EVM events carry at most four indexed arguments")
                .into(),
        )
    };
    (@arg [$context:ident] [$receiver:tt] $parameter:ident) => { $parameter };

    (@operands $iterator:expr) => {
        &$iterator
            .map(|operand| $crate::IntoOds::into_ods(*operand))
            .collect::<::std::vec::Vec<_>>()
    };

    (@chain $builder:ident [$context:ident] [$receiver:tt]) => { $builder };
    (@chain $builder:ident [$context:ident] [$receiver:tt] .$setter:ident (unit_flag) $($rest:tt)*) => {{
        let $builder = $builder.$setter(::melior::ir::Attribute::unit($context.melior));
        sol_ops!(@chain $builder [$context] [$receiver] $($rest)*)
    }};
    (@chain $builder:ident [$context:ident] [$receiver:tt] .$setter:ident (optional_str($text:ident)) $($rest:tt)*) => {{
        let $builder = if let ::core::option::Option::Some(__text) = $text {
            $builder.$setter(sol_ops!(@arg [$context] [$receiver] str_attr(__text)))
        } else {
            $builder
        };
        sol_ops!(@chain $builder [$context] [$receiver] $($rest)*)
    }};
    (@chain $builder:ident [$context:ident] [$receiver:tt] .$setter:ident (optional_value($operand:ident)) $($rest:tt)*) => {{
        let $builder = if let ::core::option::Option::Some(__operand) = $operand {
            $builder.$setter($crate::IntoOds::into_ods(__operand))
        } else {
            $builder
        };
        sol_ops!(@chain $builder [$context] [$receiver] $($rest)*)
    }};
    (@chain $builder:ident [$context:ident] [$receiver:tt] .$setter:ident (many($operands:expr)) $($rest:tt)*) => {{
        let $builder = $builder.$setter(sol_ops!(@operands $operands.iter()));
        sol_ops!(@chain $builder [$context] [$receiver] $($rest)*)
    }};
    (@chain $builder:ident [$context:ident] [$receiver:tt] .$setter:ident (status_and($result_types:ident)) $($rest:tt)*) => {{
        let status = [sol_ops!(@arg [$context] [$receiver] boolean())];
        sol_ops!(@chain $builder [$context] [$receiver] .$setter(concat(status, $result_types)) $($rest)*)
    }};
    (@chain $builder:ident [$context:ident] [$receiver:tt] .$setter:ident (concat($head:ident, $tail:ident)) $($rest:tt)*) => {{
        let $builder = $builder.$setter(sol_ops!(@operands $head.iter().chain($tail.iter())));
        sol_ops!(@chain $builder [$context] [$receiver] $($rest)*)
    }};
    (@chain $builder:ident [$context:ident] [$receiver:tt] .$setter:ident (single($operand:ident)) $($rest:tt)*) => {{
        let $builder = $builder.$setter(&[$crate::IntoOds::into_ods($operand)]);
        sol_ops!(@chain $builder [$context] [$receiver] $($rest)*)
    }};
    (@chain $builder:ident [$context:ident] [$receiver:tt] .$setter:ident ($($argument:tt)*) $($rest:tt)*) => {{
        let $builder = $builder.$setter($crate::IntoOds::into_ods(sol_ops!(@arg [$context] [$receiver] $($argument)*)));
        sol_ops!(@chain $builder [$context] [$receiver] $($rest)*)
    }};

    (@build [$context:ident] [$receiver:tt] $operation:ident $($chain:tt)*) => {
        {
            let builder = $operation::builder($context.melior, $context.location());
            sol_ops!(@chain builder [$context] [$receiver] $($chain)*)
        }
        .build()
        .into()
    };

    (@flag_ty $checked_op:ident) => { bool };
    (@op [$context:ident] [$receiver:tt] [$flag:ident] checked($checked_op:ident) $operation:ident $($chain:tt)*) => {
        if $flag {
            sol_ops!(@build [$context] [$receiver] $checked_op $($chain)*)
        } else {
            sol_ops!(@build [$context] [$receiver] $operation $($chain)*)
        }
    };
    (@op [$context:ident] [$receiver:tt] [$flag:ident] $operation:ident $($chain:tt)*) => {
        sol_ops!(@build [$context] [$receiver] $operation $($chain)*)
    };

    (@disp_ty) => { () };
    (@disp_ty value) => { $crate::Value<'context> };
    (@disp_ty place) => { $crate::Place<'context> };
    (@disp_ty values) => { ::std::vec::Vec<$crate::Value<'context>> };
    (@disp_ty status_and_values) => {
        ($crate::Value<'context>, ::std::vec::Vec<$crate::Value<'context>>)
    };

    (@region_tuple $region:ident ; empty $($empty_region:ident),+) => {
        $crate::Block<'context>
    };
    (@region_tuple $first:ident, $second:ident) => {
        ($crate::Block<'context>, $crate::Block<'context>)
    };
    (@region_tuple $first:ident, $second:ident, $third:ident) => {
        ($crate::Block<'context>, $crate::Block<'context>, $crate::Block<'context>)
    };

    (@one_result [$context:ident] $operation:expr, $message:expr) => {
        $context
            .current_block()
            .append_operation($operation)
            .result(0)
            .expect($message)
    };
    (@emit [$context:ident] $operation:expr, $message:expr) => {{
        $context.current_block().append_operation($operation);
    }};
    (@emit value [$context:ident] $operation:expr, $message:expr) => {
        $crate::Value::from(sol_ops!(@one_result [$context] $operation, $message))
    };
    (@emit place [$context:ident] $operation:expr, $message:expr) => {
        $crate::Place::from(sol_ops!(@one_result [$context] $operation, $message))
    };
    (@emit values [$context:ident] $operation:expr, $message:expr) => {{
        let operation = $context.current_block().append_operation($operation);
        (0..::melior::ir::operation::OperationLike::result_count(&operation))
            .map(|index| {
                $crate::Value::from(
                    ::melior::ir::operation::OperationLike::result(&operation, index)
                        .expect("the index is bounded by the result count"),
                )
            })
            .collect::<::std::vec::Vec<_>>()
    }};
    (@emit status_and_values [$context:ident] $operation:expr, $message:expr) => {{
        let mut results = sol_ops!(@emit values [$context] $operation, $message);
        (results.remove(0), results)
    }};

    (
        $receiver:ident :: $base:ident | $flagged:ident | $guarded:ident | $guarded_flagged:ident
        | $mode:ident | $mode_flagged:ident | $mode_guarded:ident | $mode_guarded_flagged:ident
        ($($parameters:tt)*)
        -> $disposition:ident { $operation:ident $($chain:tt)* }
        flagged .$setter:ident, .$guard:ident ; mode $(.$mode_flag:ident),+ ;
        $($rest:tt)*
    ) => {
        sol_ops!(
            $receiver :: $base | $flagged | $guarded | $guarded_flagged ($($parameters)*)
            -> $disposition { $operation $($chain)* } flagged .$setter, .$guard;
        );
        sol_ops!(
            $receiver :: $mode | $mode_flagged | $mode_guarded | $mode_guarded_flagged
            ($($parameters)*)
            -> $disposition { $operation $($chain)* $(.$mode_flag(unit_flag))+ }
            flagged .$setter, .$guard;
        );
        sol_ops!($($rest)*);
    };

    (
        $receiver:ident :: $base:ident | $flagged:ident | $guarded:ident | $guarded_flagged:ident
        ($($parameters:tt)*)
        -> $disposition:ident { $operation:ident $($chain:tt)* }
        flagged .$setter:ident, .$guard:ident ;
        $($rest:tt)*
    ) => {
        sol_ops!(
            $receiver :: $base | $flagged ($($parameters)*)
            -> $disposition { $operation $($chain)* } flagged .$setter;
        );
        sol_ops!(
            $receiver :: $guarded | $guarded_flagged ($($parameters)*)
            -> $disposition { $operation $($chain)* .$guard(unit_flag) } flagged .$setter;
        );
        sol_ops!($($rest)*);
    };

    (
        $receiver:ident :: $base:ident | $flagged:ident ($($parameters:tt)*)
        $(-> $disposition:ident)? { $operation:ident $($chain:tt)* } flagged .$setter:ident ;
        $($rest:tt)*
    ) => {
        sol_ops!($receiver :: $base ($($parameters)*) $(-> $disposition)? { $operation $($chain)* });
        sol_ops!(
            $receiver :: $flagged ($($parameters)*)
            $(-> $disposition)? { $operation $($chain)* .$setter(unit_flag) }
        );
        sol_ops!($($rest)*);
    };

    (
        Block :: $method:ident (self $(, $argument:ident : $kind:ident)* $(,)?)
        { $operation:ident $(.$setter:ident($($source:tt)*))* ; $($regions:tt)+ }
        $($rest:tt)*
    ) => {
        impl<'context> Block<'context> {
            pub fn $method(
                self,
                $($argument: sol_ops!(@ty $kind),)*
                context: &$crate::Context<'context>,
            ) -> sol_ops!(@region_tuple $($regions)+) {
                let receiver = self;
                mlir_region_op!(
                    context,
                    &receiver.inner,
                    $operation $(.$setter(sol_ops!(@arg [context] [receiver] $($source)*)))*
                    ; $($regions)+
                )
            }
        }
        sol_ops!($($rest)*);
    };

    (
        Block :: $method:ident (self $(, $argument:ident : $kind:ident)* $(,)?)
        { $operation:ident $($chain:tt)* }
        $($rest:tt)*
    ) => {
        impl<'context> Block<'context> {
            pub fn $method(
                self,
                $($argument: sol_ops!(@ty $kind),)*
                context: &$crate::Context<'context>,
            ) {
                let receiver = self;
                receiver
                    .inner
                    .append_operation(sol_ops!(@build [context] [receiver] $operation $($chain)*));
            }
        }
        sol_ops!($($rest)*);
    };

    (
        $receiver:ident :: $method:ident (self $(, $argument:ident : $kind:ident)* $(,)?)
        $(-> $disposition:ident)? $(nop_if_same($same:ident))? $(checked($checked_op:ident))?
        { $operation:ident $($chain:tt)* }
        $($rest:tt)*
    ) => {
        impl<'context> $receiver<'context> {
            pub fn $method(
                self,
                $($argument: sol_ops!(@ty $kind),)*
                $(checked: sol_ops!(@flag_ty $checked_op),)?
                context: &$crate::Context<'context>,
            ) -> sol_ops!(@disp_ty $($disposition)?) {
                let receiver = self;
                $(if receiver.r#type() == $same {
                    return receiver.into();
                })?
                sol_ops!(@emit $($disposition)? [context]
                    sol_ops!(@op [context] [receiver] [checked]
                        $(checked($checked_op))? $operation $($chain)*),
                    concat!(stringify!($operation), " produces one result"))
            }
        }
        sol_ops!($($rest)*);
    };

    (
        $receiver:ident :: $method:ident ($($argument:ident : $kind:ident),* $(,)?)
        $(-> $disposition:ident)? { $operation:ident $($chain:tt)* }
        $($rest:tt)*
    ) => {
        impl<'context> $receiver<'context> {
            pub fn $method(
                $($argument: sol_ops!(@ty $kind),)*
                context: &$crate::Context<'context>,
            ) -> sol_ops!(@disp_ty $($disposition)?) {
                sol_ops!(@emit $($disposition)? [context]
                    sol_ops!(@build [context] [()] $operation $($chain)*),
                    concat!(stringify!($operation), " produces one result"))
            }
        }
        sol_ops!($($rest)*);
    };
}
